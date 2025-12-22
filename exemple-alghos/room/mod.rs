//! Watch Party - Synchronized media playback
use serde::{Deserialize, Serialize};

/// Watch party room
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchParty {
    pub id: String,
    pub name: String,
    pub host_id: String,
    pub media: Option<MediaInfo>,
    pub state: PlaybackState,
    pub participants: Vec<Participant>,
    pub chat_enabled: bool,
    pub sync_tolerance_ms: u32,  // Adaptive sync tolerance
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaInfo {
    pub name: String,
    pub duration: f64,
    pub ticket: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlaybackState {
    pub playing: bool,
    pub position: f64,
    pub speed: f32,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub ready: bool,
    pub buffering: bool,
    pub latency_ms: u32,
}

impl Participant {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            ready: false,
            buffering: false,
            latency_ms: 0,
        }
    }
}

impl WatchParty {
    /// Create a new watch party with host assignment
    pub fn new(name: String, host_id: String) -> Self {
        Self {
            id: format!("party_{}", uuid::Uuid::new_v4()),
            name,
            host_id,
            media: None,
            state: PlaybackState::default(),
            participants: Vec::new(),
            chat_enabled: true,
            sync_tolerance_ms: 100,  // Default 100ms tolerance
        }
    }

    /// Get the host ID (Property 21: Party Host Assignment)
    pub fn get_host_id(&self) -> &str {
        &self.host_id
    }

    /// Check if a participant is the host
    pub fn is_host(&self, participant_id: &str) -> bool {
        self.host_id == participant_id
    }

    pub fn set_media(&mut self, media: MediaInfo) {
        self.media = Some(media);
        self.state = PlaybackState::default();
        // Reset all participants to not ready when media changes
        for p in &mut self.participants {
            p.ready = false;
        }
    }

    pub fn play(&mut self) {
        self.state.playing = true;
        self.state.updated_at = now();
    }

    pub fn pause(&mut self) {
        self.state.playing = false;
        self.state.updated_at = now();
    }

    pub fn seek(&mut self, position: f64) {
        self.state.position = position;
        self.state.updated_at = now();
        // Reset ready state on seek
        for p in &mut self.participants {
            p.ready = false;
        }
    }

    pub fn add_participant(&mut self, p: Participant) {
        if !self.participants.iter().any(|x| x.id == p.id) {
            self.participants.push(p);
        }
    }

    pub fn remove_participant(&mut self, id: &str) {
        self.participants.retain(|p| p.id != id);
    }

    /// Get participant by ID
    pub fn get_participant(&self, id: &str) -> Option<&Participant> {
        self.participants.iter().find(|p| p.id == id)
    }

    /// Get mutable participant by ID
    pub fn get_participant_mut(&mut self, id: &str) -> Option<&mut Participant> {
        self.participants.iter_mut().find(|p| p.id == id)
    }

    /// Set participant ready status
    pub fn set_participant_ready(&mut self, id: &str, ready: bool) {
        if let Some(p) = self.get_participant_mut(id) {
            p.ready = ready;
        }
    }

    /// Set participant buffering status
    pub fn set_participant_buffering(&mut self, id: &str, buffering: bool) {
        if let Some(p) = self.get_participant_mut(id) {
            p.buffering = buffering;
        }
    }

    /// Update participant latency
    pub fn update_latency(&mut self, id: &str, latency_ms: u32) {
        if let Some(p) = self.get_participant_mut(id) {
            p.latency_ms = latency_ms;
        }
        // Adjust sync tolerance based on max latency
        self.update_sync_tolerance();
    }

    /// Update sync tolerance based on participant latencies
    fn update_sync_tolerance(&mut self) {
        let max_latency = self.participants.iter()
            .map(|p| p.latency_ms)
            .max()
            .unwrap_or(100);
        // Set tolerance to 2x max latency, minimum 100ms
        self.sync_tolerance_ms = (max_latency * 2).max(100);
    }

    /// Check if all participants are ready and not buffering
    pub fn all_ready(&self) -> bool {
        !self.participants.is_empty() && 
        self.participants.iter().all(|p| p.ready && !p.buffering)
    }

    /// Get count of participants
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// Check if participant exists
    pub fn has_participant(&self, id: &str) -> bool {
        self.participants.iter().any(|p| p.id == id)
    }
}

