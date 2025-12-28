//! Social Manager - Coordinates P2P gossip messaging
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::{TopicId, random_topic};
use crate::messaging::{Identity, Conversation, ConversationType, Message, MessageContent};
use crate::community::Community;

/// Invite code for joining groups/communities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteCode {
    pub code: String,
    pub topic: TopicId,
    pub conv_type: ConversationType,
    pub name: String,
    pub creator_id: String,
    pub node_addrs: Vec<String>,  // Bootstrap nodes
}

impl InviteCode {
    pub fn encode(&self) -> String {
        let bytes = bincode::serialize(self).unwrap_or_default();
        format!("vx-{}", base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &bytes))
    }
    
    pub fn decode(s: &str) -> Option<Self> {
        let s = s.strip_prefix("vx-")?;
        let bytes = base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, s).ok()?;
        bincode::deserialize(&bytes).ok()
    }
}

/// Active subscription to a topic
#[derive(Debug)]
pub struct TopicSubscription {
    pub topic: TopicId,
    pub conv_id: String,
    pub conv_type: ConversationType,
}

/// Social state
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SocialState {
    pub identity: Option<Identity>,
    pub conversations: HashMap<String, Conversation>,
    pub messages: HashMap<String, Vec<Message>>,  // conv_id -> messages
    pub communities: HashMap<String, Community>,
    pub known_peers: HashMap<String, Identity>,
    pub privacy: crate::messaging::PrivacySettings,
}

impl SocialState {
    pub fn new() -> Self { Self::default() }
    
    /// Generate or load identity
    pub fn ensure_identity(&mut self) -> &Identity {
        if self.identity.is_none() {
            self.identity = Some(Identity::generate());
        }
        self.identity.as_ref().unwrap()
    }
    
    /// Get my peer ID
    pub fn my_id(&self) -> Option<&str> {
        self.identity.as_ref().map(|i| i.peer_id.as_str())
    }
    
    /// Create a DM conversation
    pub fn create_dm(&mut self, peer_id: &str) -> &Conversation {
        let my_id = self.identity.as_ref().map(|i| i.peer_id.as_str()).unwrap_or("unknown");
        let conv = Conversation::new_dm(peer_id, my_id);
        let id = conv.id.clone();
        self.conversations.entry(id.clone()).or_insert(conv);
        self.messages.entry(id.clone()).or_default();
        self.conversations.get(&id).unwrap()
    }
    
    /// Create a group
    pub fn create_group(&mut self, name: &str) -> (&Conversation, InviteCode) {
        let my_id = self.identity.as_ref().map(|i| i.peer_id.clone()).unwrap_or_default();
        let mut conv = Conversation::new_group(name, &my_id);
        let topic = random_topic();
        conv.iroh_doc_id = Some(hex::encode(topic));
        
        let invite = InviteCode {
            code: conv.invite_code.clone().unwrap_or_default(),
            topic,
            conv_type: ConversationType::Group,
            name: name.to_string(),
            creator_id: my_id,
            node_addrs: vec![],
        };
        
        let id = conv.id.clone();
        self.conversations.insert(id.clone(), conv);
        self.messages.entry(id.clone()).or_default();
        (self.conversations.get(&id).unwrap(), invite)
    }
    
    /// Create a community
    pub fn create_community(&mut self, name: &str, description: &str) -> (&Community, InviteCode) {
        let my_id = self.identity.as_ref().map(|i| i.peer_id.clone()).unwrap_or_default();
        let mut community = Community::new(name.to_string(), my_id.clone());
        community.description = description.to_string();
        
        let topic = random_topic();
        let invite = InviteCode {
            code: community.generate_invite(),
            topic,
            conv_type: ConversationType::Community,
            name: name.to_string(),
            creator_id: my_id,
            node_addrs: vec![],
        };
        
        let id = community.id.clone();
        self.communities.insert(id.clone(), community);
        (self.communities.get(&id).unwrap(), invite)
    }
    
    /// Join via invite code
    pub fn join_invite(&mut self, invite: &InviteCode) -> Result<String, &'static str> {
        match invite.conv_type {
            ConversationType::Group => {
                let my_id = self.my_id().ok_or("No identity")?.to_string();
                let mut conv = Conversation::new_group(&invite.name, &invite.creator_id);
                conv.id = format!("grp_{}", &invite.code);
                conv.iroh_doc_id = Some(hex::encode(invite.topic));
                conv.members.push(my_id);
                let id = conv.id.clone();
                self.conversations.insert(id.clone(), conv);
                self.messages.entry(id.clone()).or_default();
                Ok(id)
            }
            ConversationType::Community => {
                let my_id = self.my_id().ok_or("No identity")?.to_string();
                let my_name = self.identity.as_ref().map(|i| i.name()).unwrap_or_default();
                let mut community = Community::new(invite.name.clone(), invite.creator_id.clone());
                community.id = format!("com_{}", &invite.code);
                community.add_member(my_id, my_name);
                let id = community.id.clone();
                self.communities.insert(id.clone(), community);
                Ok(id)
            }
            ConversationType::DirectMessage => Err("Cannot join DM via invite"),
        }
    }
    
    /// Add a message to conversation
    pub fn add_message(&mut self, conv_id: &str, msg: Message) {
        if let Some(conv) = self.conversations.get_mut(conv_id) {
            conv.last_message_at = msg.timestamp;
            conv.unread_count += 1;
        }
        self.messages.entry(conv_id.to_string()).or_default().push(msg);
    }
    
    /// Get messages for conversation
    pub fn get_messages(&self, conv_id: &str, limit: usize) -> Vec<&Message> {
        self.messages.get(conv_id)
            .map(|msgs| msgs.iter().rev().take(limit).collect::<Vec<_>>().into_iter().rev().collect())
            .unwrap_or_default()
    }
    
    /// Mark conversation as read
    pub fn mark_read(&mut self, conv_id: &str) {
        if let Some(conv) = self.conversations.get_mut(conv_id) {
            conv.unread_count = 0;
        }
    }
    
    /// Get sorted conversations (pinned first, then by last message)
    pub fn get_conversations(&self) -> Vec<&Conversation> {
        let mut convs: Vec<_> = self.conversations.values().collect();
        convs.sort_by(|a, b| {
            match (a.is_pinned, b.is_pinned) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.last_message_at.cmp(&a.last_message_at),
            }
        });
        convs
    }
    
    /// Register a known peer
    pub fn register_peer(&mut self, identity: Identity) {
        self.known_peers.insert(identity.peer_id.clone(), identity);
    }
    
    /// Get peer name
    pub fn peer_name(&self, peer_id: &str) -> String {
        self.known_peers.get(peer_id)
            .map(|i| i.name())
            .unwrap_or_else(|| format!("Anon-{}", &peer_id[3..7.min(peer_id.len())]))
    }
}

/// Create a new chat message
pub fn new_message(conv_id: &str, sender_id: &str, content: MessageContent) -> Message {
    Message {
        id: format!("msg_{}", uuid::Uuid::new_v4()),
        conversation_id: conv_id.to_string(),
        sender_id: sender_id.to_string(),
        content,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        edited_at: None,
        reply_to: None,
        reactions: vec![],
        is_encrypted: true,
        expires_at: None,
    }
}
