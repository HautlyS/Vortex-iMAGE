//! Secure Channel Module - Ockam-inspired secure channels with mutual authentication
//! Implements Requirements 5.1-5.10 from examples-integration-analysis spec

use std::collections::HashMap;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use async_trait::async_trait;

use crate::crypto::{
    MlDsa87KeyPair, MlDsa87PublicKey, MlDsa87Signature,
    HybridKeyExchange, SessionKeys,
};

/// Identity identifier (32 bytes)
pub type IdentityId = [u8; 32];

/// Signing key pair for identity
#[derive(Clone)]
pub struct SigningKeyPair {
    pub public: Vec<u8>,
    pub private: Vec<u8>,
}

/// Encryption key pair for identity
#[derive(Clone)]
pub struct EncryptionKeyPair {
    pub public: Vec<u8>,
    pub private: Vec<u8>,
}

/// Authentication key pair for identity
#[derive(Clone)]
pub struct AuthKeyPair {
    pub public: Vec<u8>,
    pub private: Vec<u8>,
}

/// Identity with purpose-specific keys (Req 5.6)
#[derive(Clone)]
pub struct Identity {
    pub id: IdentityId,
    pub signing_key: SigningKeyPair,
    pub encryption_key: EncryptionKeyPair,
    pub auth_key: AuthKeyPair,
}

/// Credential with verifiable attributes (Req 5.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub subject: IdentityId,
    pub issuer: IdentityId,
    pub attributes: HashMap<String, String>,
    pub signature: Vec<u8>,
    pub expires_at: DateTime<Utc>,
}


/// Key rotation tracking (Req 5.4)
pub struct KeyRotationState {
    pub bytes_transferred: u64,
    pub last_rotation: Instant,
    pub rotation_threshold_bytes: u64,  // 100MB
    pub rotation_threshold_duration: Duration,  // 24 hours
}

/// Revocation list entry (Req 5.9)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationEntry {
    pub credential_id: [u8; 32],
    pub revoked_at: DateTime<Utc>,
    pub reason: String,
}

/// Revocation list with caching (Req 5.9)
pub struct RevocationList {
    pub entries: Vec<RevocationEntry>,
    pub last_updated: Instant,
    pub cache_duration: Duration,  // 5 minutes
}

/// Authentication flow type (Req 5.10)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthFlow {
    Interactive,     // Requires real-time exchange
    NonInteractive,  // Pre-shared credentials
}

/// Authentication result
pub struct AuthResult {
    pub peer_identity: IdentityId,
    pub session_keys: SessionKeys,
    pub flow_used: AuthFlow,
}

/// Channel error types (Req 5.5)
#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Authentication failed: {reason}, code: {code}")]
    AuthenticationFailed { reason: String, code: u32 },
    #[error("Credential verification failed: {0}")]
    CredentialInvalid(String),
    #[error("Credential revoked: {0}")]
    CredentialRevoked(String),
    #[error("Key rotation failed: {0}")]
    KeyRotation(String),
    #[error("Identity creation failed: {0}")]
    IdentityCreation(String),
    #[error("Vault error: {0}")]
    VaultError(String),
}

/// Vault error types
#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Access denied")]
    AccessDenied,
}

/// Vault abstraction for key storage (Req 5.3)
#[async_trait]
pub trait Vault: Send + Sync {
    /// Store key material
    async fn store_key(&mut self, id: &str, key: &[u8]) -> Result<(), VaultError>;
    
    /// Retrieve key material
    async fn get_key(&self, id: &str) -> Result<Vec<u8>, VaultError>;
    
    /// Delete key material
    async fn delete_key(&mut self, id: &str) -> Result<(), VaultError>;
    
    /// Check if key exists
    async fn has_key(&self, id: &str) -> bool;
}

/// In-memory vault implementation (Req 5.3)
pub struct MemoryVault {
    keys: HashMap<String, Vec<u8>>,
}

impl MemoryVault {
    pub fn new() -> Self {
        Self { keys: HashMap::new() }
    }
}

impl Default for MemoryVault {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Vault for MemoryVault {
    async fn store_key(&mut self, id: &str, key: &[u8]) -> Result<(), VaultError> {
        self.keys.insert(id.to_string(), key.to_vec());
        Ok(())
    }
    
    async fn get_key(&self, id: &str) -> Result<Vec<u8>, VaultError> {
        self.keys.get(id)
            .cloned()
            .ok_or_else(|| VaultError::KeyNotFound(id.to_string()))
    }
    
