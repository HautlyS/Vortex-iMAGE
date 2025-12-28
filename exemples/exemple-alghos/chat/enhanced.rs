//! Enhanced Chat Module - Revolt-style chat architecture
//! Implements Requirements 2.1-2.10 from examples-integration-analysis spec

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User identifier (32 bytes)
pub type UserId = [u8; 32];

/// Channel identifier (32 bytes)
pub type ChannelId = [u8; 32];

/// Server identifier (32 bytes)
pub type ServerId = [u8; 32];

/// Role identifier (16 bytes)
pub type RoleId = [u8; 16];

/// Message identifier (32 bytes)
pub type EnhancedMessageId = [u8; 32];

/// Channel types supported by the system (Req 2.4)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChannelType {
    SavedMessages { owner: UserId },
    DirectMessage { participants: [UserId; 2] },
    Group { owner: UserId, members: Vec<UserId> },
    ServerChannel { server_id: ServerId, roles: Vec<RoleId> },
}

/// Permission value with bitflags (Req 2.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PermissionValue(pub u64);

impl PermissionValue {
    pub const SEND_MESSAGES: u64 = 1 << 0;
    pub const READ_MESSAGES: u64 = 1 << 1;
    pub const MANAGE_MESSAGES: u64 = 1 << 2;
    pub const MANAGE_CHANNEL: u64 = 1 << 3;
    pub const KICK_MEMBERS: u64 = 1 << 4;
    pub const BAN_MEMBERS: u64 = 1 << 5;
    pub const ADMIN: u64 = 1 << 6;
    
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    
    pub fn has(&self, permission: u64) -> bool {
        self.0 & permission == permission
    }
    
    pub fn grant(&mut self, permission: u64) {
        self.0 |= permission;
    }
    
    pub fn revoke(&mut self, permission: u64) {
        self.0 &= !permission;
    }
    
    pub fn merge(&self, other: &PermissionValue) -> PermissionValue {
        PermissionValue(self.0 | other.0)
    }
}

/// Presence session (Req 2.2)
#[derive(Debug, Clone)]
pub struct PresenceSession {
    pub session_id: u32,
    pub user_id: UserId,
    pub connected_at: Instant,
    pub flags: u8,
}

/// Presence manager for tracking online users (Req 2.2, 2.3)
pub struct PresenceManager {
    sessions: HashMap<UserId, Vec<PresenceSession>>,
    online_users: HashSet<UserId>,
    next_session_id: u32,
}

impl PresenceManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            online_users: HashSet::new(),
            next_session_id: 1,
        }
    }
    
    /// Create new presence session (Req 2.2)
    /// Returns (is_first_session, session_id)
    pub fn create_session(&mut self, user_id: UserId, flags: u8) -> (bool, u32) {
        let session_id = self.next_session_id;
        self.next_session_id = self.next_session_id.wrapping_add(1);
        
        let session = PresenceSession {
            session_id,
            user_id,
            connected_at: Instant::now(),
            flags,
        };
        
        let sessions = self.sessions.entry(user_id).or_insert_with(Vec::new);
        let is_first = sessions.is_empty();
        sessions.push(session);
        
        if is_first {
            self.online_users.insert(user_id);
        }
        
        (is_first, session_id)
    }
    
    /// Remove presence session (Req 2.3)
    /// Returns true if this was the last session (user went offline)
    pub fn remove_session(&mut self, user_id: UserId, session_id: u32) -> bool {
        if let Some(sessions) = self.sessions.get_mut(&user_id) {
            sessions.retain(|s| s.session_id != session_id);
            
            if sessions.is_empty() {
                self.sessions.remove(&user_id);
                self.online_users.remove(&user_id);
                return true;
            }
        }
        false
    }
    
    /// Check if user is online
    pub fn is_online(&self, user_id: &UserId) -> bool {
        self.online_users.contains(user_id)
    }
    
    /// Get all online users
    pub fn get_online_users(&self) -> &HashSet<UserId> {
        &self.online_users
    }
    
    /// Get session count for user
    pub fn session_count(&self, user_id: &UserId) -> usize {
        self.sessions.get(user_id).map(|s| s.len()).unwrap_or(0)
    }
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new()
    }
}


