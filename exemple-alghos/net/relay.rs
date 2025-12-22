//! Relay server for NAT traversal - adapted from Quantum-Secure-Messaging patterns
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use rand::{RngCore, thread_rng};
use serde::{Deserialize, Serialize};
use pqcrypto_dilithium::dilithium5 as mldsa87;
use pqcrypto_traits::sign::{PublicKey as SignPublicKey, DetachedSignature};

const NONCE_SIZE: usize = 32;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(60);
const PEER_TIMEOUT: Duration = Duration::from_secs(300);
const RATE_LIMIT_COOLDOWN: Duration = Duration::from_secs(60);

#[derive(Clone, Serialize, Deserialize)]
pub enum RelayMessage {
    AuthRequest,
    Challenge { nonce: Vec<u8> },
    ChallengeResponse { public_key: Vec<u8>, nonce: Vec<u8>, signature: Vec<u8> },
    AuthSuccess,
    AuthFailed,
    KeyLink { kyber_key: Vec<u8>, signature: Vec<u8> },
    KeyLinked,
    ConnectRequest { target_key: Vec<u8>, encrypted_port: Vec<u8>, ciphertext: Vec<u8> },
    IncomingConnection { sender_key: Vec<u8>, sender_addr: String, encrypted_port: Vec<u8>, ciphertext: Vec<u8> },
    Heartbeat,
    HeartbeatAck,
    Disconnect,
}

#[derive(Clone)]
#[allow(dead_code)] // Fields used for peer tracking and future key exchange
struct PeerInfo {
    dilithium_key: Vec<u8>,
    kyber_key: Option<Vec<u8>>,
    last_seen: Instant,
    last_request: Instant,
    addr: String,
}

pub struct RelayServer {
    peers: Arc<RwLock<HashMap<Vec<u8>, PeerInfo>>>,
    connections: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<RelayMessage>>>>,
}

impl RelayServer {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn run(&self, addr: &str) -> anyhow::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("Relay server listening on {}", addr);

