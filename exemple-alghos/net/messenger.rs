//! P2P Messenger - High-level API for decentralized messaging
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::gossip::{GossipNetwork, GossipPayload, PresenceStatus};
use super::IrohNode;
use crate::messaging::{Identity, Conversation, Message, MessageContent};
use iroh_gossip::proto::TopicId;

/// Incoming message event
#[derive(Debug, Clone)]
pub enum P2PEvent {
    MessageReceived { conv_id: String, message: Message },
    PresenceUpdate { conv_id: String, peer_id: String, name: Option<String>, status: PresenceStatus },
    TypingUpdate { conv_id: String, peer_id: String, is_typing: bool },
}

/// P2P Messenger - manages all P2P messaging
pub struct P2PMessenger {
    identity: Identity,
    gossip: Arc<RwLock<GossipNetwork>>,
    conversations: HashMap<String, Conversation>,
    topics: HashMap<String, TopicId>,
}

impl P2PMessenger {
    /// Create new messenger from Iroh node
    pub async fn new(node: &IrohNode) -> Result<(Self, mpsc::UnboundedReceiver<P2PEvent>)> {
        let (gossip_tx, mut gossip_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let gossip = GossipNetwork::new(node.endpoint().clone(), gossip_tx).await?;

        let mut identity = Identity::generate();
        identity.public_key = node.public_bundle().pq.clone();

        // Spawn event processor
        tokio::spawn(async move {
            while let Some((conv_id, payload)) = gossip_rx.recv().await {
                let event = match payload {
                    GossipPayload::Message { id, sender_id, content, timestamp, reply_to } => {
                        P2PEvent::MessageReceived {
                            conv_id: conv_id.clone(),
                            message: Message {
                                id,
                                conversation_id: conv_id,
                                sender_id,
                                content: MessageContent::Text(content),
                                timestamp,
                                edited_at: None,
                                reply_to,
                                reactions: vec![],
                                is_encrypted: true,
                                expires_at: None,
                            },
                        }
                    }
                    GossipPayload::Presence { peer_id, name, status } => {
                        P2PEvent::PresenceUpdate { conv_id, peer_id, name, status }
                    }
                    GossipPayload::Typing { peer_id, is_typing } => {
                        P2PEvent::TypingUpdate { conv_id, peer_id, is_typing }
                    }
                    _ => continue,
                };
                let _ = event_tx.send(event);
            }
        });

        Ok((Self {
            identity,
            gossip: Arc::new(RwLock::new(gossip)),
            conversations: HashMap::new(),
            topics: HashMap::new(),
        }, event_rx))
    }

    pub fn identity(&self) -> &Identity { &self.identity }
    
    pub fn set_name(&mut self, name: String) { self.identity.display_name = Some(name); }

    /// Start a DM with a peer
    pub async fn start_dm(&mut self, peer_id: &str) -> Result<String> {
        let conv = Conversation::new_dm(peer_id, &self.identity.peer_id);
        let topic = GossipNetwork::dm_topic(&self.identity.peer_id, peer_id);
        let conv_id = conv.id.clone();
        
        self.gossip.write().await.subscribe(&conv_id, topic, vec![]).await?;
        self.topics.insert(conv_id.clone(), topic);
        self.conversations.insert(conv_id.clone(), conv);
        
        Ok(conv_id)
    }

    /// Create a group
    pub async fn create_group(&mut self, name: &str) -> Result<(String, String)> {
        let conv = Conversation::new_group(name, &self.identity.peer_id);
        let topic = GossipNetwork::random_topic();
        let conv_id = conv.id.clone();
        let invite = format!("vx-grp-{}", hex::encode(&topic.as_bytes()[..8]));
        
        self.gossip.write().await.subscribe(&conv_id, topic, vec![]).await?;
        self.topics.insert(conv_id.clone(), topic);
        self.conversations.insert(conv_id.clone(), conv);
        
        Ok((conv_id, invite))
    }

    /// Join a group via invite
    pub async fn join_group(&mut self, invite: &str) -> Result<String> {
        let topic_hex = invite.strip_prefix("vx-grp-").ok_or_else(|| anyhow::anyhow!("Invalid invite"))?;
        let mut topic_bytes = [0u8; 32];
        hex::decode_to_slice(topic_hex, &mut topic_bytes[..8])?;
        let topic = TopicId::from_bytes(topic_bytes);
        
        let conv = Conversation::new_group(&format!("Group {}", &topic_hex[..4]), &self.identity.peer_id);
        let conv_id = conv.id.clone();
        
        self.gossip.write().await.subscribe(&conv_id, topic, vec![]).await?;
        self.topics.insert(conv_id.clone(), topic);
        self.conversations.insert(conv_id.clone(), conv);
        
        Ok(conv_id)
    }

    /// Send a message
    pub async fn send_message(&self, conv_id: &str, content: &str) -> Result<Message> {
        let msg_id = format!("msg_{}", uuid::Uuid::new_v4());
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();

        self.gossip.read().await.broadcast(conv_id, GossipPayload::Message {
            id: msg_id.clone(),
            sender_id: self.identity.peer_id.clone(),
            content: content.to_string(),
            timestamp,
            reply_to: None,
        }).await?;

        Ok(Message {
            id: msg_id,
            conversation_id: conv_id.to_string(),
            sender_id: self.identity.peer_id.clone(),
            content: MessageContent::Text(content.to_string()),
            timestamp,
            edited_at: None,
            reply_to: None,
            reactions: vec![],
            is_encrypted: true,
            expires_at: None,
        })
    }

    pub async fn send_typing(&self, conv_id: &str, is_typing: bool) -> Result<()> {
        self.gossip.read().await.send_typing(conv_id, &self.identity.peer_id, is_typing).await
    }

    pub fn get_conversation(&self, conv_id: &str) -> Option<&Conversation> {
        self.conversations.get(conv_id)
    }

    pub fn conversations(&self) -> impl Iterator<Item = &Conversation> {
        self.conversations.values()
    }
}
