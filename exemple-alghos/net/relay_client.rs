//! Relay client for NAT traversal - connects to relay server for peer discovery
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use anyhow::Result;

use super::relay::RelayMessage;
use crate::crypto::HybridKeypair;

const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(60);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone, Debug)]
pub enum RelayEvent {
    Connected,
    Disconnected,
    IncomingConnection { sender_addr: String, encrypted_port: Vec<u8>, ciphertext: Vec<u8> },
    AuthFailed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RelayStatus {
    Disconnected,
    Connecting,
    Authenticating,
    Connected,
    Reconnecting,
}

pub struct RelayClient {
    server_addr: String,
    keypair: Arc<HybridKeypair>,
    status: Arc<RwLock<RelayStatus>>,
    event_tx: mpsc::Sender<RelayEvent>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl RelayClient {
    pub fn new(server_addr: String, keypair: HybridKeypair) -> (Self, mpsc::Receiver<RelayEvent>) {
        let (event_tx, event_rx) = mpsc::channel(64);
        
        (Self {
            server_addr,
            keypair: Arc::new(keypair),
            status: Arc::new(RwLock::new(RelayStatus::Disconnected)),
            event_tx,
            shutdown_tx: None,
        }, event_rx)
    }

    pub async fn connect(&mut self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        let server_addr = self.server_addr.clone();
        let keypair = self.keypair.clone();
        let status = self.status.clone();
        let event_tx = self.event_tx.clone();
        
        tokio::spawn(async move {
            let mut reconnect_delay = RECONNECT_DELAY;
            
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => break,
                    result = Self::connect_and_run(&server_addr, &keypair, &status, &event_tx) => {
                        if let Err(e) = result {
                            tracing::warn!("Relay connection error: {}", e);
                        }
                        
                        *status.write().await = RelayStatus::Reconnecting;
                        let _ = event_tx.send(RelayEvent::Disconnected).await;
                        
                        tokio::time::sleep(reconnect_delay).await;
                        reconnect_delay = (reconnect_delay * 2).min(MAX_RECONNECT_DELAY);
                    }
                }
            }
        });
        
        Ok(())
    }

    async fn connect_and_run(
        server_addr: &str,
        keypair: &HybridKeypair,
        status: &Arc<RwLock<RelayStatus>>,
        event_tx: &mpsc::Sender<RelayEvent>,
    ) -> Result<()> {
        *status.write().await = RelayStatus::Connecting;
        
        let mut stream = TcpStream::connect(server_addr).await?;
        
        *status.write().await = RelayStatus::Authenticating;
        
        // Send auth request
        Self::write_message(&mut stream, &RelayMessage::AuthRequest).await?;
        
        // Receive challenge
        let challenge = Self::read_message(&mut stream).await?;
        
        if let RelayMessage::Challenge { nonce } = challenge {
            // Sign the nonce with our keypair
            let public_key = keypair.public_bundle().pq.to_vec();
            let signature = keypair.sign(&nonce);
            
            Self::write_message(&mut stream, &RelayMessage::ChallengeResponse {
                public_key: public_key.clone(),
                nonce,
                signature,
            }).await?;
            
            // Wait for auth result
            let result = Self::read_message(&mut stream).await?;
            
            match result {
                RelayMessage::AuthSuccess => {
                    // Link our Kyber key
                    let kyber_key = keypair.public_bundle().pq.to_vec();
                    let signature = keypair.sign(&kyber_key);
                    
                    Self::write_message(&mut stream, &RelayMessage::KeyLink {
                        kyber_key,
                        signature,
                    }).await?;
                    
                    let link_result = Self::read_message(&mut stream).await?;
                    
                    if matches!(link_result, RelayMessage::KeyLinked) {
                        *status.write().await = RelayStatus::Connected;
                        let _ = event_tx.send(RelayEvent::Connected).await;
                        
                        // Main loop - handle heartbeats and incoming connections
                        Self::run_loop(&mut stream, status, event_tx).await?;
                    }
                }
                RelayMessage::AuthFailed => {
                    let _ = event_tx.send(RelayEvent::AuthFailed).await;
                    return Err(anyhow::anyhow!("Authentication failed"));
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    async fn run_loop(
        stream: &mut TcpStream,
        status: &Arc<RwLock<RelayStatus>>,
        event_tx: &mpsc::Sender<RelayEvent>,
    ) -> Result<()> {
        let mut heartbeat_interval = tokio::time::interval(HEARTBEAT_INTERVAL);
        
        loop {
            tokio::select! {
                _ = heartbeat_interval.tick() => {
                    Self::write_message(stream, &RelayMessage::Heartbeat).await?;
                }
                result = Self::read_message(stream) => {
                    match result? {
                        RelayMessage::HeartbeatAck => {}
                        RelayMessage::IncomingConnection { sender_addr, encrypted_port, ciphertext, .. } => {
                            let _ = event_tx.send(RelayEvent::IncomingConnection {
                                sender_addr,
                                encrypted_port,
                                ciphertext,
                            }).await;
                        }
                        RelayMessage::Disconnect => {
                            *status.write().await = RelayStatus::Disconnected;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        
        Ok(())
    }

    pub async fn request_connection(
        &self,
        target_key: Vec<u8>,
        encrypted_port: Vec<u8>,
        ciphertext: Vec<u8>,
    ) -> Result<()> {
        let mut stream = TcpStream::connect(&self.server_addr).await?;
        
        Self::write_message(&mut stream, &RelayMessage::ConnectRequest {
            target_key,
            encrypted_port,
            ciphertext,
        }).await?;
        
        Ok(())
    }

    pub async fn disconnect(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        *self.status.write().await = RelayStatus::Disconnected;
    }

    pub async fn status(&self) -> RelayStatus {
        *self.status.read().await
    }

    async fn read_message(stream: &mut TcpStream) -> Result<RelayMessage> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;
        
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;
        
        bincode::deserialize(&buf).map_err(Into::into)
    }

    async fn write_message(stream: &mut TcpStream, msg: &RelayMessage) -> Result<()> {
        let data = bincode::serialize(msg)?;
        let len = (data.len() as u32).to_be_bytes();
        stream.write_all(&len).await?;
        stream.write_all(&data).await?;
        Ok(())
    }
}
