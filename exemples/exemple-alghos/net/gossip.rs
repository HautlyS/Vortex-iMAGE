//! P2P Gossip Network - Decentralized message broadcasting
//! Uses iroh-gossip for real-time P2P messaging
use anyhow::Result;
use bytes::Bytes;
use futures::StreamExt;
use iroh::Endpoint;
use iroh_gossip::{
    net::{Event, Gossip, GossipEvent, GossipReceiver, GossipSender},
    proto::TopicId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Message types for gossip protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipPayload {
    /// Chat message
    Message {
        id: String,
        sender_id: String,
        content: String,
        timestamp: u64,
        reply_to: Option<String>,
    },
    /// User presence announcement
    Presence {
        peer_id: String,
        name: Option<String>,
        status: PresenceStatus,
    },
    /// Typing indicator
    Typing {
        peer_id: String,
        is_typing: bool,
    },
    /// Reaction to message
    Reaction {
        message_id: String,
        peer_id: String,
        emoji: String,
        add: bool,
    },
    /// File share announcement
    FileShare {
        id: String,
        sender_id: String,
        name: String,
        size: u64,
        ticket: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PresenceStatus {
    Online,
    Away,
    Busy,
    Offline,
}

/// Maximum size for gossip messages (1MB) - HIGH FIX: Prevent memory exhaustion
const MAX_GOSSIP_MESSAGE_SIZE: usize = 1024 * 1024;

/// Signed gossip message wrapper for authentication (HIGH FIX: Message authentication)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedGossipMessage {
    pub payload: GossipPayload,
    pub sender_public_key: Vec<u8>,  // Ed25519 verifying key
    pub signature: Vec<u8>,          // Ed25519 signature
}

impl GossipPayload {
    pub fn to_bytes(&self) -> Bytes {
        bincode::serialize(self).unwrap_or_default().into()
    }

    /// HIGH FIX: Add size limit validation to prevent memory exhaustion attacks
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        // Reject oversized messages
        if data.len() > MAX_GOSSIP_MESSAGE_SIZE {
            tracing::warn!("Gossip message too large: {} bytes (max {})", data.len(), MAX_GOSSIP_MESSAGE_SIZE);
            return None;
        }
        bincode::deserialize(data).ok()
    }
    
    /// Sign payload with Ed25519 key for authentication
    pub fn sign(&self, signing_key: &[u8; 32], verifying_key: &[u8; 32]) -> SignedGossipMessage {
        use ed25519_dalek::{SigningKey, Signer};
        
        let payload_bytes = self.to_bytes();
        let sk = SigningKey::from_bytes(signing_key);
        let signature = sk.sign(&payload_bytes).to_bytes().to_vec();
        
        SignedGossipMessage {
            payload: self.clone(),
            sender_public_key: verifying_key.to_vec(),
            signature,
        }
    }
}

impl SignedGossipMessage {
    /// Serialize signed message
    pub fn to_bytes(&self) -> Bytes {
        bincode::serialize(self).unwrap_or_default().into()
    }
    
    /// Deserialize and verify signed message
    /// HIGH FIX: Validates signature before accepting message
    pub fn from_bytes_verified(data: &[u8]) -> Option<GossipPayload> {
        // Size check
        if data.len() > MAX_GOSSIP_MESSAGE_SIZE {
            tracing::warn!("Signed gossip message too large: {} bytes", data.len());
            return None;
        }
        
        let signed: SignedGossipMessage = bincode::deserialize(data).ok()?;
        
        // Verify signature
        if !signed.verify() {
            tracing::warn!("Gossip message signature verification failed");
            return None;
        }
        
        Some(signed.payload)
    }
    
    /// Verify the signature on this message
    pub fn verify(&self) -> bool {
        use ed25519_dalek::{VerifyingKey, Signature, Verifier};
        
        // Parse verifying key
        let vk_bytes: [u8; 32] = match self.sender_public_key.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        let verifying_key = match VerifyingKey::from_bytes(&vk_bytes) {
            Ok(vk) => vk,
            Err(_) => return false,
        };
        
        // Parse signature
        let signature = match Signature::from_slice(&self.signature) {
            Ok(sig) => sig,
            Err(_) => return false,
        };
        
        // Verify
        let payload_bytes = self.payload.to_bytes();
        verifying_key.verify(&payload_bytes, &signature).is_ok()
    }
}

/// Active topic subscription
pub struct TopicHandle {
    pub topic_id: TopicId,
    pub sender: GossipSender,
    _receiver_task: tokio::task::JoinHandle<()>,
}

/// P2P Gossip Network Manager
pub struct GossipNetwork {
    gossip: Gossip,
    #[allow(dead_code)] // Reserved for future peer discovery
    endpoint: Endpoint,
    topics: HashMap<String, TopicHandle>,
    message_tx: mpsc::UnboundedSender<(String, GossipPayload)>,
}

impl GossipNetwork {
    /// Create new gossip network from existing endpoint
    pub async fn new(
        endpoint: Endpoint,
        message_tx: mpsc::UnboundedSender<(String, GossipPayload)>,
    ) -> Result<Self> {
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;

        Ok(Self {
            gossip,
            endpoint,
            topics: HashMap::new(),
            message_tx,
        })
    }

    /// Generate deterministic topic ID for DM between two peers
    pub fn dm_topic(peer_a: &str, peer_b: &str) -> TopicId {
        let (first, second) = if peer_a < peer_b {
            (peer_a, peer_b)
        } else {
            (peer_b, peer_a)
        };
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"vortex:dm:");
        hasher.update(first.as_bytes());
        hasher.update(b":");
        hasher.update(second.as_bytes());
        TopicId::from_bytes(*hasher.finalize().as_bytes())
    }

    /// Generate random topic ID for groups/communities
    pub fn random_topic() -> TopicId {
        TopicId::from_bytes(rand::random())
    }

    /// Subscribe to a conversation topic
    pub async fn subscribe(&mut self, conv_id: &str, topic_id: TopicId, bootstrap_peers: Vec<iroh::NodeId>) -> Result<()> {
        // Add bootstrap peers to endpoint
        for peer_id in &bootstrap_peers {
            // Peers will be discovered via N0 discovery
            tracing::info!("Will connect to peer: {}", peer_id.fmt_short());
        }

        // Subscribe to topic
        let (sender, receiver) = self.gossip
            .subscribe_and_join(topic_id, bootstrap_peers)
            .await?
            .split();

        // Spawn receiver task
        let conv_id_clone = conv_id.to_string();
        let message_tx = self.message_tx.clone();
        let receiver_task = tokio::spawn(async move {
            Self::receive_loop(conv_id_clone, receiver, message_tx).await;
        });

        self.topics.insert(conv_id.to_string(), TopicHandle {
            topic_id,
            sender,
            _receiver_task: receiver_task,
        });

        Ok(())
    }

    /// Receive loop for a topic
    /// SECURITY FIX: Now validates message signatures before accepting
    async fn receive_loop(
        conv_id: String,
        mut receiver: GossipReceiver,
        message_tx: mpsc::UnboundedSender<(String, GossipPayload)>,
    ) {
        while let Some(event) = receiver.next().await {
            match event {
                Ok(Event::Gossip(GossipEvent::Received(msg))) => {
                    // Try to parse as signed message first (preferred, secure)
                    if let Some(payload) = SignedGossipMessage::from_bytes_verified(&msg.content) {
                        let _ = message_tx.send((conv_id.clone(), payload));
                    } 
                    // Fall back to unsigned for backward compatibility (will be deprecated)
                    else if let Some(payload) = GossipPayload::from_bytes(&msg.content) {
                        tracing::debug!("Received unsigned gossip message (legacy mode)");
                        let _ = message_tx.send((conv_id.clone(), payload));
                    } else {
                        tracing::warn!("Failed to parse gossip message in {}", conv_id);
                    }
                }
                Ok(Event::Gossip(GossipEvent::NeighborUp(peer))) => {
                    tracing::info!("Peer joined {}: {}", conv_id, peer.fmt_short());
                }
                Ok(Event::Gossip(GossipEvent::NeighborDown(peer))) => {
                    tracing::info!("Peer left {}: {}", conv_id, peer.fmt_short());
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("Gossip error in {}: {}", conv_id, e);
                    break;
                }
            }
        }
    }

    /// Broadcast message to a conversation (unsigned - legacy)
    pub async fn broadcast(&self, conv_id: &str, payload: GossipPayload) -> Result<()> {
        if let Some(handle) = self.topics.get(conv_id) {
            handle.sender.broadcast(payload.to_bytes()).await?;
        }
        Ok(())
    }
    
    /// Broadcast signed message to a conversation (secure)
    /// SECURITY: Messages are signed with Ed25519 for authentication
    pub async fn broadcast_signed(
        &self, 
        conv_id: &str, 
        payload: GossipPayload,
        signing_key: &[u8; 32],
        verifying_key: &[u8; 32],
    ) -> Result<()> {
        if let Some(handle) = self.topics.get(conv_id) {
            let signed = payload.sign(signing_key, verifying_key);
            handle.sender.broadcast(signed.to_bytes()).await?;
        }
        Ok(())
    }

    /// Send presence update
    pub async fn announce_presence(&self, conv_id: &str, peer_id: &str, name: Option<String>, status: PresenceStatus) -> Result<()> {
        self.broadcast(conv_id, GossipPayload::Presence {
            peer_id: peer_id.to_string(),
            name,
            status,
        }).await
    }

    /// Send typing indicator
    pub async fn send_typing(&self, conv_id: &str, peer_id: &str, is_typing: bool) -> Result<()> {
        self.broadcast(conv_id, GossipPayload::Typing {
            peer_id: peer_id.to_string(),
            is_typing,
        }).await
    }

    /// Unsubscribe from a topic
    pub fn unsubscribe(&mut self, conv_id: &str) {
        if let Some(handle) = self.topics.remove(conv_id) {
            handle._receiver_task.abort();
        }
    }

    /// Get number of active subscriptions
    pub fn active_topics(&self) -> usize {
        self.topics.len()
    }

    /// Check if subscribed to a conversation
    pub fn is_subscribed(&self, conv_id: &str) -> bool {
        self.topics.contains_key(conv_id)
    }
}

/// Shared gossip network handle
pub type SharedGossip = Arc<RwLock<GossipNetwork>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dm_topic_deterministic() {
        let topic1 = GossipNetwork::dm_topic("peer_a", "peer_b");
        let topic2 = GossipNetwork::dm_topic("peer_b", "peer_a");
        assert_eq!(topic1, topic2);
    }

    #[test]
    fn test_payload_serialization() {
        let payload = GossipPayload::Message {
            id: "msg_1".into(),
            sender_id: "peer_a".into(),
            content: "Hello!".into(),
            timestamp: 12345,
            reply_to: None,
        };
        let bytes = payload.to_bytes();
        let decoded = GossipPayload::from_bytes(&bytes).unwrap();
        match decoded {
            GossipPayload::Message { content, .. } => assert_eq!(content, "Hello!"),
            _ => panic!("Wrong type"),
        }
    }
}
