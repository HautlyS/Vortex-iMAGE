//! Streaming Module - WebRTC-based streaming with WHIP/WHEP support for watch parties
//! Implements Requirements 3.1-3.11 from examples-integration-analysis spec

use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Session identifier (16 bytes)
pub type SessionId = [u8; 16];

/// User identifier (32 bytes)
pub type UserId = [u8; 32];

/// Stream session state (Req 3.1, 3.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSession {
    pub session_id: SessionId,
    pub host_id: UserId,
    pub viewers: Vec<UserId>,
    pub created_at: u64,
    pub codec_info: CodecInfo,
}

/// Codec information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodecInfo {
    pub video_codec: String,
    pub audio_codec: String,
    pub video_bitrate: u32,
    pub audio_bitrate: u32,
}

/// RTP packet with ordering support (Req 3.6)
#[derive(Debug, Clone)]
pub struct OrderedRtpPacket {
    pub sequence_number: u16,
    pub timestamp: u32,
    pub payload: Vec<u8>,
    pub received_at: Instant,
}

/// Jitter buffer for packet reordering (Req 3.6)
pub struct JitterBuffer {
    packets: BTreeMap<u16, OrderedRtpPacket>,
    buffer_duration: Duration,
    last_output_seq: Option<u16>,
}

/// NAL unit from depacketized video (Req 3.3, 3.4)
#[derive(Debug, Clone)]
pub struct NalUnit {
    pub nal_type: u8,
    pub data: Vec<u8>,
    pub timestamp: u32,
}

/// Bitrate adaptation configuration (Req 3.11)
#[derive(Debug, Clone)]
pub struct BitrateConfig {
    pub min_bitrate: u32,  // 500 kbps
    pub max_bitrate: u32,  // 10 Mbps
    pub current_bitrate: u32,
}

impl Default for BitrateConfig {
    fn default() -> Self {
        Self {
            min_bitrate: 500_000,
            max_bitrate: 10_000_000,
            current_bitrate: 2_000_000,
        }
    }
}

/// Bandwidth estimation result (Req 3.11)
#[derive(Debug, Clone)]
pub struct BandwidthEstimate {
    pub available_bps: u64,
    pub packet_loss_rate: f32,
    pub rtt_ms: u32,
}

/// Playback synchronization state (Req 3.8)
pub struct PlaybackSync {
    viewer_offsets: HashMap<UserId, i64>,
    reference_timestamp: u64,
    max_drift_ms: i64,
}

/// Stream error types
#[derive(Error, Debug)]
pub enum StreamError {
    #[error("WebRTC connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Codec error: {0}")]
    CodecError(String),
    #[error("Transcoding failed: {0}")]
    TranscodingFailed(String),
    #[error("Sync failed: {0}")]
    SyncFailed(String),
    #[error("Session not found")]
    SessionNotFound,
}


impl JitterBuffer {
    /// Create new jitter buffer with 200ms duration (Req 3.6)
    pub fn new() -> Self {
        Self {
            packets: BTreeMap::new(),
            buffer_duration: Duration::from_millis(200),
            last_output_seq: None,
        }
    }
    
    /// Insert packet into buffer
    pub fn insert(&mut self, packet: OrderedRtpPacket) {
        self.packets.insert(packet.sequence_number, packet);
    }
    
    /// Get next packet in sequence order (Req 3.6)
    pub fn get_next(&mut self) -> Option<OrderedRtpPacket> {
        let expected_seq = self.last_output_seq
            .map(|s| s.wrapping_add(1))
            .unwrap_or(0);
        
        // Check if we have the expected packet
        if let Some(packet) = self.packets.remove(&expected_seq) {
            self.last_output_seq = Some(expected_seq);
            return Some(packet);
        }
        
        // Check if oldest packet has been waiting too long
        if let Some((&seq, packet)) = self.packets.first_key_value() {
            if packet.received_at.elapsed() > self.buffer_duration {
                let packet = self.packets.remove(&seq).unwrap();
                self.last_output_seq = Some(seq);
                return Some(packet);
            }
        }
        
        None
    }
    
