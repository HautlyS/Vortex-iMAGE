//! Network module - P2P connections via Iroh + optional Tor + Relay fallback
mod iroh_node;
pub mod gossip;
pub mod messenger;
pub mod relay;
pub mod relay_client;
pub mod connection;
pub mod device_link;
pub mod tor_transport;

pub use iroh_node::{IrohNode, SharedNode};
pub use gossip::{GossipNetwork, GossipPayload, PresenceStatus, SharedGossip};
pub use messenger::{P2PMessenger, P2PEvent};
pub use relay::{RelayServer, RelayMessage};
pub use relay_client::{RelayClient, RelayEvent, RelayStatus};
pub use connection::{ConnectionManager, ConnectionQuality, ConnectionType};
pub use device_link::{DeviceLinker, DeviceLinkCode, LinkedDevice, LinkMessage};
pub use tor_transport::{TorStatus, TorConnectionInfo};

#[cfg(feature = "tor")]
pub use tor_transport::TorTransport;

use serde::{Deserialize, Serialize};

/// Peer identifier
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PeerId(pub String);

impl From<&str> for PeerId {
    fn from(s: &str) -> Self {
        PeerId(s.to_string())
    }
}

impl From<String> for PeerId {
    fn from(s: String) -> Self {
        PeerId(s)
    }
}

/// Contact info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contact {
    pub id: PeerId,
    pub name: String,
    pub public_key: [u8; 32],
    pub trusted: bool,
}

/// Shared file/blob
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedItem {
    pub name: String,
    pub size: u64,
    pub hash: String,
    pub contacts: Vec<PeerId>,
}

/// Connection status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Offline,
    Connecting,
    Online,
    OnionRouted,
}

/// Tor circuit info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInfo {
    pub hops: Vec<String>,
    pub latency_ms: u32,
}

/// Network statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub peers_connected: u32,
    pub active_transfers: u32,
}
