//! P2P Chat with end-to-end encryption and signatures
pub mod queue;
pub mod enhanced;

use serde::{Deserialize, Serialize};

pub use enhanced::{
    ChannelType, PermissionValue, PresenceSession, PresenceManager,
    TypingIndicator, TypingManager, EnhancedChatMessage, Embed, EmbedType,
    Masquerade, PaginationCursor, PaginatedMessages, ChannelPermissions,
    PermissionQuery, DefaultPermissionCalculator, MessageHistory,
    UserId as EnhancedUserId, ChannelId, ServerId, RoleId, EnhancedMessageId,
};
use crate::crypto::{encrypt, decrypt, PublicBundle, HybridKeypair, EncryptedPayload, CryptoError};

pub use queue::{MessageQueue, QueuedMessage, HistoryReconciler};

/// Unique message ID
pub type MessageId = String;

/// Chat message with signature for authenticity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub room_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: MessageContent,
    pub timestamp: u64,
    pub signature: Vec<u8>,  // Ed25519 signature
    pub encrypted: bool,
    #[serde(default)]
    pub expires_at: Option<u64>,  // For disappearing messages
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MessageContent {
    Text(String),
    File { name: String, size: u64, ticket: String },
    Image { name: String, ticket: String },
    Reaction { target_id: MessageId, emoji: String },
    System(String),
}

/// Chat room
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatRoom {
    pub id: String,
    pub name: String,
    pub members: Vec<RoomMember>,
    pub messages: Vec<Message>,
    pub created_at: u64,
    pub is_group: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomMember {
    pub id: String,
    pub name: String,
    pub public_bundle: Option<PublicBundle>,
    pub online: bool,
    pub typing: bool,
}

impl ChatRoom {
    /// Create a DM room with exactly 2 members (self + peer)
    pub fn new_dm(id: String, self_member: RoomMember, peer: RoomMember) -> Self {
        Self {
            id,
            name: peer.name.clone(),
            members: vec![self_member, peer],
            messages: Vec::new(),
            created_at: now(),
            is_group: false,
        }
    }

    pub fn new_group(id: String, name: String) -> Self {
        Self {
            id,
            name,
            members: Vec::new(),
            messages: Vec::new(),
            created_at: now(),
            is_group: true,
        }
    }

    pub fn add_member(&mut self, member: RoomMember) {
        if !self.members.iter().any(|m| m.id == member.id) {
            self.members.push(member);
        }
    }

    pub fn remove_member(&mut self, member_id: &str) {
        self.members.retain(|m| m.id != member_id);
    }

    pub fn add_message(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    pub fn unread_count(&self, last_read_timestamp: u64) -> usize {
        self.messages.iter().filter(|m| m.timestamp > last_read_timestamp).count()
    }

    /// Get member count (for DM validation)
    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

/// Encrypted message for transport
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub id: MessageId,
    pub room_id: String,
    pub sender_id: String,
    pub payload: EncryptedPayload,
    pub timestamp: u64,
}

impl EncryptedMessage {
    pub fn encrypt(msg: &Message, recipient: &PublicBundle) -> Result<Self, CryptoError> {
        let data = bincode::serialize(msg).map_err(|_| CryptoError::Encrypt)?;
        let payload = encrypt(&data, recipient)?;
        Ok(Self {
            id: msg.id.clone(),
            room_id: msg.room_id.clone(),
            sender_id: msg.sender_id.clone(),
            payload,
            timestamp: msg.timestamp,
        })
    }

    pub fn decrypt(&self, keypair: &HybridKeypair) -> Result<Message, CryptoError> {
        let data = decrypt(&self.payload, keypair)?;
        bincode::deserialize(&data).map_err(|_| CryptoError::Decrypt("invalid message format".into()))
    }
}

impl Message {
    /// Create a new text message
    pub fn text(room_id: String, sender_id: String, sender_name: String, text: String) -> Self {
        Self {
            id: generate_message_id(),
            room_id,
            sender_id,
            sender_name,
            content: MessageContent::Text(text),
            timestamp: now(),
            signature: Vec::new(),
            encrypted: false,
            expires_at: None,
        }
    }

    /// Create a reaction message
    pub fn reaction(room_id: String, sender_id: String, sender_name: String, target_id: MessageId, emoji: String) -> Self {
        Self {
            id: generate_message_id(),
            room_id,
            sender_id,
            sender_name,
            content: MessageContent::Reaction { target_id, emoji },
            timestamp: now(),
            signature: Vec::new(),
            encrypted: false,
            expires_at: None,
        }
    }

    /// Create a file message
    pub fn file(room_id: String, sender_id: String, sender_name: String, name: String, size: u64, ticket: String) -> Self {
        Self {
            id: generate_message_id(),
            room_id,
            sender_id,
            sender_name,
            content: MessageContent::File { name, size, ticket },
            timestamp: now(),
            signature: Vec::new(),
            encrypted: false,
            expires_at: None,
        }
    }

    /// Sign the message with the sender's keypair
    pub fn sign(&mut self, keypair: &HybridKeypair) {
        let data = self.signable_bytes();
        self.signature = keypair.sign(&data);
    }

    /// Verify the message signature
    pub fn verify_signature(&self, sender_bundle: &PublicBundle) -> Result<(), CryptoError> {
        if self.signature.is_empty() {
            return Err(CryptoError::SignatureInvalid);
        }
        let data = self.signable_bytes();
        sender_bundle.verify(&data, &self.signature)
    }

    /// Get bytes to sign (excludes signature field)
    fn signable_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.id.as_bytes());
        data.extend_from_slice(self.room_id.as_bytes());
        data.extend_from_slice(self.sender_id.as_bytes());
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        if let Ok(content_bytes) = bincode::serialize(&self.content) {
            data.extend_from_slice(&content_bytes);
        }
        data
    }

    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, CryptoError> {
        bincode::serialize(self).map_err(|_| CryptoError::Encrypt)
    }

    /// Deserialize message from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, CryptoError> {
        bincode::deserialize(data).map_err(|_| CryptoError::Decrypt("invalid message format".into()))
    }

    /// Set disappearing message expiration
    pub fn set_expires_in(&mut self, seconds: u64) {
        self.expires_at = Some(self.timestamp + seconds);
    }

    /// Check if message has expired
    pub fn is_expired(&self, current_time: u64) -> bool {
        self.expires_at.map(|exp| current_time > exp).unwrap_or(false)
    }

    /// Validate message fields are complete
    pub fn is_valid(&self) -> bool {
        !self.id.is_empty() 
            && !self.room_id.is_empty() 
            && !self.sender_id.is_empty() 
            && self.timestamp > 0
    }

    /// Validate reaction message structure
    pub fn validate_reaction(&self) -> bool {
        if let MessageContent::Reaction { target_id, emoji } = &self.content {
            !target_id.is_empty() && !emoji.is_empty()
        } else {
            true  // Not a reaction, so valid
        }
    }
}