/// Typing indicator with automatic expiry (Req 2.7)
#[derive(Debug, Clone)]
pub struct TypingIndicator {
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub started_at: Instant,
    pub expires_at: Instant,
}

impl TypingIndicator {
    /// Create new typing indicator with 5-second expiry
    pub fn new(user_id: UserId, channel_id: ChannelId) -> Self {
        let now = Instant::now();
        Self {
            user_id,
            channel_id,
            started_at: now,
            expires_at: now + Duration::from_secs(5),
        }
    }
    
    /// Check if indicator has expired
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
    
    /// Renew the indicator (reset 5s expiry)
    pub fn renew(&mut self) {
        self.expires_at = Instant::now() + Duration::from_secs(5);
    }
}

/// Typing manager for tracking typing indicators (Req 2.7)
pub struct TypingManager {
    indicators: HashMap<(UserId, ChannelId), TypingIndicator>,
}

impl TypingManager {
    pub fn new() -> Self {
        Self {
            indicators: HashMap::new(),
        }
    }
    
    /// Start typing indicator (Req 2.7)
    pub fn start_typing(&mut self, user_id: UserId, channel_id: ChannelId) -> TypingIndicator {
        let indicator = TypingIndicator::new(user_id, channel_id);
        self.indicators.insert((user_id, channel_id), indicator.clone());
        indicator
    }
    
    /// Renew typing indicator (resets 5s expiry)
    pub fn renew_typing(&mut self, user_id: &UserId, channel_id: &ChannelId) -> bool {
        if let Some(indicator) = self.indicators.get_mut(&(*user_id, *channel_id)) {
            indicator.renew();
            true
        } else {
            false
        }
    }
    
    /// Stop typing
    pub fn stop_typing(&mut self, user_id: &UserId, channel_id: &ChannelId) {
        self.indicators.remove(&(*user_id, *channel_id));
    }
    
    /// Get active typing indicators for channel
    pub fn get_typing(&self, channel_id: &ChannelId) -> Vec<&TypingIndicator> {
        self.indicators.values()
            .filter(|i| &i.channel_id == channel_id && !i.is_expired())
            .collect()
    }
    
    /// Clean up expired indicators
    pub fn cleanup_expired(&mut self) -> Vec<TypingIndicator> {
        let expired: Vec<_> = self.indicators.iter()
            .filter(|(_, i)| i.is_expired())
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        
        let mut removed = Vec::new();
        for (key, indicator) in expired {
            self.indicators.remove(&key);
            removed.push(indicator);
        }
        removed
    }
}

impl Default for TypingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Embed content for rich messages (Req 2.5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embed {
    pub embed_type: EmbedType,
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub color: Option<u32>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbedType {
    Link,
    Image,
    Video,
    Rich,
}

/// Masquerade for custom name/avatar (Req 2.5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Masquerade {
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

/// Enhanced chat message with full feature support (Req 2.5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedChatMessage {
    pub id: EnhancedMessageId,
    pub channel_id: ChannelId,
    pub author_id: UserId,
    pub content: String,
    pub reply_to: Option<EnhancedMessageId>,
    pub reactions: HashMap<String, Vec<UserId>>,
    pub embeds: Vec<Embed>,
    pub masquerade: Option<Masquerade>,
    pub timestamp: DateTime<Utc>,
}

impl EnhancedChatMessage {
    /// Create new message
    pub fn new(channel_id: ChannelId, author_id: UserId, content: String) -> Self {
        let mut id = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut id);
        
        Self {
            id,
            channel_id,
            author_id,
            content,
            reply_to: None,
            reactions: HashMap::new(),
            embeds: Vec::new(),
            masquerade: None,
            timestamp: Utc::now(),
        }
    }
    
    /// Set reply reference
    pub fn with_reply(mut self, reply_to: EnhancedMessageId) -> Self {
        self.reply_to = Some(reply_to);
        self
    }
    
    /// Add embed
    pub fn with_embed(mut self, embed: Embed) -> Self {
        self.embeds.push(embed);
        self
    }
    
    /// Set masquerade
    pub fn with_masquerade(mut self, masquerade: Masquerade) -> Self {
        self.masquerade = Some(masquerade);
        self
    }
    
    /// Add reaction
    pub fn add_reaction(&mut self, emoji: String, user_id: UserId) {
        self.reactions.entry(emoji).or_insert_with(Vec::new).push(user_id);
    }
    
    /// Remove reaction
    pub fn remove_reaction(&mut self, emoji: &str, user_id: &UserId) {
        if let Some(users) = self.reactions.get_mut(emoji) {
            users.retain(|u| u != user_id);
            if users.is_empty() {
                self.reactions.remove(emoji);
            }
        }
    }
}

