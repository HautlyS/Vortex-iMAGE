//! Social Module - Unified P2P Social Platform
//! Anonymous DMs, Groups, Communities with iroh-gossip
use serde::{Deserialize, Serialize};
use bytes::Bytes;
use std::collections::HashMap;

// Re-exports
pub use crate::messaging::{Identity, Conversation, ConversationType, Message as ChatMessage, MessageContent, PrivacySettings};
pub use crate::community::{Community, Channel, ChannelType, Member, MemberPresence, Role, Permissions};

/// Topic ID for gossip (32 bytes)
pub type TopicId = [u8; 32];

/// Generate topic for DM between two peers (deterministic)
pub fn dm_topic(peer_a: &str, peer_b: &str) -> TopicId {
    let (first, second) = if peer_a < peer_b { (peer_a, peer_b) } else { (peer_b, peer_a) };
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"vortex:dm:");
    hasher.update(first.as_bytes());
    hasher.update(b":");
    hasher.update(second.as_bytes());
    *hasher.finalize().as_bytes()
}

/// Generate random topic for groups/communities
pub fn random_topic() -> TopicId {
    rand::random()
}

/// Gossip message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    // Identity
    Announce { identity: Identity },
    
    // Chat
    Chat { msg: ChatMessage },
    Typing { conv_id: String, peer_id: String },
    Read { conv_id: String, peer_id: String, msg_id: String },
    
    // Group management
    GroupInvite { group_id: String, inviter: String, topic: TopicId },
    GroupJoin { group_id: String, member: Identity },
    GroupLeave { group_id: String, peer_id: String },
    
    // Community
    CommunitySync { community: Community },
    ChannelMessage { community_id: String, channel_id: String, msg: ChatMessage },
    MemberUpdate { community_id: String, member: Member },
    
    // Reactions
    React { conv_id: String, msg_id: String, emoji: String, peer_id: String },
    Unreact { conv_id: String, msg_id: String, peer_id: String },
}

impl GossipMessage {
    pub fn to_bytes(&self) -> Bytes {
        bincode::serialize(self).unwrap_or_default().into()
    }
    
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        bincode::deserialize(data).ok()
    }
}

/// Social feed post
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: PostContent,
    pub timestamp: u64,
    pub reactions: Vec<Reaction>,
    pub comments: Vec<Comment>,
    pub encrypted_for: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PostContent {
    Text(String),
    Image { ticket: String, caption: Option<String> },
    Video { ticket: String, caption: Option<String> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reaction {
    pub user_id: String,
    pub emoji: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub timestamp: u64,
}

/// 24-hour story
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Story {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: PostContent,
    pub created_at: u64,
    pub expires_at: u64,
    pub viewers: Vec<String>,
}

impl Story {
    pub fn new(id: String, author_id: String, author_name: String, content: PostContent) -> Self {
        let created_at = now();
        Self { id, author_id, author_name, content, created_at, expires_at: created_at + 86400, viewers: Vec::new() }
    }
    
    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time > self.expires_at
    }
}

/// Social feed
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Feed {
    pub posts: Vec<Post>,
    pub stories: HashMap<String, Vec<Story>>,
}

impl Feed {
    pub fn new() -> Self { Self::default() }
    
    pub fn add_post(&mut self, post: Post) {
        self.posts.push(post);
    }
    
    pub fn add_story(&mut self, story: Story) {
        self.stories.entry(story.author_id.clone()).or_default().push(story);
    }
    
    pub fn get_chronological(&self) -> Vec<&Post> {
        let mut posts: Vec<_> = self.posts.iter().collect();
        posts.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        posts
    }
    
    pub fn remove_expired_stories(&mut self, current_time: u64) {
        for stories in self.stories.values_mut() {
            stories.retain(|s| !s.is_expired(current_time));
        }
        self.stories.retain(|_, stories| !stories.is_empty());
    }
}

impl Post {
    pub fn text(id: String, author_id: String, author_name: String, text: String) -> Self {
        Self {
            id, author_id, author_name,
            content: PostContent::Text(text),
            timestamp: now(),
            reactions: Vec::new(),
            comments: Vec::new(),
            encrypted_for: Vec::new(),
        }
    }
    
    pub fn add_reaction(&mut self, user_id: String, emoji: String) {
        self.reactions.retain(|r| r.user_id != user_id);
        self.reactions.push(Reaction { user_id, emoji, timestamp: now() });
    }
    
    pub fn add_comment(&mut self, id: String, author_id: String, author_name: String, content: String) {
        self.comments.push(Comment { id, author_id, author_name, content, timestamp: now() });
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn generate_post_id() -> String { format!("post_{}", uuid::Uuid::new_v4()) }
pub fn generate_story_id() -> String { format!("story_{}", uuid::Uuid::new_v4()) }

pub mod manager;
pub use manager::{SocialState, InviteCode, TopicSubscription, new_message};
