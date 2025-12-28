//! Watch Party - P2P synchronized video playback
//! Uses iroh-gossip for real-time sync across peers

use bytes::Bytes;
use iroh_gossip::proto::TopicId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Media source type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MediaSource {
    YouTube { video_id: String },
    LocalFile { ticket: String, name: String },
    DirectUrl { url: String },
}

/// Media info for the party
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaInfo {
    pub source: MediaSource,
    pub duration: f64,
    pub title: String,
}

/// Playback state synchronized across peers
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlaybackState {
    pub playing: bool,
    pub position: f64,
    pub speed: f32,
    pub timestamp: u64,
}

impl PlaybackState {
    /// Calculate current position accounting for time elapsed
    pub fn current_position(&self) -> f64 {
        if !self.playing {
            return self.position;
        }
        let now = now_ms();
        let elapsed = (now.saturating_sub(self.timestamp)) as f64 / 1000.0;
        self.position + (elapsed * self.speed as f64)
    }
}

/// Participant in the watch party
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub ready: bool,
    pub buffering: bool,
    pub position: f64,
}

/// Watch party state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchParty {
    pub id: String,
    pub name: String,
    pub host_id: String,
    pub topic_id: [u8; 32],
    pub media: Option<MediaInfo>,
    pub state: PlaybackState,
    pub participants: Vec<Participant>,
    pub chat_enabled: bool,
}

impl WatchParty {
    pub fn new(name: String, host_id: String) -> Self {
        let topic_bytes: [u8; 32] = rand::random();
        Self {
            id: format!("party_{}", &hex::encode(&topic_bytes[..8])),
            name,
            host_id,
            topic_id: topic_bytes,
            media: None,
            state: PlaybackState::default(),
            participants: Vec::new(),
            chat_enabled: true,
        }
    }

    pub fn topic(&self) -> TopicId {
        TopicId::from_bytes(self.topic_id)
    }

    pub fn is_host(&self, peer_id: &str) -> bool {
        self.host_id == peer_id
    }
}

/// Events broadcast over gossip for sync
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PartySync {
    /// Playback control
    Play { position: f64, timestamp: u64 },
    Pause { position: f64 },
    Seek { position: f64, timestamp: u64 },
    
    /// Media change
    SetMedia { media: MediaInfo },
    
    /// Participant updates
    Join { participant: Participant },
    Leave { id: String },
    Ready { id: String, ready: bool },
    Buffering { id: String, buffering: bool },
    PositionReport { id: String, position: f64 },
    
    /// Chat
    Chat { sender_id: String, sender_name: String, text: String, timestamp: u64 },
    
    /// Full state sync (for new joiners)
    FullState { party: WatchParty },
    RequestState,
}

impl PartySync {
    pub fn to_bytes(&self) -> Bytes {
        bincode::serialize(self).unwrap_or_default().into()
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        bincode::deserialize(data).ok()
    }
}

/// Invite ticket for joining a party
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyInvite {
    pub party_id: String,
    pub party_name: String,
    pub topic_id: [u8; 32],
    pub host_node_id: String,
}

impl PartyInvite {
    /// Encode invite to shareable string
    pub fn encode(&self) -> String {
        let bytes = bincode::serialize(self).unwrap_or_default();
        format!("vortex-party:{}", base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &bytes))
    }

    /// Decode invite from string
    pub fn decode(s: &str) -> Option<Self> {
        let data = s.strip_prefix("vortex-party:")?;
        let bytes = base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, data).ok()?;
        bincode::deserialize(&bytes).ok()
    }
}

/// Chat message in party
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub text: String,
    pub timestamp: u64,
}

/// Party manager - handles multiple parties
pub struct PartyManager {
    parties: HashMap<String, WatchParty>,
    active_party: Option<String>,
    my_id: String,
    my_name: String,
    event_tx: mpsc::UnboundedSender<PartyEvent>,
    chat_history: HashMap<String, Vec<ChatMessage>>,
}

/// Events emitted to UI
#[derive(Clone, Debug)]
pub enum PartyEvent {
    StateChanged(WatchParty),
    ChatReceived(ChatMessage),
    ParticipantJoined(Participant),
    ParticipantLeft(String),
    SyncRequired { position: f64 },
}

impl PartyManager {
    pub fn new(my_id: String, my_name: String, event_tx: mpsc::UnboundedSender<PartyEvent>) -> Self {
        Self {
            parties: HashMap::new(),
            active_party: None,
            my_id,
            my_name,
            event_tx,
            chat_history: HashMap::new(),
        }
    }

    /// Create a new watch party (I become host)
    pub fn create_party(&mut self, name: String) -> (WatchParty, PartyInvite) {
        let party = WatchParty::new(name, self.my_id.clone());
        let invite = PartyInvite {
            party_id: party.id.clone(),
            party_name: party.name.clone(),
            topic_id: party.topic_id,
            host_node_id: self.my_id.clone(),
        };
        
        // Add myself as participant
        let mut party = party;
        party.participants.push(Participant {
            id: self.my_id.clone(),
            name: self.my_name.clone(),
            ready: true,
            buffering: false,
            position: 0.0,
        });
        
        self.parties.insert(party.id.clone(), party.clone());
        self.active_party = Some(party.id.clone());
        
        (party, invite)
    }

