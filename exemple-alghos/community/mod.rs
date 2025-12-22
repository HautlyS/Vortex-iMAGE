//! Community module - Discord-like communities over P2P
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Community {
    pub id: String,
    pub name: String,
    pub description: String,
    pub avatar_ticket: Option<String>,
    pub owner_id: String,
    pub created_at: u64,
    pub channels: Vec<Channel>,
    pub members: Vec<Member>,
    pub roles: Vec<Role>,
    pub invite_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub channel_type: ChannelType,
    pub community_id: String,
    pub position: u32,
    pub topic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChannelType {
    Text,
    Voice,
    Video,
    Announcements,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Member {
    pub peer_id: String,
    pub display_name: String,
    pub avatar_ticket: Option<String>,
    pub role_ids: Vec<String>,
    pub joined_at: u64,
    pub presence: MemberPresence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemberPresence {
    Online,
    Away,
    Busy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub color: String,
    pub permissions: Permissions,
    pub position: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Permissions {
    pub admin: bool,
    pub manage_channels: bool,
    pub manage_members: bool,
    pub send_messages: bool,
    pub read_messages: bool,
    pub voice_connect: bool,
    pub video_connect: bool,
}

impl Community {
    pub fn new(name: String, owner_id: String) -> Self {
        let id = format!("com_{}", rand::random::<u32>());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let default_role = Role {
            id: "everyone".into(),
            name: "@everyone".into(),
            color: "#99aab5".into(),
            permissions: Permissions {
                send_messages: true,
                read_messages: true,
                voice_connect: true,
                ..Default::default()
            },
            position: 0,
        };
        
        let general = Channel {
            id: format!("ch_{}", rand::random::<u32>()),
            name: "general".into(),
            channel_type: ChannelType::Text,
            community_id: id.clone(),
            position: 0,
            topic: Some("General discussion".into()),
        };
        
        let voice = Channel {
            id: format!("ch_{}", rand::random::<u32>()),
            name: "Voice".into(),
            channel_type: ChannelType::Voice,
            community_id: id.clone(),
            position: 1,
            topic: None,
        };
        
        Self {
            id,
            name,
            description: String::new(),
            avatar_ticket: None,
            owner_id: owner_id.clone(),
            created_at: now,
            channels: vec![general, voice],
            members: vec![],
            roles: vec![default_role],
            invite_code: None,
        }
    }
    
    pub fn add_channel(&mut self, name: String, channel_type: ChannelType) -> &Channel {
        let ch = Channel {
            id: format!("ch_{}", rand::random::<u32>()),
            name,
            channel_type,
            community_id: self.id.clone(),
            position: self.channels.len() as u32,
            topic: None,
        };
        self.channels.push(ch);
        self.channels.last().unwrap()
    }
    
    pub fn add_member(&mut self, peer_id: String, display_name: String) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.members.push(Member {
            peer_id,
            display_name,
            avatar_ticket: None,
            role_ids: vec!["everyone".into()],
            joined_at: now,
            presence: MemberPresence::Online,
        });
    }
    
    pub fn generate_invite(&mut self) -> String {
        let code = format!("vx-{}", rand::random::<u32>());
        self.invite_code = Some(code.clone());
        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_community() {
        let com = Community::new("Test".into(), "peer_123".into());
        assert_eq!(com.name, "Test");
        assert_eq!(com.channels.len(), 2);
        assert_eq!(com.roles.len(), 1);
    }
}