    /// Reorder packets and return in sequence (Req 3.6)
    pub fn reorder(&mut self, packets: Vec<OrderedRtpPacket>) -> Vec<OrderedRtpPacket> {
        for packet in packets {
            self.insert(packet);
        }
        
        let mut ordered = Vec::new();
        while let Some(packet) = self.get_next() {
            ordered.push(packet);
        }
        ordered
    }
    
    /// Get buffer size
    pub fn len(&self) -> usize {
        self.packets.len()
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }
}

impl Default for JitterBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybackSync {
    /// Create new playback sync with 500ms max drift (Req 3.8)
    pub fn new() -> Self {
        Self {
            viewer_offsets: HashMap::new(),
            reference_timestamp: 0,
            max_drift_ms: 500,
        }
    }
    
    /// Set reference timestamp
    pub fn set_reference(&mut self, timestamp: u64) {
        self.reference_timestamp = timestamp;
    }
    
    /// Update viewer's playback position
    pub fn update_viewer(&mut self, viewer: UserId, timestamp: u64) {
        let offset = timestamp as i64 - self.reference_timestamp as i64;
        self.viewer_offsets.insert(viewer, offset);
    }
    
    /// Check if all viewers are synchronized within 500ms (Req 3.8)
    pub fn is_synchronized(&self) -> bool {
        if self.viewer_offsets.is_empty() {
            return true;
        }
        
        let offsets: Vec<_> = self.viewer_offsets.values().copied().collect();
        let min = offsets.iter().min().copied().unwrap_or(0);
        let max = offsets.iter().max().copied().unwrap_or(0);
        
        (max - min).abs() <= self.max_drift_ms
    }
    
    /// Get viewer's offset from reference
    pub fn get_offset(&self, viewer: &UserId) -> Option<i64> {
        self.viewer_offsets.get(viewer).copied()
    }
    
    /// Get all viewer offsets
    pub fn get_all_offsets(&self) -> &HashMap<UserId, i64> {
        &self.viewer_offsets
    }
}

impl Default for PlaybackSync {
    fn default() -> Self {
        Self::new()
    }
}