/// Pagination cursor for message history (Req 2.10)
#[derive(Debug, Clone)]
pub struct PaginationCursor {
    pub before: Option<EnhancedMessageId>,
    pub after: Option<EnhancedMessageId>,
    pub limit: usize,
}

impl Default for PaginationCursor {
    fn default() -> Self {
        Self {
            before: None,
            after: None,
            limit: 50,
        }
    }
}

/// Paginated message result (Req 2.10)
#[derive(Debug, Clone)]
pub struct PaginatedMessages {
    pub messages: Vec<EnhancedChatMessage>,
    pub has_more: bool,
    pub next_cursor: Option<PaginationCursor>,
}

/// Permission configuration for a channel
#[derive(Debug, Clone, Default)]
pub struct ChannelPermissions {
    pub user_base: HashMap<UserId, PermissionValue>,
    pub server_defaults: PermissionValue,
    pub role_overrides: HashMap<RoleId, PermissionValue>,
    pub channel_overrides: HashMap<UserId, PermissionValue>,
}

/// Permission query trait (Req 2.1)
pub trait PermissionQuery {
    /// Calculate effective permissions for user in channel (Req 2.1)
    /// Order: user base → server defaults → role overrides → channel-specific overrides
    fn calculate_permissions(
        &self,
        user_id: &UserId,
        user_roles: &[RoleId],
        channel_perms: &ChannelPermissions,
    ) -> PermissionValue;
}

/// Default permission calculator
pub struct DefaultPermissionCalculator;

impl PermissionQuery for DefaultPermissionCalculator {
    fn calculate_permissions(
        &self,
        user_id: &UserId,
        user_roles: &[RoleId],
        channel_perms: &ChannelPermissions,
    ) -> PermissionValue {
        // Start with user base permissions
        let mut perms = channel_perms.user_base.get(user_id)
            .copied()
            .unwrap_or_default();
        
        // Apply server defaults
        perms = perms.merge(&channel_perms.server_defaults);
        
        // Apply role overrides
        for role_id in user_roles {
            if let Some(role_perms) = channel_perms.role_overrides.get(role_id) {
                perms = perms.merge(role_perms);
            }
        }
        
        // Apply channel-specific overrides (highest priority)
        if let Some(channel_override) = channel_perms.channel_overrides.get(user_id) {
            perms = perms.merge(channel_override);
        }
        
        perms
    }
}

/// Message history store with pagination support (Req 2.10)
pub struct MessageHistory {
    messages: Vec<EnhancedChatMessage>,
}

impl MessageHistory {
    pub fn new() -> Self {
        Self { messages: Vec::new() }
    }
    
    pub fn add_message(&mut self, message: EnhancedChatMessage) {
        self.messages.push(message);
    }
    
    /// Get paginated message history (Req 2.10)
    pub fn get_messages(&self, cursor: PaginationCursor) -> PaginatedMessages {
        let mut filtered: Vec<_> = self.messages.iter().cloned().collect();
        
        // Filter by cursor
        if let Some(before) = &cursor.before {
            if let Some(idx) = filtered.iter().position(|m| &m.id == before) {
                filtered = filtered[..idx].to_vec();
            }
        }
        
        if let Some(after) = &cursor.after {
            if let Some(idx) = filtered.iter().position(|m| &m.id == after) {
                filtered = filtered[idx + 1..].to_vec();
            }
        }
        
        // Apply limit
        let has_more = filtered.len() > cursor.limit;
        let messages: Vec<_> = filtered.into_iter().take(cursor.limit).collect();
        
        let next_cursor = if has_more {
            messages.last().map(|m| PaginationCursor {
                before: Some(m.id),
                after: None,
                limit: cursor.limit,
            })
        } else {
            None
        };
        
        PaginatedMessages {
            messages,
            has_more,
            next_cursor,
        }
    }
    
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl Default for MessageHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_calculation() {
        let calc = DefaultPermissionCalculator;
        let user_id = [1u8; 32];
        let role_id = [1u8; 16];
        
        let mut channel_perms = ChannelPermissions::default();
        channel_perms.server_defaults = PermissionValue::new(PermissionValue::READ_MESSAGES);
        channel_perms.role_overrides.insert(role_id, PermissionValue::new(PermissionValue::SEND_MESSAGES));
        
        let perms = calc.calculate_permissions(&user_id, &[role_id], &channel_perms);
        
        assert!(perms.has(PermissionValue::READ_MESSAGES));
        assert!(perms.has(PermissionValue::SEND_MESSAGES));
    }

