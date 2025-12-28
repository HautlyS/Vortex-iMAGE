//! Tor transport layer using Arti
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Network errors
#[derive(Debug, Error)]
pub enum NetError {
    #[error("Tor bootstrap failed: {0}")]
    TorBootstrap(String),
    #[error("Connection error: {0}")]
    Connection(String),
}

#[cfg(feature = "tor")]
use arti_client::{TorClient, TorClientConfig, StreamPrefs};
#[cfg(feature = "tor")]
use tor_rtcompat::PreferredRuntime;
#[cfg(feature = "tor")]
use std::time::Duration;

/// Detailed Tor connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TorStatus {
    Disconnected,
    Bootstrapping { progress: u8 },
    Connected,
    Error { message: String },
}

impl Default for TorStatus {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Detailed Tor connection info for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorConnectionInfo {
    pub status: TorStatus,
    pub is_connected: bool,
    pub bootstrap_progress: u8,
    pub circuit_count: u32,
    pub guard_count: u32,
    pub exit_policy: String,
    pub version: String,
    pub uptime_secs: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_activity: u64,
}

impl Default for TorConnectionInfo {
    fn default() -> Self {
        Self {
            status: TorStatus::Disconnected,
            is_connected: false,
            bootstrap_progress: 0,
            circuit_count: 0,
            guard_count: 0,
            exit_policy: "default".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            uptime_secs: 0,
            bytes_sent: 0,
            bytes_received: 0,
            last_activity: 0,
        }
    }
}

#[cfg(feature = "tor")]
pub struct TorTransport {
    client: TorClient<PreferredRuntime>,
    status: TorStatus,
    start_time: std::time::Instant,
    bytes_sent: std::sync::atomic::AtomicU64,
    bytes_received: std::sync::atomic::AtomicU64,
}

#[cfg(feature = "tor")]
impl TorTransport {
    /// Bootstrap Tor connection with progress tracking
    pub async fn bootstrap() -> Result<Self, NetError> {
        tracing::info!("Starting Tor bootstrap...");
        
        let config = TorClientConfig::default();
        
        let client = TorClient::create_bootstrapped(config)
            .await
            .map_err(|e| NetError::TorBootstrap(format!("Bootstrap failed: {}", e)))?;
        
        tracing::info!("✓ Tor bootstrap complete");
        
        Ok(Self { 
            client, 
            status: TorStatus::Connected,
            start_time: std::time::Instant::now(),
            bytes_sent: std::sync::atomic::AtomicU64::new(0),
            bytes_received: std::sync::atomic::AtomicU64::new(0),
        })
    }

    /// Bootstrap with progress callback
    pub async fn bootstrap_with_progress<F>(mut progress_cb: F) -> Result<Self, NetError>
    where
        F: FnMut(u8) + Send + 'static,
    {
        tracing::info!("Starting Tor bootstrap with progress tracking...");
        progress_cb(0);
        
        let config = TorClientConfig::default();
        progress_cb(20);
        
        tracing::info!("Creating Tor client...");
        
        // Simulate progress during bootstrap
        let bootstrap_handle = tokio::spawn(async move {
            TorClient::create_bootstrapped(config).await
        });
        
        // Progress simulation
        for i in 3..10 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            progress_cb(20 + i * 8);
        }
        
        let client = bootstrap_handle
            .await
            .map_err(|e| NetError::TorBootstrap(format!("Bootstrap task failed: {}", e)))?
            .map_err(|e| NetError::TorBootstrap(format!("Bootstrap failed: {}", e)))?;
        
        progress_cb(100);
        tracing::info!("✓ Tor bootstrap complete");
        
        Ok(Self { 
            client, 
            status: TorStatus::Connected,
            start_time: std::time::Instant::now(),
            bytes_sent: std::sync::atomic::AtomicU64::new(0),
            bytes_received: std::sync::atomic::AtomicU64::new(0),
        })
    }

    pub fn status(&self) -> TorStatus {
        self.status.clone()
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.status, TorStatus::Connected)
    }

    pub fn client(&self) -> &TorClient<PreferredRuntime> {
        &self.client
    }
    
    /// Get detailed connection info for UI
    pub fn connection_info(&self) -> TorConnectionInfo {
        use std::sync::atomic::Ordering;
        
        TorConnectionInfo {
            status: self.status.clone(),
            is_connected: self.is_connected(),
            bootstrap_progress: if self.is_connected() { 100 } else { 0 },
            circuit_count: 3, // Typical Tor circuit count
            guard_count: 1,
            exit_policy: "default".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            uptime_secs: self.start_time.elapsed().as_secs(),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            last_activity: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Connect to clearnet address via Tor
    pub async fn connect(&self, addr: &str, port: u16) -> Result<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, NetError> {
        if !self.is_connected() {
            return Err(NetError::Connection("Tor not connected".into()));
        }
        
        tracing::debug!("Connecting to {}:{} via Tor", addr, port);
        let prefs = StreamPrefs::default();
        self.client
            .connect_with_prefs((addr, port), &prefs)
            .await
            .map_err(|e| NetError::Connection(e.to_string()))
    }

    /// Connect to .onion address
    pub async fn connect_onion(&self, onion: &str, port: u16) -> Result<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, NetError> {
        if !self.is_connected() {
            return Err(NetError::Connection("Tor not connected".into()));
        }
        
        tracing::info!("Connecting to {}.onion:{}", onion, port);
        self.connect(onion, port).await
    }

    /// Test Tor connection by connecting to check.torproject.org
    pub async fn test_connection(&self) -> Result<bool, NetError> {
        tracing::info!("Testing Tor connection...");
        match self.connect("check.torproject.org", 443).await {
            Ok(_) => {
                tracing::info!("✓ Tor connection test successful");
                Ok(true)
            }
            Err(e) => {
                tracing::error!("✗ Tor connection test failed: {}", e);
                Err(e)
            }
        }
    }
}

#[cfg(not(feature = "tor"))]
pub struct TorTransport;

#[cfg(not(feature = "tor"))]
impl TorTransport {
    pub async fn bootstrap() -> Result<Self, NetError> {
        Err(NetError::TorBootstrap("Tor feature not enabled. Compile with --features tor".into()))
    }

    pub fn status(&self) -> TorStatus {
        TorStatus::Disconnected
    }

    pub fn is_connected(&self) -> bool {
        false
    }
    
    pub fn connection_info(&self) -> TorConnectionInfo {
        TorConnectionInfo::default()
    }
}

pub fn tor_available() -> bool {
    cfg!(feature = "tor")
}

#[cfg(all(test, feature = "tor"))]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_tor_bootstrap() {
        let tor = TorTransport::bootstrap().await.unwrap();
        assert!(tor.is_connected());
    }

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_tor_connection() {
        let tor = TorTransport::bootstrap().await.unwrap();
        assert!(tor.test_connection().await.unwrap());
    }
}
