//! Unified P2P Messaging - DMs, Groups, Communities
//! All anonymous, decentralized, E2E encrypted
use serde::{Deserialize, Serialize};

/// Anonymous identity - no signup required
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Identity {
    pub peer_id: String,
    pub display_name: Option<String>,
    pub avatar_seed: u64,  // For generating identicon
    pub public_key: Vec<u8>,
    pub created_at: u64,
}

impl Identity {
    pub fn generate() -> Self {
        let seed = rand::random::<u64>();
        Self {
            peer_id: format!("vx_{:016x}", seed),
            display_name: None,
            avatar_seed: seed,
            public_key: vec![],  // Set by crypto layer
            created_at: now(),
        }
    }
    
    pub fn name(&self) -> String {
        self.display_name.clone().unwrap_or_else(|| {
            format!("Anon-{}", &self.peer_id[3..7])
        })
    }
}

/// Conversation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConversationType {
    DirectMessage,  // 1:1 private
    Group,          // Invite-only, shared key
    Community,      // P2P hosted, public/private channels
}

/// Unified conversation (DM, Group, or Community)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub conv_type: ConversationType,
    pub name: Option<String>,
    pub description: Option<String>,
    pub avatar_seed: u64,
    pub members: Vec<String>,  // peer_ids
    pub created_at: u64,
    pub last_message_at: u64,
    pub unread_count: u32,
    pub is_muted: bool,
    pub is_pinned: bool,
    // P2P hosting info
    pub iroh_doc_id: Option<String>,
    pub invite_code: Option<String>,
}

impl Conversation {
    /// Create a new DM
    pub fn new_dm(peer_id: &str, my_id: &str) -> Self {
        let id = format!("dm_{}_{}", 
            std::cmp::min(peer_id, my_id),
            std::cmp::max(peer_id, my_id)
        );
        Self {
            id,
            conv_type: ConversationType::DirectMessage,
            name: None,
            description: None,
            avatar_seed: rand::random(),
            members: vec![peer_id.into(), my_id.into()],
            created_at: now(),
            last_message_at: now(),
            unread_count: 0,
            is_muted: false,
            is_pinned: false,
            iroh_doc_id: None,
            invite_code: None,
        }
    }
    
    /// Create a new group
    pub fn new_group(name: &str, creator_id: &str) -> Self {
        Self {
            id: format!("grp_{:08x}", rand::random::<u32>()),
            conv_type: ConversationType::Group,
            name: Some(name.into()),
            description: None,
            avatar_seed: rand::random(),
            members: vec![creator_id.into()],
            created_at: now(),
            last_message_at: now(),
            unread_count: 0,
            is_muted: false,
            is_pinned: false,
            iroh_doc_id: None,
            invite_code: Some(generate_invite()),
        }
    }
    
    /// Create a new community (P2P hosted)
    pub fn new_community(name: &str, creator_id: &str) -> Self {
        Self {
            id: format!("com_{:08x}", rand::random::<u32>()),
            conv_type: ConversationType::Community,
            name: Some(name.into()),
            description: None,
            avatar_seed: rand::random(),
            members: vec![creator_id.into()],
            created_at: now(),
            last_message_at: now(),
            unread_count: 0,
            is_muted: false,
            is_pinned: false,
            iroh_doc_id: Some(format!("doc_{:016x}", rand::random::<u64>())),
            invite_code: Some(generate_invite()),
        }
    }
    
    pub fn display_name(&self, my_id: &str) -> String {
        match &self.conv_type {
            ConversationType::DirectMessage => {
                // Show other person's ID
                self.members.iter()
                    .find(|m| *m != my_id)
                    .map(|m| format!("Anon-{}", &m[3..7]))
                    .unwrap_or("Unknown".into())
            }
            _ => self.name.clone().unwrap_or("Unnamed".into())
        }
    }
}

/// Message in any conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub content: MessageContent,
    pub timestamp: u64,
    pub edited_at: Option<u64>,
    pub reply_to: Option<String>,
    pub reactions: Vec<Reaction>,
    pub is_encrypted: bool,
    pub expires_at: Option<u64>,  // Disappearing messages
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    File { name: String, size: u64, ticket: String },
    Image { ticket: String, width: u32, height: u32 },
    Voice { ticket: String, duration_secs: u32 },
    System(String),  // Join/leave notifications
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub user_ids: Vec<String>,
}

/// Community channel (text, voice, video)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub community_id: String,
    pub name: String,
    pub channel_type: ChannelType,
    pub topic: Option<String>,
    pub position: u32,
    pub is_private: bool,
    pub allowed_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChannelType {
    Text,
    Voice,
    Video,
    Announcements,
}

/// Privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub use_tor: bool,
    pub hide_online_status: bool,
    pub read_receipts: bool,
    pub typing_indicators: bool,
    pub default_message_expiry: Option<u64>,  // seconds
    pub block_unknown_dms: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            use_tor: false,
            hide_online_status: false,
            read_receipts: true,
            typing_indicators: true,
            default_message_expiry: None,
            block_unknown_dms: false,
        }
    }
}

// Helpers
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// MEDIUM FIX: Increased invite code entropy from 32-bit to 128-bit
/// 32-bit was brute-forceable (~4 billion possibilities)
/// 128-bit provides cryptographically secure entropy
fn generate_invite() -> String {
    let high: u64 = rand::random();
    let low: u64 = rand::random();
    format!("vx-{:016x}{:016x}", high, low)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identity() {
        let id = Identity::generate();
        assert!(id.peer_id.starts_with("vx_"));
        assert!(id.name().starts_with("Anon-"));
    }
    
    #[test]
    fn test_dm() {
        let dm = Conversation::new_dm("peer_a", "peer_b");
        assert_eq!(dm.conv_type, ConversationType::DirectMessage);
        assert_eq!(dm.members.len(), 2);
    }
}