    /// Join a party via invite
    pub fn join_party(&mut self, invite: &PartyInvite) -> WatchParty {
        let party = WatchParty {
            id: invite.party_id.clone(),
            name: invite.party_name.clone(),
            host_id: invite.host_node_id.clone(),
            topic_id: invite.topic_id,
            media: None,
            state: PlaybackState::default(),
            participants: vec![Participant {
                id: self.my_id.clone(),
                name: self.my_name.clone(),
                ready: false,
                buffering: false,
                position: 0.0,
            }],
            chat_enabled: true,
        };
        
        self.parties.insert(party.id.clone(), party.clone());
        self.active_party = Some(party.id.clone());
        party
    }

    /// Get active party
    pub fn active(&self) -> Option<&WatchParty> {
        self.active_party.as_ref().and_then(|id| self.parties.get(id))
    }

    /// Get active party mutable
    pub fn active_mut(&mut self) -> Option<&mut WatchParty> {
        self.active_party.as_ref().and_then(|id| self.parties.get_mut(id))
    }

    /// Handle incoming sync message
    pub fn handle_sync(&mut self, sync: PartySync) -> Option<PartySync> {
        // Clone values we need before borrowing mutably
        let my_id = self.my_id.clone();
        let event_tx = self.event_tx.clone();
        
        match sync {
            PartySync::Play { position, timestamp } => {
                let party = self.active_mut()?;
                party.state.playing = true;
                party.state.position = position;
                party.state.timestamp = timestamp;
                self.emit_state_changed();
            }
            PartySync::Pause { position } => {
                let party = self.active_mut()?;
                party.state.playing = false;
                party.state.position = position;
                self.emit_state_changed();
            }
            PartySync::Seek { position, timestamp } => {
                self.active_mut()?.state.position = position;
                self.active_mut()?.state.timestamp = timestamp;
                let _ = event_tx.send(PartyEvent::SyncRequired { position });
                self.emit_state_changed();
            }
            PartySync::SetMedia { media } => {
                let party = self.active_mut()?;
                party.media = Some(media);
                party.state = PlaybackState::default();
                self.emit_state_changed();
            }
            PartySync::Join { participant } => {
                let party = self.active_mut()?;
                if !party.participants.iter().any(|p| p.id == participant.id) {
                    let _ = event_tx.send(PartyEvent::ParticipantJoined(participant.clone()));
                    party.participants.push(participant);
                }
                self.emit_state_changed();
            }
            PartySync::Leave { id } => {
                self.active_mut()?.participants.retain(|p| p.id != id);
                let _ = event_tx.send(PartyEvent::ParticipantLeft(id));
                self.emit_state_changed();
            }
            PartySync::Ready { id, ready } => {
                if let Some(p) = self.active_mut()?.participants.iter_mut().find(|p| p.id == id) {
                    p.ready = ready;
                }
                self.emit_state_changed();
            }
            PartySync::Buffering { id, buffering } => {
                if let Some(p) = self.active_mut()?.participants.iter_mut().find(|p| p.id == id) {
                    p.buffering = buffering;
                }
                self.emit_state_changed();
            }
            PartySync::PositionReport { id, position } => {
                let party = self.active_mut()?;
                if let Some(p) = party.participants.iter_mut().find(|p| p.id == id) {
                    p.position = position;
                }
            }
            PartySync::Chat { sender_id, sender_name, text, timestamp } => {
                let party = self.active()?;
                let party_id = party.id.clone();
                let msg = ChatMessage {
                    id: format!("msg_{}", rand::random::<u32>()),
                    sender_id,
                    sender_name,
                    text,
                    timestamp,
                };
                self.chat_history.entry(party_id).or_default().push(msg.clone());
                let _ = event_tx.send(PartyEvent::ChatReceived(msg));
            }
            PartySync::FullState { party: full_party } => {
                // Update our state from host
                if let Some(p) = self.parties.get_mut(&full_party.id) {
                    p.media = full_party.media;
                    p.state = full_party.state;
                    p.participants = full_party.participants;
                }
                self.emit_state_changed();
            }
            PartySync::RequestState => {
                // If we're host, send full state
                let party = self.active()?;
                if party.is_host(&my_id) {
                    return Some(PartySync::FullState { party: party.clone() });
                }
            }
        }
        None
    }

    /// Play - returns sync message to broadcast
    pub fn play(&mut self) -> Option<PartySync> {
        let ts = now_ms();
        let party = self.active_mut()?;
        party.state.playing = true;
        party.state.timestamp = ts;
        let position = party.state.position;
        self.emit_state_changed();
        Some(PartySync::Play { position, timestamp: ts })
    }

    /// Pause - returns sync message to broadcast
    pub fn pause(&mut self) -> Option<PartySync> {
        let party = self.active_mut()?;
        party.state.playing = false;
        party.state.position = party.state.current_position();
        let position = party.state.position;
        self.emit_state_changed();
        Some(PartySync::Pause { position })
    }

