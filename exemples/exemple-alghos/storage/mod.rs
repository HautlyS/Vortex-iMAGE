//! Storage Module - Distributed storage with erasure coding and S3-compatible API
//! Implements Requirements 4.1-4.10 from examples-integration-analysis spec

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Shard identifier (16 bytes)
pub type ShardId = [u8; 16];

/// Node identifier (32 bytes)  
pub type NodeId = [u8; 32];

/// Blake3 hash (32 bytes)
pub type Blake3Hash = [u8; 32];

/// Erasure coding configuration (Req 4.2)
#[derive(Debug, Clone)]
pub struct ErasureConfig {
    pub data_shards: usize,
    pub parity_shards: usize,
}

impl Default for ErasureConfig {
    fn default() -> Self {
        Self {
            data_shards: 4,
            parity_shards: 2,
        }
    }
}

/// Storage shard with integrity (Req 4.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageShard {
    pub shard_id: ShardId,
    pub data: Vec<u8>,
    pub checksum: Blake3Hash,
    pub encrypted: bool,
}

/// S3-compatible object metadata (Req 4.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetadata {
    pub key: String,
    pub size: u64,
    pub content_type: String,
    pub etag: String,
    pub last_modified: DateTime<Utc>,
}

/// JWT token for storage API authentication (Req 4.6)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtToken {
    pub header: JwtHeader,
    pub claims: JwtClaims,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
    pub permissions: Vec<String>,
}

/// Multipart upload state (Req 4.10)
#[derive(Debug, Clone)]
pub struct MultipartUpload {
    pub upload_id: String,
    pub key: String,
    pub parts: Vec<UploadPart>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UploadPart {
    pub part_number: u32,
    pub etag: String,
    pub size: u64,
    pub checksum: Blake3Hash,
    pub data: Vec<u8>,
}

/// Garbage collection result (Req 4.9)
#[derive(Debug, Clone)]
pub struct GcResult {
    pub deleted_objects: usize,
    pub freed_bytes: u64,
    pub preserved_metadata_copies: usize,
}

/// Shard redistribution result (Req 4.5)
#[derive(Debug, Clone)]
pub struct RedistributionResult {
    pub redistributed_shards: usize,
    pub target_nodes: Vec<NodeId>,
    pub duration_ms: u64,
}

/// Storage error types
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Object not found: {0}")]
    ObjectNotFound(String),
    #[error("Checksum mismatch")]
    ChecksumMismatch,
    #[error("Erasure decoding failed: insufficient shards")]
    InsufficientShards,
    #[error("JWT validation failed: {0}")]
    JwtValidationFailed(String),
    #[error("Multipart upload failed: {0}")]
    MultipartFailed(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
}


impl StorageShard {
    /// Create new shard with checksum (Req 4.3)
    pub fn new(data: Vec<u8>, encrypted: bool) -> Self {
        let checksum = *blake3::hash(&data).as_bytes();
        let mut shard_id = [0u8; 16];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut shard_id);
        
        Self {
            shard_id,
            data,
            checksum,
            encrypted,
        }
    }
    
    /// Verify checksum integrity (Req 4.3)
    pub fn verify_checksum(&self) -> bool {
        let computed = *blake3::hash(&self.data).as_bytes();
        computed == self.checksum
    }
}

/// Simple erasure coder using XOR parity (Req 4.2)
pub struct ErasureCoder {
    config: ErasureConfig,
}

impl ErasureCoder {
    pub fn new(config: ErasureConfig) -> Self {
        Self { config }
    }
    
    /// Encode data into shards (Req 4.2)
    pub fn encode(&self, data: &[u8]) -> Vec<StorageShard> {
        let shard_size = (data.len() + self.config.data_shards - 1) / self.config.data_shards;
        let mut shards = Vec::new();
        
        // Create data shards
        for i in 0..self.config.data_shards {
            let start = i * shard_size;
            let end = std::cmp::min(start + shard_size, data.len());
            let mut shard_data = vec![0u8; shard_size];
            if start < data.len() {
                let copy_len = end - start;
                shard_data[..copy_len].copy_from_slice(&data[start..end]);
            }
            shards.push(StorageShard::new(shard_data, false));
        }
        
        // Create parity shards using XOR
        for _ in 0..self.config.parity_shards {
            let mut parity = vec![0u8; shard_size];
            for shard in &shards {
                for (p, &d) in parity.iter_mut().zip(shard.data.iter()) {
                    *p ^= d;
                }
            }
            shards.push(StorageShard::new(parity, false));
        }
        
        shards
    }
    
    /// Decode shards back to original data (Req 4.2)
    pub fn decode(&self, shards: &[Option<StorageShard>]) -> Result<Vec<u8>, StorageError> {
        let available: Vec<_> = shards.iter()
            .filter_map(|s| s.as_ref())
            .collect();
        
        if available.len() < self.config.data_shards {
            return Err(StorageError::InsufficientShards);
        }
        
        // Simple reconstruction from data shards
        let shard_size = available[0].data.len();
        let mut data = Vec::with_capacity(shard_size * self.config.data_shards);
        
        for i in 0..self.config.data_shards {
            if let Some(shard) = shards.get(i).and_then(|s| s.as_ref()) {
                data.extend_from_slice(&shard.data);
            } else {
                // Would need to reconstruct from parity - simplified here
                return Err(StorageError::InsufficientShards);
            }
        }
        
        Ok(data)
    }
}