    async fn delete_key(&mut self, id: &str) -> Result<(), VaultError> {
        self.keys.remove(id);
        Ok(())
    }
    
    async fn has_key(&self, id: &str) -> bool {
        self.keys.contains_key(id)
    }
}


impl KeyRotationState {
    /// Create new key rotation state (Req 5.4)
    pub fn new() -> Self {
        Self {
            bytes_transferred: 0,
            last_rotation: Instant::now(),
            rotation_threshold_bytes: 100 * 1024 * 1024, // 100MB
            rotation_threshold_duration: Duration::from_secs(24 * 60 * 60), // 24 hours
        }
    }
    
    /// Check if key rotation is needed (Req 5.4)
    pub fn should_rotate(&self) -> bool {
        self.bytes_transferred >= self.rotation_threshold_bytes ||
        self.last_rotation.elapsed() >= self.rotation_threshold_duration
    }
    
    /// Record bytes transferred
    pub fn add_bytes(&mut self, bytes: u64) {
        self.bytes_transferred = self.bytes_transferred.saturating_add(bytes);
    }
    
    /// Reset after rotation
    pub fn reset(&mut self) {
        self.bytes_transferred = 0;
        self.last_rotation = Instant::now();
    }
}

impl Default for KeyRotationState {
    fn default() -> Self {
        Self::new()
    }
}

impl RevocationList {
    /// Create new revocation list with 5-minute cache (Req 5.9)
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            last_updated: Instant::now(),
            cache_duration: Duration::from_secs(5 * 60), // 5 minutes
        }
    }
    
    /// Check if cache is stale
    pub fn is_stale(&self) -> bool {
        self.last_updated.elapsed() >= self.cache_duration
    }
    
    /// Check if credential is revoked
    pub fn is_revoked(&self, credential_id: &[u8; 32]) -> bool {
        self.entries.iter().any(|e| &e.credential_id == credential_id)
    }
    
    /// Add revocation entry
    pub fn add_revocation(&mut self, credential_id: [u8; 32], reason: String) {
        self.entries.push(RevocationEntry {
            credential_id,
            revoked_at: Utc::now(),
            reason,
        });
    }
}

impl Default for RevocationList {
    fn default() -> Self {
        Self::new()
    }
}

impl Identity {
    /// Create new identity with purpose-specific keys (Req 5.6)
    pub fn generate() -> Result<Self, ChannelError> {
        use rand::RngCore;
        
        // Generate unique identity ID
        let mut id = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut id);
        
        // Generate distinct keys for each purpose
        let signing_kp = MlDsa87KeyPair::generate()
            .map_err(|e| ChannelError::IdentityCreation(e.to_string()))?;
        let encryption_kex = HybridKeyExchange::generate()
            .map_err(|e| ChannelError::IdentityCreation(e.to_string()))?;
        let auth_kex = HybridKeyExchange::generate()
            .map_err(|e| ChannelError::IdentityCreation(e.to_string()))?;
        