/// Typing indicator
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypingEvent {
    pub room_id: String,
    pub user_id: String,
    pub typing: bool,
}

/// Presence status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Presence {
    Online,
    Away,
    Busy,
    Offline,
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn generate_message_id() -> MessageId {
    format!("msg_{}", uuid::Uuid::new_v4())
}

pub fn generate_room_id() -> String {
    format!("room_{}", uuid::Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn mock_member(id: &str, name: &str) -> RoomMember {
        RoomMember {
            id: id.into(),
            name: name.into(),
            public_bundle: None,
            online: false,
            typing: false,
        }
    }

    #[test]
    fn test_dm_room_member_count() {
        // Property 7: DM Room Member Count
        let self_member = mock_member("self", "Me");
        let peer = mock_member("peer", "Alice");
        let room = ChatRoom::new_dm("dm1".into(), self_member, peer);
        
        assert!(!room.is_group);
        assert_eq!(room.member_count(), 2);
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::text(
            "room1".into(),
            "sender1".into(),
            "Alice".into(),
            "Hello!".into(),
        );
        
        assert!(!msg.id.is_empty());
        assert_eq!(msg.room_id, "room1");
        assert!(msg.is_valid());
    }

    #[test]
    fn test_message_signing() {
        let keypair = HybridKeypair::generate();
        let mut msg = Message::text(
            "room1".into(),
            "sender1".into(),
            "Alice".into(),
            "Hello!".into(),
        );
        
        msg.sign(&keypair);
        assert!(!msg.signature.is_empty());
        
        let bundle = keypair.public_bundle();
        assert!(msg.verify_signature(&bundle).is_ok());
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::text(
            "room1".into(),
            "sender1".into(),
            "Alice".into(),
            "Hello!".into(),
        );
        
        let bytes = msg.to_bytes().unwrap();
        let restored = Message::from_bytes(&bytes).unwrap();
        
        assert_eq!(msg.id, restored.id);
        assert_eq!(msg.content, restored.content);
    }

    #[test]
    fn test_reaction_message() {
        let msg = Message::reaction(
            "room1".into(),
            "sender1".into(),
            "Alice".into(),
            "msg_target".into(),
            "👍".into(),
        );
        
        assert!(msg.validate_reaction());
        
        if let MessageContent::Reaction { target_id, emoji } = &msg.content {
            assert_eq!(target_id, "msg_target");
            assert_eq!(emoji, "👍");
        } else {
            panic!("Expected reaction content");
        }
    }

    #[test]
    fn test_disappearing_message() {
        let mut msg = Message::text(
            "room1".into(),
            "sender1".into(),
            "Alice".into(),
            "Secret".into(),
        );
        
        msg.set_expires_in(3600);  // 1 hour
        
        assert!(!msg.is_expired(msg.timestamp));
        assert!(!msg.is_expired(msg.timestamp + 3599));
        assert!(msg.is_expired(msg.timestamp + 3601));
    }

    #[test]
    fn test_encrypted_message_roundtrip() {
        let keypair = HybridKeypair::generate();
        let mut msg = Message::text(
            "room1".into(),
            "sender1".into(),
            "Alice".into(),
            "Secret message".into(),
        );
        msg.sign(&keypair);
        
        let encrypted = EncryptedMessage::encrypt(&msg, &keypair.public_bundle()).unwrap();
        let decrypted = encrypted.decrypt(&keypair).unwrap();
        
        assert_eq!(msg.id, decrypted.id);
        assert_eq!(msg.content, decrypted.content);
        assert_eq!(msg.signature, decrypted.signature);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 5: Message Field Completeness
        #[test]
        fn prop_message_field_completeness(
            room_id in "[a-z0-9]{8}",
            sender_id in "[a-z0-9]{8}",
            sender_name in "[A-Za-z ]{1,20}",
            text in ".{1,100}",
        ) {
            let keypair = HybridKeypair::generate();
            let mut msg = Message::text(room_id, sender_id, sender_name, text);
            msg.sign(&keypair);
            
            prop_assert!(!msg.id.is_empty());
            prop_assert!(!msg.room_id.is_empty());
            prop_assert!(!msg.sender_id.is_empty());
            prop_assert!(msg.timestamp > 0);
            prop_assert!(!msg.signature.is_empty());
        }

        /// Property 6: Message Serialization Round-Trip
        #[test]
        fn prop_message_serialization_roundtrip(
            room_id in "[a-z0-9]{8}",
            sender_id in "[a-z0-9]{8}",
            text in ".{1,100}",
        ) {
            let msg = Message::text(room_id, sender_id, "Sender".into(), text);
            let bytes = msg.to_bytes().unwrap();
            let restored = Message::from_bytes(&bytes).unwrap();
            
            prop_assert_eq!(msg.id, restored.id);
            prop_assert_eq!(msg.room_id, restored.room_id);
            prop_assert_eq!(msg.sender_id, restored.sender_id);
            prop_assert_eq!(msg.content, restored.content);
        }

        /// Property 7: DM Room Member Count
        #[test]
        fn prop_dm_room_member_count(
            room_id in "[a-z0-9]{8}",
            self_id in "[a-z0-9]{8}",
            peer_id in "[a-z0-9]{8}",
        ) {
            let self_member = mock_member(&self_id, "Me");
            let peer = mock_member(&peer_id, "Peer");
            let room = ChatRoom::new_dm(room_id, self_member, peer);
            
            prop_assert!(!room.is_group);
            prop_assert_eq!(room.member_count(), 2);
        }

        /// Property 9: Reaction Message Structure
        #[test]
        fn prop_reaction_message_structure(
            room_id in "[a-z0-9]{8}",
            sender_id in "[a-z0-9]{8}",
            target_id in "[a-z0-9]{8}",
            emoji in "[😀-🙏]{1}",
        ) {
            let msg = Message::reaction(
                room_id,
                sender_id,
                "Sender".into(),
                target_id.clone(),
                emoji.clone(),
            );
            
            prop_assert!(msg.validate_reaction());
            
            if let MessageContent::Reaction { target_id: tid, emoji: e } = &msg.content {
                prop_assert!(!tid.is_empty());
                prop_assert!(!e.is_empty());
            }
        }

        /// Property 34: Disappearing Message Deletion
        #[test]
        fn prop_disappearing_message_deletion(
            duration in 1u64..86400,
            elapsed in 0u64..172800,
        ) {
            let mut msg = Message::text(
                "room".into(),
                "sender".into(),
                "Sender".into(),
                "Secret".into(),
            );
            msg.set_expires_in(duration);
            
            let current_time = msg.timestamp + elapsed;
            let should_be_expired = elapsed > duration;
            
            prop_assert_eq!(msg.is_expired(current_time), should_be_expired);
        }
    }
}