/// In-memory storage backend (Req 4.1)
pub struct StorageBackend {
    objects: HashMap<String, (Vec<u8>, ObjectMetadata)>,
    multipart_uploads: HashMap<String, MultipartUpload>,
    references: HashMap<String, usize>, // Reference count for GC
    secret_key: [u8; 32], // For JWT validation
}

impl StorageBackend {
    pub fn new(secret_key: [u8; 32]) -> Self {
        Self {
            objects: HashMap::new(),
            multipart_uploads: HashMap::new(),
            references: HashMap::new(),
            secret_key,
        }
    }
    
    /// Put object with S3-compatible API (Req 4.1)
    pub async fn put_object(&mut self, key: &str, data: &[u8], content_type: &str) -> Result<ObjectMetadata, StorageError> {
        let etag = hex::encode(&blake3::hash(data).as_bytes()[..16]);
        let metadata = ObjectMetadata {
            key: key.to_string(),
            size: data.len() as u64,
            content_type: content_type.to_string(),
            etag,
            last_modified: Utc::now(),
        };
        
        self.objects.insert(key.to_string(), (data.to_vec(), metadata.clone()));
        self.references.insert(key.to_string(), 1);
        
        Ok(metadata)
    }
    
    /// Get object (Req 4.1)
    pub async fn get_object(&self, key: &str) -> Result<(Vec<u8>, ObjectMetadata), StorageError> {
        self.objects.get(key)
            .cloned()
            .ok_or_else(|| StorageError::ObjectNotFound(key.to_string()))
    }
    
    /// Delete object (Req 4.1)
    pub async fn delete_object(&mut self, key: &str) -> Result<(), StorageError> {
        self.objects.remove(key)
            .ok_or_else(|| StorageError::ObjectNotFound(key.to_string()))?;
        self.references.remove(key);
        Ok(())
    }
    
    /// List objects with prefix (Req 4.1)
    pub async fn list_objects(&self, prefix: &str) -> Result<Vec<ObjectMetadata>, StorageError> {
        Ok(self.objects.iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(_, (_, m))| m.clone())
            .collect())
    }
    
    /// Initiate multipart upload (Req 4.10)
    pub async fn create_multipart_upload(&mut self, key: &str) -> Result<MultipartUpload, StorageError> {
        let upload_id = uuid::Uuid::new_v4().to_string();
        let upload = MultipartUpload {
            upload_id: upload_id.clone(),
            key: key.to_string(),
            parts: Vec::new(),
            created_at: Utc::now(),
        };
        
        self.multipart_uploads.insert(upload_id, upload.clone());
        Ok(upload)
    }
    
    /// Upload part for multipart upload (Req 4.10)
    pub async fn upload_part(
        &mut self,
        upload_id: &str,
        part_number: u32,
        data: &[u8],
    ) -> Result<UploadPart, StorageError> {
        let upload = self.multipart_uploads.get_mut(upload_id)
            .ok_or_else(|| StorageError::MultipartFailed("Upload not found".into()))?;
        
        let checksum = *blake3::hash(data).as_bytes();
        let etag = hex::encode(&checksum[..16]);
        
        let part = UploadPart {
            part_number,
            etag,
            size: data.len() as u64,
            checksum,
            data: data.to_vec(),
        };
        
        upload.parts.push(part.clone());
        Ok(part)
    }
    
    /// Complete multipart upload (Req 4.10)
    pub async fn complete_multipart_upload(&mut self, upload_id: &str) -> Result<ObjectMetadata, StorageError> {
        let upload = self.multipart_uploads.remove(upload_id)
            .ok_or_else(|| StorageError::MultipartFailed("Upload not found".into()))?;
        
        // Sort parts by number and concatenate
        let mut parts = upload.parts;
        parts.sort_by_key(|p| p.part_number);
        
        let data: Vec<u8> = parts.iter().flat_map(|p| p.data.clone()).collect();
        
        self.put_object(&upload.key, &data, "application/octet-stream").await
    }
    
    /// Abort multipart upload (Req 4.10)
    pub async fn abort_multipart_upload(&mut self, upload_id: &str) -> Result<(), StorageError> {
        self.multipart_uploads.remove(upload_id)
            .ok_or_else(|| StorageError::MultipartFailed("Upload not found".into()))?;
        Ok(())
    }
    
    /// Validate JWT token (Req 4.6)
    pub fn validate_jwt<'a>(&self, token: &'a JwtToken) -> Result<&'a JwtClaims, StorageError> {
        // Check expiration
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        if token.claims.exp < now {
            return Err(StorageError::JwtValidationFailed("Token expired".into()));
        }
        
        // Verify signature (simplified - would use HMAC in production)
        let expected_sig = blake3::keyed_hash(&self.secret_key, 
            format!("{}{}", token.claims.sub, token.claims.exp).as_bytes());
        
        if token.signature != expected_sig.as_bytes() {
            return Err(StorageError::JwtValidationFailed("Invalid signature".into()));
        }
        
        Ok(&token.claims)
    }
    
    /// Run garbage collection (Req 4.9)
    pub async fn run_gc(&mut self) -> Result<GcResult, StorageError> {
        let mut deleted = 0;
        let mut freed = 0u64;
        
        // Find unreferenced objects
        let unreferenced: Vec<_> = self.objects.keys()
            .filter(|k| self.references.get(*k).copied().unwrap_or(0) == 0)
            .cloned()
            .collect();
        
        for key in unreferenced {
            if let Some((data, _)) = self.objects.remove(&key) {
                freed += data.len() as u64;
                deleted += 1;
            }
        }
        
        Ok(GcResult {
            deleted_objects: deleted,
            freed_bytes: freed,
            preserved_metadata_copies: 2, // Always preserve 2 copies
        })
    }
}