    /// Seek - returns sync message to broadcast
    pub fn seek(&mut self, position: f64) -> Option<PartySync> {
        let ts = now_ms();
        let party = self.active_mut()?;
        party.state.position = position;
        party.state.timestamp = ts;
        self.emit_state_changed();
        Some(PartySync::Seek { position, timestamp: ts })
    }

    /// Set media - returns sync message to broadcast
    pub fn set_media(&mut self, media: MediaInfo) -> Option<PartySync> {
        let party = self.active_mut()?;
        party.media = Some(media.clone());
        party.state = PlaybackState::default();
        self.emit_state_changed();
        Some(PartySync::SetMedia { media })
    }

    /// Send chat message - returns sync message to broadcast
    pub fn send_chat(&mut self, text: String) -> Option<PartySync> {
        let _party = self.active()?;
        let msg = PartySync::Chat {
            sender_id: self.my_id.clone(),
            sender_name: self.my_name.clone(),
            text,
            timestamp: now_ms(),
        };
        Some(msg)
    }

    /// Report my buffering state
    pub fn set_buffering(&mut self, buffering: bool) -> Option<PartySync> {
        let my_id = self.my_id.clone();
        let party = self.active_mut()?;
        if let Some(p) = party.participants.iter_mut().find(|p| p.id == my_id) {
            p.buffering = buffering;
        }
        Some(PartySync::Buffering { id: my_id, buffering })
    }

    /// Report my ready state
    pub fn set_ready(&mut self, ready: bool) -> Option<PartySync> {
        let my_id = self.my_id.clone();
        let party = self.active_mut()?;
        if let Some(p) = party.participants.iter_mut().find(|p| p.id == my_id) {
            p.ready = ready;
        }
        Some(PartySync::Ready { id: my_id, ready })
    }

    /// Get chat history for active party
    pub fn chat_history(&self) -> Vec<ChatMessage> {
        self.active_party.as_ref()
            .and_then(|id| self.chat_history.get(id))
            .cloned()
            .unwrap_or_default()
    }

    /// Leave current party
    pub fn leave(&mut self) -> Option<PartySync> {
        let party_id = self.active_party.take()?;
        self.parties.remove(&party_id);
        Some(PartySync::Leave { id: self.my_id.clone() })
    }

    fn emit_state_changed(&self) {
        if let Some(party) = self.active() {
            let _ = self.event_tx.send(PartyEvent::StateChanged(party.clone()));
        }
    }
}

pub type SharedPartyManager = Arc<RwLock<PartyManager>>;

/// Extract YouTube video ID from URL
pub fn extract_youtube_id(url: &str) -> Option<String> {
    // youtube.com/watch?v=VIDEO_ID
    if let Some(pos) = url.find("v=") {
        let start = pos + 2;
        let end = url[start..].find('&').map(|i| start + i).unwrap_or(url.len());
        return Some(url[start..end].to_string());
    }
    // youtu.be/VIDEO_ID
    if url.contains("youtu.be/") {
        let parts: Vec<&str> = url.split("youtu.be/").collect();
        if parts.len() > 1 {
            let id = parts[1].split('?').next().unwrap_or(parts[1]);
            return Some(id.to_string());
        }
    }
    // youtube.com/embed/VIDEO_ID
    if url.contains("/embed/") {
        let parts: Vec<&str> = url.split("/embed/").collect();
        if parts.len() > 1 {
            let id = parts[1].split('?').next().unwrap_or(parts[1]);
            return Some(id.to_string());
        }
    }
    None
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_youtube_id_extraction() {
        assert_eq!(extract_youtube_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ"), Some("dQw4w9WgXcQ".into()));
        assert_eq!(extract_youtube_id("https://youtu.be/dQw4w9WgXcQ"), Some("dQw4w9WgXcQ".into()));
        assert_eq!(extract_youtube_id("https://youtube.com/embed/dQw4w9WgXcQ"), Some("dQw4w9WgXcQ".into()));
        assert_eq!(extract_youtube_id("https://www.youtube.com/watch?v=abc123&t=10"), Some("abc123".into()));
    }

    #[test]
    fn test_invite_encode_decode() {
        let invite = PartyInvite {
            party_id: "party_123".into(),
            party_name: "Movie Night".into(),
            topic_id: [1u8; 32],
            host_node_id: "host_abc".into(),
        };
        let encoded = invite.encode();
        let decoded = PartyInvite::decode(&encoded).unwrap();
        assert_eq!(decoded.party_id, invite.party_id);
        assert_eq!(decoded.party_name, invite.party_name);
    }

    #[test]
    fn test_party_creation() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut manager = PartyManager::new("me".into(), "Alice".into(), tx);
        let (party, invite) = manager.create_party("Movie Night".into());
        
        assert!(party.is_host("me"));
        assert_eq!(party.participants.len(), 1);
        assert_eq!(invite.party_name, "Movie Night");
    }

    #[test]
    fn test_playback_position() {
        let state = PlaybackState {
            playing: false,
            position: 100.0,
            speed: 1.0,
            timestamp: 0,
        };
        assert_eq!(state.current_position(), 100.0);
    }
}