        let peers = self.peers.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(HEARTBEAT_INTERVAL).await;
                Self::cleanup_peers(&peers).await;
            }
        });

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let peers = self.peers.clone();
            let connections = self.connections.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, peer_addr.to_string(), peers, connections).await {
                    tracing::warn!("Connection error: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        mut stream: TcpStream,
        addr: String,
        peers: Arc<RwLock<HashMap<Vec<u8>, PeerInfo>>>,
        connections: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<RelayMessage>>>>,
    ) -> anyhow::Result<()> {
        let msg = Self::read_message(&mut stream).await?;
        
        match msg {
            RelayMessage::AuthRequest => {
                Self::handle_auth(stream, addr, peers, connections).await
            }
            RelayMessage::ConnectRequest { target_key, encrypted_port, ciphertext } => {
                Self::handle_connect_request(&addr, target_key, encrypted_port, ciphertext, &peers, &connections).await
            }
            _ => Ok(())
        }
    }

    async fn handle_auth(
        mut stream: TcpStream,
        addr: String,
        peers: Arc<RwLock<HashMap<Vec<u8>, PeerInfo>>>,
        connections: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<RelayMessage>>>>,
    ) -> anyhow::Result<()> {
        // Generate challenge nonce
        let mut nonce = vec![0u8; NONCE_SIZE];
        thread_rng().fill_bytes(&mut nonce);
        
        Self::write_message(&mut stream, &RelayMessage::Challenge { nonce: nonce.clone() }).await?;
        
        // Wait for response
        let response = Self::read_message(&mut stream).await?;
        
        if let RelayMessage::ChallengeResponse { public_key, nonce: returned_nonce, signature } = response {
            if returned_nonce != nonce {
                Self::write_message(&mut stream, &RelayMessage::AuthFailed).await?;
                return Ok(());
            }
            
            // CRITICAL FIX #2: Proper ML-DSA-87 signature verification
            // Verify the signature cryptographically using the provided public key
            if !Self::verify_signature(&public_key, &nonce, &signature) {
                tracing::warn!("Relay auth failed: invalid signature from {}", addr);
                Self::write_message(&mut stream, &RelayMessage::AuthFailed).await?;
                return Ok(());
            }
            
            Self::write_message(&mut stream, &RelayMessage::AuthSuccess).await?;
            
            // Wait for key linking
            let key_link = Self::read_message(&mut stream).await?;
            
            if let RelayMessage::KeyLink { kyber_key, signature: _ } = key_link {
                // Store peer
                let peer_info = PeerInfo {
                    dilithium_key: public_key.clone(),
                    kyber_key: Some(kyber_key.clone()),
                    last_seen: Instant::now(),
                    last_request: Instant::now() - RATE_LIMIT_COOLDOWN,
                    addr: addr.clone(),
                };
                
                peers.write().await.insert(kyber_key.clone(), peer_info);
                Self::write_message(&mut stream, &RelayMessage::KeyLinked).await?;
                
                // Create channel for incoming messages
                let (tx, mut rx) = tokio::sync::mpsc::channel::<RelayMessage>(32);
                connections.write().await.insert(hex::encode(&kyber_key), tx);
                
                // Listen for incoming connection requests
                loop {
                    tokio::select! {
                        msg = rx.recv() => {
                            if let Some(msg) = msg {
                                Self::write_message(&mut stream, &msg).await?;
                            } else {
                                break;
                            }
                        }
                        result = Self::read_message(&mut stream) => {
                            match result {
                                Ok(RelayMessage::Heartbeat) => {
                                    if let Some(peer) = peers.write().await.get_mut(&kyber_key) {
                                        peer.last_seen = Instant::now();
                                    }
                                    Self::write_message(&mut stream, &RelayMessage::HeartbeatAck).await?;
                                }
                                Ok(RelayMessage::Disconnect) | Err(_) => break,
                                _ => {}
                            }
                        }
                    }
                }
                
                // Cleanup
                peers.write().await.remove(&kyber_key);
                connections.write().await.remove(&hex::encode(&kyber_key));
            }
        }
        
        Ok(())
    }

    async fn handle_connect_request(
        sender_addr: &str,
        target_key: Vec<u8>,
        encrypted_port: Vec<u8>,
        ciphertext: Vec<u8>,
        peers: &Arc<RwLock<HashMap<Vec<u8>, PeerInfo>>>,
        connections: &Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<RelayMessage>>>>,
    ) -> anyhow::Result<()> {
        let mut peers_guard = peers.write().await;
        
        // Rate limiting
        if let Some(peer) = peers_guard.get_mut(&target_key) {
            if peer.last_request.elapsed() < RATE_LIMIT_COOLDOWN {
                return Ok(());
            }
            peer.last_request = Instant::now();
        }
        
        drop(peers_guard);
        
        // Forward to target
        let target_hex = hex::encode(&target_key);
        if let Some(tx) = connections.read().await.get(&target_hex) {
            let _ = tx.send(RelayMessage::IncomingConnection {
                sender_key: vec![], // Would be filled from auth context
                sender_addr: sender_addr.to_string(),
                encrypted_port,
                ciphertext,
            }).await;
        }
        
        Ok(())
    }

    async fn cleanup_peers(peers: &Arc<RwLock<HashMap<Vec<u8>, PeerInfo>>>) {
        let mut guard = peers.write().await;
        guard.retain(|_, peer| peer.last_seen.elapsed() < PEER_TIMEOUT);
    }

    async fn read_message(stream: &mut TcpStream) -> anyhow::Result<RelayMessage> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;
        
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;
        
        bincode::deserialize(&buf).map_err(Into::into)
    }

    async fn write_message(stream: &mut TcpStream, msg: &RelayMessage) -> anyhow::Result<()> {
        let data = bincode::serialize(msg)?;
        let len = (data.len() as u32).to_be_bytes();
        stream.write_all(&len).await?;
        stream.write_all(&data).await?;
        Ok(())
    }
}

impl RelayServer {
    /// Verify ML-DSA-87 (Dilithium5) signature (CRITICAL FIX #2)
    fn verify_signature(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        // Parse public key
        let pk = match mldsa87::PublicKey::from_bytes(public_key) {
            Ok(pk) => pk,
            Err(_) => {
                tracing::warn!("Invalid public key format");
                return false;
            }
        };
        
        // Parse signature
        let sig = match mldsa87::DetachedSignature::from_bytes(signature) {
            Ok(sig) => sig,
            Err(_) => {
                tracing::warn!("Invalid signature format");
                return false;
            }
        };
        
        // Verify signature
        mldsa87::verify_detached_signature(&sig, message, &pk).is_ok()
    }
}

impl Default for RelayServer {
    fn default() -> Self {
        Self::new()
    }
}