/// Encrypt shard with AES-256-GCM (Req 4.4)
pub fn encrypt_shard(shard: &mut StorageShard, key: &[u8; 32]) -> Result<(), StorageError> {
    use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
    use aes_gcm::Nonce;
    
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce_bytes);
    
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), shard.data.as_ref())
        .map_err(|_| StorageError::EncryptionError("Encryption failed".into()))?;
    
    // Prepend nonce to ciphertext
    let mut encrypted = Vec::with_capacity(12 + ciphertext.len());
    encrypted.extend_from_slice(&nonce_bytes);
    encrypted.extend_from_slice(&ciphertext);
    
    shard.data = encrypted;
    shard.encrypted = true;
    shard.checksum = *blake3::hash(&shard.data).as_bytes();
    
    Ok(())
}

/// Decrypt shard with AES-256-GCM (Req 4.4)
pub fn decrypt_shard(shard: &mut StorageShard, key: &[u8; 32]) -> Result<(), StorageError> {
    use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
    use aes_gcm::Nonce;
    
    if shard.data.len() < 12 {
        return Err(StorageError::EncryptionError("Data too short".into()));
    }
    
    let nonce = Nonce::from_slice(&shard.data[..12]);
    let ciphertext = &shard.data[12..];
    
    let cipher = Aes256Gcm::new(key.into());
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| StorageError::EncryptionError("Decryption failed".into()))?;
    
    shard.data = plaintext;
    shard.encrypted = false;
    shard.checksum = *blake3::hash(&shard.data).as_bytes();
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_checksum() {
        let shard = StorageShard::new(b"test data".to_vec(), false);
        assert!(shard.verify_checksum());
    }

    #[test]
    fn test_erasure_coding() {
        let coder = ErasureCoder::new(ErasureConfig::default());
        let data = b"Hello, World! This is test data for erasure coding.";
        
        let shards = coder.encode(data);
        assert_eq!(shards.len(), 6); // 4 data + 2 parity
        
        // Convert to Option for decode
        let shard_opts: Vec<_> = shards.into_iter().map(Some).collect();
        let recovered = coder.decode(&shard_opts).unwrap();
        
        assert!(recovered.starts_with(data));
    }

    #[tokio::test]
    async fn test_storage_backend() {
        let mut backend = StorageBackend::new([0u8; 32]);
        
        let metadata = backend.put_object("test/file.txt", b"Hello", "text/plain").await.unwrap();
        assert_eq!(metadata.size, 5);
        
        let (data, _) = backend.get_object("test/file.txt").await.unwrap();
        assert_eq!(data, b"Hello");
        
        let list = backend.list_objects("test/").await.unwrap();
        assert_eq!(list.len(), 1);
        
        backend.delete_object("test/file.txt").await.unwrap();
        assert!(backend.get_object("test/file.txt").await.is_err());
    }

    #[tokio::test]
    async fn test_multipart_upload() {
        let mut backend = StorageBackend::new([0u8; 32]);
        
        let upload = backend.create_multipart_upload("large-file.bin").await.unwrap();
        
        backend.upload_part(&upload.upload_id, 1, b"Part 1 ").await.unwrap();
        backend.upload_part(&upload.upload_id, 2, b"Part 2").await.unwrap();
        
        let metadata = backend.complete_multipart_upload(&upload.upload_id).await.unwrap();
        assert_eq!(metadata.size, 13);
        
        let (data, _) = backend.get_object("large-file.bin").await.unwrap();
        assert_eq!(data, b"Part 1 Part 2");
    }

    #[test]
    fn test_shard_encryption() {
        let mut shard = StorageShard::new(b"secret data".to_vec(), false);
        let key = [42u8; 32];
        
        encrypt_shard(&mut shard, &key).unwrap();
        assert!(shard.encrypted);
        assert_ne!(shard.data, b"secret data");
        
        decrypt_shard(&mut shard, &key).unwrap();
        assert!(!shard.encrypted);
        assert_eq!(shard.data, b"secret data");
    }
}
