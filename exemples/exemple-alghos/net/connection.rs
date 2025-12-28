//! Connection manager - unified connection status, auto-reconnect, quality metrics
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use super::{ConnectionStatus, RelayStatus};
use super::TorStatus;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType {
    Direct,      // Direct P2P via Iroh
    Relay,       // Via relay server
    Tor,         // Via Tor network
    TorRelay,    // Tor + Relay fallback
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionQuality {
    pub latency_ms: u32,
    pub packet_loss: f32,
    pub bandwidth_kbps: u32,
    pub connection_type: ConnectionType,
    pub uptime_secs: u64,
}

impl Default for ConnectionQuality {
    fn default() -> Self {
        Self {
            latency_ms: 0,
            packet_loss: 0.0,
            bandwidth_kbps: 0,
            connection_type: ConnectionType::Direct,
            uptime_secs: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionState {
    pub status: ConnectionStatus,
    pub quality: ConnectionQuality,
    pub relay_status: Option<String>,
    pub tor_status: Option<String>,
    pub node_id: String,
    pub peers_connected: u32,
}

pub struct ConnectionManager {
    state: Arc<RwLock<ConnectionState>>,
    connected_at: Arc<RwLock<Option<Instant>>>,
    ping_samples: Arc<RwLock<Vec<u32>>>,
}

impl ConnectionManager {
    pub fn new(node_id: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(ConnectionState {
                status: ConnectionStatus::Offline,
                quality: ConnectionQuality::default(),
                relay_status: None,
                tor_status: None,
                node_id,
                peers_connected: 0,
            })),
            connected_at: Arc::new(RwLock::new(None)),
            ping_samples: Arc::new(RwLock::new(Vec::with_capacity(10))),
        }
    }

    pub async fn update_status(&self, status: ConnectionStatus) {
        let mut state = self.state.write().await;
        let was_offline = state.status == ConnectionStatus::Offline;
        state.status = status.clone();
        
        if was_offline && status != ConnectionStatus::Offline {
            *self.connected_at.write().await = Some(Instant::now());
        } else if status == ConnectionStatus::Offline {
            *self.connected_at.write().await = None;
        }
        
        // Update connection type based on status
        state.quality.connection_type = match status {
            ConnectionStatus::OnionRouted => ConnectionType::Tor,
            _ => state.quality.connection_type,
        };
    }

    pub async fn update_relay_status(&self, status: RelayStatus) {
        let mut state = self.state.write().await;
        state.relay_status = Some(format!("{:?}", status));
        
        if status == RelayStatus::Connected {
            state.quality.connection_type = match state.status {
                ConnectionStatus::OnionRouted => ConnectionType::TorRelay,
                _ => ConnectionType::Relay,
            };
        }
    }

    pub async fn update_tor_status(&self, status: TorStatus) {
        let mut state = self.state.write().await;
        state.tor_status = Some(format!("{:?}", status));
    }

    pub async fn record_ping(&self, latency_ms: u32) {
        let mut samples = self.ping_samples.write().await;
        if samples.len() >= 10 {
            samples.remove(0);
        }
        samples.push(latency_ms);
        
        // Update average latency
        let avg = samples.iter().sum::<u32>() / samples.len() as u32;
        self.state.write().await.quality.latency_ms = avg;
    }

    pub async fn update_peers(&self, count: u32) {
        self.state.write().await.peers_connected = count;
    }

    pub async fn update_bandwidth(&self, kbps: u32) {
        self.state.write().await.quality.bandwidth_kbps = kbps;
    }

    pub async fn get_state(&self) -> ConnectionState {
        let mut state = self.state.read().await.clone();
        
        // Update uptime
        if let Some(connected_at) = *self.connected_at.read().await {
            state.quality.uptime_secs = connected_at.elapsed().as_secs();
        }
        
        state
    }

    pub async fn get_quality(&self) -> ConnectionQuality {
        self.get_state().await.quality
    }

    pub async fn is_connected(&self) -> bool {
        let state = self.state.read().await;
        matches!(state.status, ConnectionStatus::Online | ConnectionStatus::OnionRouted)
    }

    pub async fn connection_type(&self) -> ConnectionType {
        self.state.read().await.quality.connection_type
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_manager() {
        let mgr = ConnectionManager::new("test_node".into());
        
        assert!(!mgr.is_connected().await);
        
        mgr.update_status(ConnectionStatus::Online).await;
        assert!(mgr.is_connected().await);
        
        mgr.record_ping(50).await;
        mgr.record_ping(60).await;
        
        let quality = mgr.get_quality().await;
        assert_eq!(quality.latency_ms, 55);
    }
}