    #[test]
    fn test_permission_idempotence() {
        let calc = DefaultPermissionCalculator;
        let user_id = [1u8; 32];
        let channel_perms = ChannelPermissions::default();
        
        let perms1 = calc.calculate_permissions(&user_id, &[], &channel_perms);
        let perms2 = calc.calculate_permissions(&user_id, &[], &channel_perms);
        
        assert_eq!(perms1, perms2);
    }

    #[test]
    fn test_presence_session() {
        let mut manager = PresenceManager::new();
        let user_id = [1u8; 32];
        
        // First session - user comes online
        let (is_first, session_id) = manager.create_session(user_id, 0);
        assert!(is_first);
        assert!(manager.is_online(&user_id));
        
        // Second session - user still online
        let (is_first2, _) = manager.create_session(user_id, 0);
        assert!(!is_first2);
        assert!(manager.is_online(&user_id));
        
        // Remove first session - user still online
        let went_offline = manager.remove_session(user_id, session_id);
        assert!(!went_offline);
        assert!(manager.is_online(&user_id));
    }

    #[test]
    fn test_presence_offline() {
        let mut manager = PresenceManager::new();
        let user_id = [1u8; 32];
        
        let (_, session_id) = manager.create_session(user_id, 0);
        assert!(manager.is_online(&user_id));
        
        let went_offline = manager.remove_session(user_id, session_id);
        assert!(went_offline);
        assert!(!manager.is_online(&user_id));
    }

    #[test]
    fn test_typing_indicator_expiry() {
        let user_id = [1u8; 32];
        let channel_id = [2u8; 32];
        
        let indicator = TypingIndicator::new(user_id, channel_id);
        assert!(!indicator.is_expired());
        
        // Note: Can't easily test actual expiry without sleeping
    }

    #[test]
    fn test_message_pagination() {
        let mut history = MessageHistory::new();
        let channel_id = [1u8; 32];
        let author_id = [2u8; 32];
        
        // Add 100 messages
        for i in 0..100 {
            let mut msg = EnhancedChatMessage::new(channel_id, author_id, format!("Message {}", i));
            msg.id[0] = i as u8;
            history.add_message(msg);
        }
        
        // Get first page
        let cursor = PaginationCursor { limit: 50, ..Default::default() };
        let result = history.get_messages(cursor);
        
        assert_eq!(result.messages.len(), 50);
        assert!(result.has_more);
        assert!(result.next_cursor.is_some());
    }

    #[test]
    fn test_message_reactions() {
        let channel_id = [1u8; 32];
        let author_id = [2u8; 32];
        let reactor_id = [3u8; 32];
        
        let mut msg = EnhancedChatMessage::new(channel_id, author_id, "Hello".into());
        
        msg.add_reaction("👍".into(), reactor_id);
        assert_eq!(msg.reactions.get("👍").unwrap().len(), 1);
        
        msg.remove_reaction("👍", &reactor_id);
        assert!(msg.reactions.get("👍").is_none());
    }

    #[test]
    fn test_channel_types() {
        let owner = [1u8; 32];
        let participant = [2u8; 32];
        
        let saved = ChannelType::SavedMessages { owner };
        let dm = ChannelType::DirectMessage { participants: [owner, participant] };
        let group = ChannelType::Group { owner, members: vec![participant] };
        
        assert!(matches!(saved, ChannelType::SavedMessages { .. }));
        assert!(matches!(dm, ChannelType::DirectMessage { .. }));
        assert!(matches!(group, ChannelType::Group { .. }));
    }
}