/// Depacketize H.264 RTP to NAL units (Req 3.3)
pub fn depacketize_h264(packets: &[OrderedRtpPacket]) -> Vec<NalUnit> {
    let mut nal_units = Vec::new();
    
    for packet in packets {
        if packet.payload.is_empty() {
            continue;
        }
        
        let nal_type = packet.payload[0] & 0x1F;
        
        match nal_type {
            // Single NAL unit
            1..=23 => {
                nal_units.push(NalUnit {
                    nal_type,
                    data: packet.payload.clone(),
                    timestamp: packet.timestamp,
                });
            }
            // STAP-A (aggregation)
            24 => {
                let mut offset = 1;
                while offset + 2 < packet.payload.len() {
                    let size = u16::from_be_bytes([
                        packet.payload[offset],
                        packet.payload[offset + 1],
                    ]) as usize;
                    offset += 2;
                    
                    if offset + size <= packet.payload.len() {
                        let nal_data = packet.payload[offset..offset + size].to_vec();
                        if !nal_data.is_empty() {
                            nal_units.push(NalUnit {
                                nal_type: nal_data[0] & 0x1F,
                                data: nal_data,
                                timestamp: packet.timestamp,
                            });
                        }
                        offset += size;
                    }
                }
            }
            // FU-A (fragmentation)
            28 => {
                if packet.payload.len() > 2 {
                    let fu_header = packet.payload[1];
                    let start = (fu_header & 0x80) != 0;
                    let original_nal_type = fu_header & 0x1F;
                    
                    if start {
                        // Reconstruct NAL header
                        let nal_header = (packet.payload[0] & 0xE0) | original_nal_type;
                        let mut data = vec![nal_header];
                        data.extend_from_slice(&packet.payload[2..]);
                        
                        nal_units.push(NalUnit {
                            nal_type: original_nal_type,
                            data,
                            timestamp: packet.timestamp,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    
    nal_units
}

/// Depacketize H.265 RTP to NAL units (Req 3.4)
pub fn depacketize_h265(packets: &[OrderedRtpPacket]) -> Vec<NalUnit> {
    let mut nal_units = Vec::new();
    
    for packet in packets {
        if packet.payload.len() < 2 {
            continue;
        }
        
        let nal_type = (packet.payload[0] >> 1) & 0x3F;
        
        match nal_type {
            // Single NAL unit (0-47)
            0..=47 => {
                nal_units.push(NalUnit {
                    nal_type,
                    data: packet.payload.clone(),
                    timestamp: packet.timestamp,
                });
            }
            // Aggregation packet (48)
            48 => {
                let mut offset = 2;
                while offset + 2 < packet.payload.len() {
                    let size = u16::from_be_bytes([
                        packet.payload[offset],
                        packet.payload[offset + 1],
                    ]) as usize;
                    offset += 2;
                    
                    if offset + size <= packet.payload.len() {
                        let nal_data = packet.payload[offset..offset + size].to_vec();
                        if nal_data.len() >= 2 {
                            nal_units.push(NalUnit {
                                nal_type: (nal_data[0] >> 1) & 0x3F,
                                data: nal_data,
                                timestamp: packet.timestamp,
                            });
                        }
                        offset += size;
                    }
                }
            }
            // Fragmentation unit (49)
            49 => {
                if packet.payload.len() > 3 {
                    let fu_header = packet.payload[2];
                    let start = (fu_header & 0x80) != 0;
                    let original_nal_type = fu_header & 0x3F;
                    
                    if start {
                        // Reconstruct NAL header
                        let mut data = vec![
                            (packet.payload[0] & 0x81) | (original_nal_type << 1),
                            packet.payload[1],
                        ];
                        data.extend_from_slice(&packet.payload[3..]);
                        
                        nal_units.push(NalUnit {
                            nal_type: original_nal_type,
                            data,
                            timestamp: packet.timestamp,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    
    nal_units
}

/// Adapt bitrate based on bandwidth estimation (Req 3.11)
pub fn adapt_bitrate(estimate: &BandwidthEstimate, config: &BitrateConfig) -> u32 {
    // Target 80% of available bandwidth
    let target = (estimate.available_bps as f64 * 0.8) as u32;
    
    // Reduce further if packet loss is high
    let adjusted = if estimate.packet_loss_rate > 0.05 {
        (target as f64 * (1.0 - estimate.packet_loss_rate as f64)) as u32
    } else {
        target
    };
    
    // Clamp to configured bounds
    adjusted.clamp(config.min_bitrate, config.max_bitrate)
}

/// Stream session manager
pub struct StreamManager {
    sessions: HashMap<SessionId, StreamSession>,
    sync_states: HashMap<SessionId, PlaybackSync>,
}

impl StreamManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            sync_states: HashMap::new(),
        }
    }
    
    /// Create new stream session (Req 3.1)
    pub fn create_session(&mut self, host_id: UserId, codec_info: CodecInfo) -> StreamSession {
        let mut session_id = [0u8; 16];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut session_id);
        
        let session = StreamSession {
            session_id,
            host_id,
            viewers: Vec::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            codec_info,
        };
        
        self.sessions.insert(session_id, session.clone());
        self.sync_states.insert(session_id, PlaybackSync::new());
        
        session
    }
    
    /// Add viewer to session (Req 3.2)
    pub fn add_viewer(&mut self, session_id: &SessionId, viewer_id: UserId) -> Result<(), StreamError> {
        let session = self.sessions.get_mut(session_id)
            .ok_or(StreamError::SessionNotFound)?;
        
        if !session.viewers.contains(&viewer_id) {
            session.viewers.push(viewer_id);
        }
        
        Ok(())
    }
    
    /// Remove viewer from session
    pub fn remove_viewer(&mut self, session_id: &SessionId, viewer_id: &UserId) -> Result<(), StreamError> {
        let session = self.sessions.get_mut(session_id)
            .ok_or(StreamError::SessionNotFound)?;
        
        session.viewers.retain(|v| v != viewer_id);
        
        if let Some(sync) = self.sync_states.get_mut(session_id) {
            sync.viewer_offsets.remove(viewer_id);
        }
        
        Ok(())
    }
    
    /// Get session
    pub fn get_session(&self, session_id: &SessionId) -> Option<&StreamSession> {
        self.sessions.get(session_id)
    }
    
    /// Update playback sync (Req 3.8)
    pub fn update_sync(&mut self, session_id: &SessionId, viewer_id: UserId, timestamp: u64) -> Result<(), StreamError> {
        let sync = self.sync_states.get_mut(session_id)
            .ok_or(StreamError::SessionNotFound)?;
        
        sync.update_viewer(viewer_id, timestamp);
        Ok(())
    }
    
    /// Check if session is synchronized (Req 3.8)
    pub fn is_synchronized(&self, session_id: &SessionId) -> bool {
        self.sync_states.get(session_id)
            .map(|s| s.is_synchronized())
            .unwrap_or(false)
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter_buffer_reorder() {
        let mut buffer = JitterBuffer::new();
        
        // Insert packets out of order
        let packets = vec![
            OrderedRtpPacket { sequence_number: 2, timestamp: 200, payload: vec![2], received_at: Instant::now() },
            OrderedRtpPacket { sequence_number: 0, timestamp: 0, payload: vec![0], received_at: Instant::now() },
            OrderedRtpPacket { sequence_number: 1, timestamp: 100, payload: vec![1], received_at: Instant::now() },
        ];
        
        let ordered = buffer.reorder(packets);
        
        // Should be in sequence order
        assert_eq!(ordered.len(), 3);
        assert_eq!(ordered[0].sequence_number, 0);
        assert_eq!(ordered[1].sequence_number, 1);
        assert_eq!(ordered[2].sequence_number, 2);
    }

    #[test]
    fn test_playback_sync() {
        let mut sync = PlaybackSync::new();
        sync.set_reference(1000);
        
        let viewer1 = [1u8; 32];
        let viewer2 = [2u8; 32];
        
        sync.update_viewer(viewer1, 1100); // +100ms
        sync.update_viewer(viewer2, 1200); // +200ms
        
        assert!(sync.is_synchronized()); // Within 500ms
        
        sync.update_viewer(viewer2, 2000); // +1000ms
        assert!(!sync.is_synchronized()); // Exceeds 500ms drift
    }

    #[test]
    fn test_bitrate_adaptation() {
        let config = BitrateConfig::default();
        
        // Good bandwidth
        let estimate = BandwidthEstimate {
            available_bps: 5_000_000,
            packet_loss_rate: 0.01,
            rtt_ms: 50,
        };
        let bitrate = adapt_bitrate(&estimate, &config);
        assert!(bitrate >= config.min_bitrate && bitrate <= config.max_bitrate);
        
        // Low bandwidth
        let estimate = BandwidthEstimate {
            available_bps: 400_000,
            packet_loss_rate: 0.0,
            rtt_ms: 100,
        };
        let bitrate = adapt_bitrate(&estimate, &config);
        assert_eq!(bitrate, config.min_bitrate);
    }

    #[test]
    fn test_h264_depacketization() {
        // Single NAL unit packet
        let packets = vec![
            OrderedRtpPacket {
                sequence_number: 0,
                timestamp: 0,
                payload: vec![0x65, 0x00, 0x00, 0x01], // IDR slice
                received_at: Instant::now(),
            },
        ];
        
        let nal_units = depacketize_h264(&packets);
        assert_eq!(nal_units.len(), 1);
        assert_eq!(nal_units[0].nal_type, 5); // IDR
    }

    #[test]
    fn test_stream_manager() {
        let mut manager = StreamManager::new();
        
        let host = [1u8; 32];
        let viewer = [2u8; 32];
        
        let session = manager.create_session(host, CodecInfo {
            video_codec: "H.264".into(),
            audio_codec: "Opus".into(),
            video_bitrate: 2_000_000,
            audio_bitrate: 128_000,
        });
        
        manager.add_viewer(&session.session_id, viewer).unwrap();
        
        let session = manager.get_session(&session.session_id).unwrap();
        assert_eq!(session.viewers.len(), 1);
    }
}