/// Sync event for watch party
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PartyEvent {
    Play,
    Pause,
    Seek { position: f64 },
    SetSpeed { speed: f32 },
    MediaChanged { media: MediaInfo },
    ParticipantJoined { participant: Participant },
    ParticipantLeft { id: String },
    ParticipantReady { id: String, ready: bool },
    ParticipantBuffering { id: String, buffering: bool },
    ChatMessage { sender: String, text: String },
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_party_host_assignment() {
        // Property 21: Party Host Assignment
        let party = WatchParty::new("Movie Night".into(), "host123".into());
        assert_eq!(party.get_host_id(), "host123");
        assert!(party.is_host("host123"));
        assert!(!party.is_host("other"));
    }

    #[test]
    fn test_participant_addition() {
        // Property 22: Participant Addition
        let mut party = WatchParty::new("Movie Night".into(), "host".into());
        
        let p1 = Participant::new("p1".into(), "Alice".into());
        let p2 = Participant::new("p2".into(), "Bob".into());
        
        party.add_participant(p1);
        party.add_participant(p2);
        
        assert_eq!(party.participant_count(), 2);
        assert!(party.has_participant("p1"));
        assert!(party.has_participant("p2"));
    }

    #[test]
    fn test_participant_removal() {
        let mut party = WatchParty::new("Movie Night".into(), "host".into());
        
        party.add_participant(Participant::new("p1".into(), "Alice".into()));
        party.add_participant(Participant::new("p2".into(), "Bob".into()));
        
        party.remove_participant("p1");
        
        assert_eq!(party.participant_count(), 1);
        assert!(!party.has_participant("p1"));
        assert!(party.has_participant("p2"));
    }

    #[test]
    fn test_all_ready_check() {
        // Property 23: All Ready Check
        let mut party = WatchParty::new("Movie Night".into(), "host".into());
        
        let mut p1 = Participant::new("p1".into(), "Alice".into());
        p1.ready = true;
        p1.buffering = false;
        
        let mut p2 = Participant::new("p2".into(), "Bob".into());
        p2.ready = true;
        p2.buffering = false;
        
        party.add_participant(p1);
        party.add_participant(p2);
        
        assert!(party.all_ready());
        
        // One participant buffering
        party.set_participant_buffering("p1", true);
        assert!(!party.all_ready());
        
        // Reset buffering, set not ready
        party.set_participant_buffering("p1", false);
        party.set_participant_ready("p2", false);
        assert!(!party.all_ready());
    }

    #[test]
    fn test_empty_party_not_ready() {
        let party = WatchParty::new("Movie Night".into(), "host".into());
        assert!(!party.all_ready());  // Empty party is not ready
    }

    #[test]
    fn test_playback_controls() {
        let mut party = WatchParty::new("Movie Night".into(), "host".into());
        
        party.play();
        assert!(party.state.playing);
        
        party.pause();
        assert!(!party.state.playing);
        
        party.seek(120.5);
        assert_eq!(party.state.position, 120.5);
    }

    #[test]
    fn test_latency_updates_tolerance() {
        let mut party = WatchParty::new("Movie Night".into(), "host".into());
        
        party.add_participant(Participant::new("p1".into(), "Alice".into()));
        party.add_participant(Participant::new("p2".into(), "Bob".into()));
        
        party.update_latency("p1", 50);
        party.update_latency("p2", 200);
        
        // Tolerance should be 2x max latency = 400ms
        assert_eq!(party.sync_tolerance_ms, 400);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 21: Party Host Assignment
        #[test]
        fn prop_party_host_assignment(
            name in "[A-Za-z ]{1,20}",
            host_id in "[a-z0-9]{8}",
        ) {
            let party = WatchParty::new(name, host_id.clone());
            prop_assert_eq!(party.get_host_id(), &host_id);
            prop_assert!(party.is_host(&host_id));
        }

        /// Property 22: Participant Addition
        #[test]
        fn prop_participant_addition(
            host_id in "[a-z0-9]{8}",
            participant_ids in prop::collection::vec("[a-z0-9]{8}", 1..10),
        ) {
            let mut party = WatchParty::new("Party".into(), host_id);
            
            for id in &participant_ids {
                party.add_participant(Participant::new(id.clone(), "Name".into()));
            }
            
            // All participants should be present
            for id in &participant_ids {
                prop_assert!(party.has_participant(id));
            }
        }

        /// Property 23: All Ready Check
        #[test]
        fn prop_all_ready_check(
            num_participants in 1usize..10,
            ready_flags in prop::collection::vec(any::<bool>(), 1..10),
            buffering_flags in prop::collection::vec(any::<bool>(), 1..10),
        ) {
            let mut party = WatchParty::new("Party".into(), "host".into());
            
            let num = num_participants.min(ready_flags.len()).min(buffering_flags.len());
            
            for i in 0..num {
                let mut p = Participant::new(format!("p{}", i), "Name".into());
                p.ready = ready_flags[i];
                p.buffering = buffering_flags[i];
                party.add_participant(p);
            }
            
            let expected_all_ready = (0..num).all(|i| ready_flags[i] && !buffering_flags[i]);
            prop_assert_eq!(party.all_ready(), expected_all_ready);
        }
    }
}