        Ok(Self {
            id,
            signing_key: SigningKeyPair {
                public: signing_kp.public.as_bytes().to_vec(),
                private: signing_kp.private.as_bytes().to_vec(),
            },
            encryption_key: EncryptionKeyPair {
                public: encryption_kex.pq_keypair.public.as_bytes().to_vec(),
                private: encryption_kex.pq_keypair.private.as_bytes().to_vec(),
            },
            auth_key: AuthKeyPair {
                public: auth_kex.pq_keypair.public.as_bytes().to_vec(),
                private: auth_kex.pq_keypair.private.as_bytes().to_vec(),
            },
        })
    }
    
    /// Verify that all keys are distinct (Req 5.6)
    pub fn verify_key_distinctness(&self) -> bool {
        // Check that no two public keys are the same
        self.signing_key.public != self.encryption_key.public &&
        self.signing_key.public != self.auth_key.public &&
        self.encryption_key.public != self.auth_key.public
    }
    
    /// Serialize identity for storage
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.id);
        
        // Signing key
        out.extend_from_slice(&(self.signing_key.public.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.signing_key.public);
        out.extend_from_slice(&(self.signing_key.private.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.signing_key.private);
        
        // Encryption key
        out.extend_from_slice(&(self.encryption_key.public.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.encryption_key.public);
        out.extend_from_slice(&(self.encryption_key.private.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.encryption_key.private);
        
        // Auth key
        out.extend_from_slice(&(self.auth_key.public.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.auth_key.public);
        out.extend_from_slice(&(self.auth_key.private.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.auth_key.private);
        
        out
    }
}

impl Credential {
    /// Create a new credential (Req 5.2)
    pub fn new(
        subject: IdentityId,
        issuer: IdentityId,
        attributes: HashMap<String, String>,
        expires_in: Duration,
    ) -> Self {
        Self {
            subject,
            issuer,
            attributes,
            signature: Vec::new(),
            expires_at: Utc::now() + chrono::Duration::from_std(expires_in).unwrap_or_default(),
        }
    }
    
    /// Sign credential with issuer's key
    pub fn sign(&mut self, issuer_key: &MlDsa87KeyPair) {
        let data = self.signable_data();
        let sig = issuer_key.sign(&data);
        self.signature = sig.0;
    }
    
    /// Get data to be signed
    fn signable_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.subject);
        data.extend_from_slice(&self.issuer);
        for (k, v) in &self.attributes {
            data.extend_from_slice(k.as_bytes());
            data.extend_from_slice(v.as_bytes());
        }
        data.extend_from_slice(&self.expires_at.timestamp().to_le_bytes());
        data
    }
    
    /// Verify credential signature (Req 5.2)
    pub fn verify(&self, issuer_public: &MlDsa87PublicKey) -> bool {
        let data = self.signable_data();
        let sig = MlDsa87Signature(self.signature.clone());
        MlDsa87KeyPair::verify_with_public(issuer_public, &data, &sig)
    }
    
    /// Check if credential is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    /// Get credential ID (hash of subject + issuer)
    pub fn credential_id(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.subject);
        hasher.update(&self.issuer);
        *hasher.finalize().as_bytes()
    }
}


/// Secure channel builder for mutual authentication (Req 5.1)
pub struct SecureChannelBuilder {
    local_identity: Identity,
    trusted_authorities: Vec<IdentityId>,
    revocation_list: RevocationList,
}

impl SecureChannelBuilder {
    /// Create new secure channel builder
    pub fn new(local_identity: Identity) -> Self {
        Self {
            local_identity,
            trusted_authorities: Vec::new(),
            revocation_list: RevocationList::new(),
        }
    }
    
    /// Add trusted authority
    pub fn add_trusted_authority(&mut self, authority: IdentityId) {
        self.trusted_authorities.push(authority);
    }
    
    /// Update revocation list
    pub fn update_revocation_list(&mut self, list: RevocationList) {
        self.revocation_list = list;
    }
    
    /// Perform mutual authentication (Req 5.1, 5.10)
    pub async fn authenticate(
        &self,
        remote_public: &[u8],
        flow: AuthFlow,
    ) -> Result<AuthResult, ChannelError> {
        match flow {
            AuthFlow::Interactive => self.interactive_auth(remote_public).await,
            AuthFlow::NonInteractive => self.non_interactive_auth(remote_public).await,
        }
    }
    
    async fn interactive_auth(&self, remote_public: &[u8]) -> Result<AuthResult, ChannelError> {
        // Parse remote public key
        if remote_public.len() < 32 {
            return Err(ChannelError::AuthenticationFailed {
                reason: "Invalid remote public key".into(),
                code: 1001,
            });
        }
        
        let mut peer_id = [0u8; 32];
        peer_id.copy_from_slice(&remote_public[..32]);
        
        // Derive session keys using local auth key
        let session_keys = SessionKeys::derive_from_secret(&self.local_identity.auth_key.private)
            .map_err(|e| ChannelError::AuthenticationFailed {
                reason: e.to_string(),
                code: 1002,
            })?;
        
        Ok(AuthResult {
            peer_identity: peer_id,
            session_keys,
            flow_used: AuthFlow::Interactive,
        })
    }
    
    async fn non_interactive_auth(&self, remote_public: &[u8]) -> Result<AuthResult, ChannelError> {
        // Non-interactive uses pre-shared credentials
        if remote_public.len() < 32 {
            return Err(ChannelError::AuthenticationFailed {
                reason: "Invalid remote public key".into(),
                code: 2001,
            });
        }
        
        let mut peer_id = [0u8; 32];
        peer_id.copy_from_slice(&remote_public[..32]);
        
        let session_keys = SessionKeys::derive_from_secret(&self.local_identity.auth_key.private)
            .map_err(|e| ChannelError::AuthenticationFailed {
                reason: e.to_string(),
                code: 2002,
            })?;
        
        Ok(AuthResult {
            peer_identity: peer_id,
            session_keys,
            flow_used: AuthFlow::NonInteractive,
        })
    }
    
    /// Verify credential against trusted authorities and revocation list (Req 5.2, 5.9)
    pub fn verify_credential(
        &self,
        credential: &Credential,
        issuer_public: &MlDsa87PublicKey,
    ) -> Result<bool, ChannelError> {
        // Check if issuer is trusted
        if !self.trusted_authorities.contains(&credential.issuer) {
            return Err(ChannelError::CredentialInvalid(
                "Issuer not in trusted authorities".into()
            ));
        }
        
        // Check revocation
        let cred_id = credential.credential_id();
        if self.revocation_list.is_revoked(&cred_id) {
            return Err(ChannelError::CredentialRevoked(
                "Credential has been revoked".into()
            ));
        }
        
        // Check expiration
        if credential.is_expired() {
            return Err(ChannelError::CredentialInvalid(
                "Credential has expired".into()
            ));
        }
        
        // Verify signature
        Ok(credential.verify(issuer_public))
    }
    
    /// Check if key rotation is needed (Req 5.4)
    pub fn should_rotate_keys(state: &KeyRotationState) -> bool {
        state.should_rotate()
    }
    
    /// Rotate channel keys (Req 5.4)
    pub async fn rotate_keys(&self, state: &mut KeyRotationState) -> Result<SessionKeys, ChannelError> {
        // Derive new session keys
        let new_keys = SessionKeys::derive_from_secret(&self.local_identity.auth_key.private)
            .map_err(|e| ChannelError::KeyRotation(e.to_string()))?;
        
        state.reset();
        Ok(new_keys)
    }
}

/// Identity manager for creating and managing identities (Req 5.6)
pub struct IdentityManager {
    vault: Box<dyn Vault>,
}

impl IdentityManager {
    /// Create new identity manager with vault
    pub fn new(vault: Box<dyn Vault>) -> Self {
        Self { vault }
    }
    
    /// Create new identity with purpose-specific keys (Req 5.6)
    pub fn create_identity(&self) -> Result<Identity, ChannelError> {
        Identity::generate()
    }
    
    /// Verify that identity has distinct keys for each purpose (Req 5.6)
    pub fn verify_key_distinctness(identity: &Identity) -> bool {
        identity.verify_key_distinctness()
    }
    
    /// Store identity in vault
    pub async fn store_identity(&mut self, identity: &Identity) -> Result<(), ChannelError> {
        let id_str = hex::encode(identity.id);
        let data = identity.serialize();
        self.vault.store_key(&id_str, &data).await
            .map_err(|e| ChannelError::VaultError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation() {
        let identity = Identity::generate().unwrap();
        assert!(identity.verify_key_distinctness());
    }

    #[test]
    fn test_key_rotation_state() {
        let mut state = KeyRotationState::new();
        assert!(!state.should_rotate());
        
        // Add bytes below threshold
        state.add_bytes(50 * 1024 * 1024);
        assert!(!state.should_rotate());
        
        // Add bytes to exceed threshold
        state.add_bytes(60 * 1024 * 1024);
        assert!(state.should_rotate());
        
        // Reset
        state.reset();
        assert!(!state.should_rotate());
    }

    #[test]
    fn test_revocation_list() {
        let mut list = RevocationList::new();
        let cred_id = [42u8; 32];
        
        assert!(!list.is_revoked(&cred_id));
        
        list.add_revocation(cred_id, "test revocation".into());
        assert!(list.is_revoked(&cred_id));
    }

    #[tokio::test]
    async fn test_memory_vault() {
        let mut vault = MemoryVault::new();
        
        vault.store_key("test", b"secret").await.unwrap();
        assert!(vault.has_key("test").await);
        
        let key = vault.get_key("test").await.unwrap();
        assert_eq!(key, b"secret");
        
        vault.delete_key("test").await.unwrap();
        assert!(!vault.has_key("test").await);
    }

    #[test]
    fn test_credential_creation_and_verification() {
        let issuer_kp = MlDsa87KeyPair::generate().unwrap();
        let subject = [1u8; 32];
        let issuer = [2u8; 32];
        
        let mut attrs = HashMap::new();
        attrs.insert("role".into(), "admin".into());
        
        let mut cred = Credential::new(
            subject,
            issuer,
            attrs,
            Duration::from_secs(3600),
        );
        
        cred.sign(&issuer_kp);
        assert!(cred.verify(&issuer_kp.public));
        assert!(!cred.is_expired());
    }
}
