//! Meetings module - P2P video call scheduling and management
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Meeting status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MeetingStatus {
    Scheduled,
    Live,
    Ended,
}

/// Meeting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    pub scheduled_at: u64,
    pub duration: u32, // minutes
    pub participants: Vec<String>,
    pub room_code: String,
    pub host_id: String,
    pub status: MeetingStatus,
    pub created_at: u64,
}

/// Call signal for WebRTC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSignal {
    pub peer_id: String,
    pub signal_type: SignalType,
    pub sdp: Option<String>,
    pub candidate: Option<IceCandidate>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignalType {
    Offer,
    Answer,
    Candidate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_m_line_index: Option<u16>,
}

/// Meeting manager
#[derive(Debug, Default)]
pub struct MeetingManager {
    meetings: HashMap<String, Meeting>,
    signals: HashMap<String, Vec<CallSignal>>, // room_code -> signals
}

impl MeetingManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new meeting
    pub fn create_meeting(
        &mut self,
        title: String,
        scheduled_at: u64,
        duration: u32,
        host_id: String,
    ) -> Meeting {
        let id = format!("meet_{}", rand::random::<u32>());
        let room_code = format!("vx-{}", &id[5..13]);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let meeting = Meeting {
            id: id.clone(),
            title,
            scheduled_at,
            duration,
            participants: vec![host_id.clone()],
            room_code,
            host_id,
            status: MeetingStatus::Scheduled,
            created_at: now,
        };

        self.meetings.insert(id, meeting.clone());
        meeting
    }

    /// Join a meeting by room code
    pub fn join_meeting(&mut self, room_code: &str, peer_id: &str) -> Option<Meeting> {
        for meeting in self.meetings.values_mut() {
            if meeting.room_code == room_code {
                if !meeting.participants.contains(&peer_id.to_string()) {
                    meeting.participants.push(peer_id.to_string());
                }
                return Some(meeting.clone());
            }
        }
        None
    }

    /// Get all meetings
    pub fn get_meetings(&self) -> Vec<Meeting> {
        self.meetings.values().cloned().collect()
    }

    /// Get meeting by ID
    pub fn get_meeting(&self, id: &str) -> Option<&Meeting> {
        self.meetings.get(id)
    }

    /// Start a meeting
    pub fn start_meeting(&mut self, id: &str) -> Option<Meeting> {
        if let Some(meeting) = self.meetings.get_mut(id) {
            meeting.status = MeetingStatus::Live;
            return Some(meeting.clone());
        }
        None
    }

    /// End a meeting
    pub fn end_meeting(&mut self, id: &str) -> bool {
        if let Some(meeting) = self.meetings.get_mut(id) {
            meeting.status = MeetingStatus::Ended;
            return true;
        }
        false
    }

    /// Delete a meeting
    pub fn delete_meeting(&mut self, id: &str) -> bool {
        self.meetings.remove(id).is_some()
    }

    /// Add a call signal
    pub fn add_signal(&mut self, room_code: &str, signal: CallSignal) {
        self.signals
            .entry(room_code.to_string())
            .or_default()
            .push(signal);
    }

    /// Get signals for a room
    pub fn get_signals(&self, room_code: &str) -> Vec<CallSignal> {
        self.signals.get(room_code).cloned().unwrap_or_default()
    }

    /// Clear signals for a room
    pub fn clear_signals(&mut self, room_code: &str) {
        self.signals.remove(room_code);
    }

    /// Get upcoming meetings
    pub fn get_upcoming(&self) -> Vec<Meeting> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.meetings
            .values()
            .filter(|m| m.status != MeetingStatus::Ended && m.scheduled_at >= now)
            .cloned()
            .collect()
    }

    /// Get live meetings
    pub fn get_live(&self) -> Vec<Meeting> {
        self.meetings
            .values()
            .filter(|m| m.status == MeetingStatus::Live)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_meeting() {
        let mut manager = MeetingManager::new();
        let meeting = manager.create_meeting(
            "Test Meeting".to_string(),
            1700000000,
            30,
            "host_123".to_string(),
        );

        assert_eq!(meeting.title, "Test Meeting");
        assert_eq!(meeting.duration, 30);
        assert_eq!(meeting.status, MeetingStatus::Scheduled);
        assert!(meeting.room_code.starts_with("vx-"));
    }

    #[test]
    fn test_join_meeting() {
        let mut manager = MeetingManager::new();
        let meeting = manager.create_meeting(
            "Test".to_string(),
            1700000000,
            30,
            "host".to_string(),
        );

        let joined = manager.join_meeting(&meeting.room_code, "peer_1").unwrap();
        assert_eq!(joined.participants.len(), 2);
    }

    #[test]
    fn test_meeting_lifecycle() {
        let mut manager = MeetingManager::new();
        let meeting = manager.create_meeting(
            "Test".to_string(),
            1700000000,
            30,
            "host".to_string(),
        );

        let started = manager.start_meeting(&meeting.id).unwrap();
        assert_eq!(started.status, MeetingStatus::Live);

        assert!(manager.end_meeting(&meeting.id));
        let ended = manager.get_meeting(&meeting.id).unwrap();
        assert_eq!(ended.status, MeetingStatus::Ended);
    }
}
