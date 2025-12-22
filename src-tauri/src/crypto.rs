//! Post-Quantum Cryptography Module
//! Implements ML-KEM-1024 (Kyber) + X25519 hybrid key exchange
//! ML-DSA-87 (Dilithium5) signatures
//! ChaCha20-Poly1305 symmetric encryption
//! Based on exemple-alghos/crypto implementation

use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Nonce};
use pqcrypto_mlkem::mlkem1024;
use pqcrypto_dilithium::dilithium5 as mldsa87;
use pqcrypto_traits::kem::{PublicKey as KemPublicKey, SecretKey as KemSecretKey, SharedSecret as KemSharedSecret, Ciphertext as KemCiphertext};
use pqcrypto_traits::sign::{PublicKey as SignPublicKey, SecretKey as SignSecretKey, DetachedSignature};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use x25519_dalek::{StaticSecret, PublicKey as X25519Public};
use hkdf::Hkdf;
use sha2::Sha512;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("encryption failed")]
    Encrypt,
    #[error("decryption failed: {0}")]
    Decrypt(String),
    #[error("key exchange failed: {0}")]
    KeyExchange(String),
    #[error("signature verification failed")]
    SignatureInvalid,
    #[error("key derivation failed")]
    KeyDerivation,
    #[error("key generation failed: {0}")]
    KeyGeneration(String),
}

impl Serialize for CryptoError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// ============================================================================
// Hybrid Keypair (ML-KEM-1024 + X25519 + ML-DSA-87)
// ============================================================================

/// Hybrid keypair combining post-quantum and classical cryptography
#[derive(Clone)]
pub struct HybridKeypair {
    /// ML-KEM-1024 public key (1568 bytes)
    pub pq_public: Vec<u8>,
    /// ML-KEM-1024 secret key (3168 bytes)
    pub pq_secret: Vec<u8>,
    /// X25519 secret key (32 bytes)
    pub x25519_secret: [u8; 32],
    /// X25519 public key (32 bytes)
    pub x25519_public: [u8; 32],
    /// ML-DSA-87 public key for signatures
    pub signing_public: Vec<u8>,
    /// ML-DSA-87 secret key for signatures
    pub signing_secret: Vec<u8>,
}

/// Public bundle for sharing with others
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PublicBundle {
    pub pq: Vec<u8>,
    pub x25519: [u8; 32],
    pub signing: Vec<u8>,
}

/// Encapsulated key for key exchange
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncapsulatedKey {
    pub pq_ciphertext: Vec<u8>,
    pub x25519_ephemeral: [u8; 32],
}

/// Encrypted payload with all necessary data for decryption
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
    pub encap: EncapsulatedKey,
}

impl HybridKeypair {
    /// Generate a new hybrid keypair
    pub fn generate() -> Result<Self, CryptoError> {
        // Generate ML-KEM-1024 keypair
        let (pq_pk, pq_sk) = mlkem1024::keypair();
        
        // Generate X25519 keypair
        let x_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let x_public = X25519Public::from(&x_secret);
        
        // Generate ML-DSA-87 signing keypair
        let (sign_pk, sign_sk) = mldsa87::keypair();
        
        Ok(Self {
            pq_public: pq_pk.as_bytes().to_vec(),
            pq_secret: pq_sk.as_bytes().to_vec(),
            x25519_secret: x_secret.to_bytes(),
            x25519_public: x_public.to_bytes(),
            signing_public: sign_pk.as_bytes().to_vec(),
            signing_secret: sign_sk.as_bytes().to_vec(),
        })
    }

    /// Get public bundle for sharing
    pub fn public_bundle(&self) -> PublicBundle {
        PublicBundle {
            pq: self.pq_public.clone(),
            x25519: self.x25519_public,
            signing: self.signing_public.clone(),
        }
    }

    /// Sign data with ML-DSA-87
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let sk = mldsa87::SecretKey::from_bytes(&self.signing_secret)
            .expect("valid secret key");
        let sig = mldsa87::detached_sign(data, &sk);
        sig.as_bytes().to_vec()
    }

    /// Verify signature
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        let pk = mldsa87::PublicKey::from_bytes(&self.signing_public)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let sig = mldsa87::DetachedSignature::from_bytes(signature)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        mldsa87::verify_detached_signature(&sig, data, &pk)
            .map_err(|_| CryptoError::SignatureInvalid)
    }

    /// Serialize keypair for storage (unencrypted)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(
            4 + self.pq_public.len() +
            4 + self.pq_secret.len() +
            32 + 32 +
            4 + self.signing_public.len() +
            4 + self.signing_secret.len()
        );
        
        // PQ public
        out.extend_from_slice(&(self.pq_public.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.pq_public);
        
        // PQ secret
        out.extend_from_slice(&(self.pq_secret.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.pq_secret);
        
        // X25519 keys
        out.extend_from_slice(&self.x25519_secret);
        out.extend_from_slice(&self.x25519_public);
        
        // Signing keys
        out.extend_from_slice(&(self.signing_public.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.signing_public);
        out.extend_from_slice(&(self.signing_secret.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.signing_secret);
        
        out
    }

    /// Deserialize keypair from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, CryptoError> {
        let mut offset = 0;
        
        // PQ public
        if data.len() < offset + 4 {
            return Err(CryptoError::KeyExchange("data too short".into()));
        }
        let pq_pub_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        let pq_public = data[offset..offset+pq_pub_len].to_vec();
        offset += pq_pub_len;
        
        // PQ secret
        let pq_sec_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        let pq_secret = data[offset..offset+pq_sec_len].to_vec();
        offset += pq_sec_len;
        
        // X25519 keys
        let x25519_secret: [u8; 32] = data[offset..offset+32].try_into()
            .map_err(|_| CryptoError::KeyExchange("invalid x25519 secret".into()))?;
        offset += 32;
        let x25519_public: [u8; 32] = data[offset..offset+32].try_into()
            .map_err(|_| CryptoError::KeyExchange("invalid x25519 public".into()))?;
        offset += 32;
        
        // Signing keys
        let sign_pub_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        let signing_public = data[offset..offset+sign_pub_len].to_vec();
        offset += sign_pub_len;
        
        let sign_sec_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        let signing_secret = data[offset..offset+sign_sec_len].to_vec();
        
        Ok(Self {
            pq_public,
            pq_secret,
            x25519_secret,
            x25519_public,
            signing_public,
            signing_secret,
        })
    }

    /// Encrypt keypair with password using Argon2id + ChaCha20-Poly1305
    pub fn to_encrypted_bytes(&self, password: &[u8]) -> Result<Vec<u8>, CryptoError> {
        use argon2::Argon2;
        
        let plaintext = self.to_bytes();
        
        // Generate random salt
        let mut salt = [0u8; 16];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut salt);
        
        // Derive key using Argon2id
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password, &salt, &mut key)
            .map_err(|_| CryptoError::KeyDerivation)?;
        
        let cipher = ChaCha20Poly1305::new(&key.into());
        let mut nonce_bytes = [0u8; 12];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce_bytes);
        
        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), plaintext.as_ref())
            .map_err(|_| CryptoError::Encrypt)?;
        
        // Format: version(1) + salt(16) + nonce(12) + ciphertext
        let mut out = Vec::with_capacity(1 + 16 + 12 + ciphertext.len());
        out.push(0x01); // Version
        out.extend_from_slice(&salt);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    /// Decrypt keypair from encrypted bytes
    pub fn from_encrypted_bytes(data: &[u8], password: &[u8]) -> Result<Self, CryptoError> {
        use argon2::Argon2;
        
        if data.len() < 45 {
            return Err(CryptoError::Decrypt("data too short".into()));
        }
        
        let version = data[0];
        if version != 0x01 {
            return Err(CryptoError::Decrypt("unsupported version".into()));
        }
        
        let salt: [u8; 16] = data[1..17].try_into().unwrap();
        let nonce_bytes: [u8; 12] = data[17..29].try_into().unwrap();
        let ciphertext = &data[29..];
        
        // Derive key
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password, &salt, &mut key)
            .map_err(|_| CryptoError::KeyDerivation)?;
        
        let cipher = ChaCha20Poly1305::new(&key.into());
        let plaintext = cipher.decrypt(Nonce::from_slice(&nonce_bytes), ciphertext)
            .map_err(|_| CryptoError::Decrypt("wrong password".into()))?;
        
        Self::from_bytes(&plaintext)
    }
}

impl PublicBundle {
    /// Verify signature with this bundle's signing key
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        let pk = mldsa87::PublicKey::from_bytes(&self.signing)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let sig = mldsa87::DetachedSignature::from_bytes(signature)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        mldsa87::verify_detached_signature(&sig, data, &pk)
            .map_err(|_| CryptoError::SignatureInvalid)
    }
}

// ============================================================================
// Hybrid Encryption/Decryption
// ============================================================================

/// Derive hybrid key from PQ and classical shared secrets
fn derive_hybrid_key(pq_ss: &[u8], x25519_ss: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"vortex-hybrid-v1");
    hasher.update(pq_ss);
    hasher.update(x25519_ss);
    *hasher.finalize().as_bytes()
}

/// Encrypt data for a recipient using hybrid encryption
pub fn encrypt(data: &[u8], recipient: &PublicBundle) -> Result<EncryptedPayload, CryptoError> {
    // ML-KEM encapsulation
    let pq_pk = mlkem1024::PublicKey::from_bytes(&recipient.pq)
        .map_err(|_| CryptoError::KeyExchange("invalid PQ public key".into()))?;
    let (pq_ss, pq_ct) = mlkem1024::encapsulate(&pq_pk);
    
    // X25519 key exchange
    let x_ephemeral = StaticSecret::random_from_rng(rand::thread_rng());
    let x_ephemeral_pub = X25519Public::from(&x_ephemeral);
    let x_recipient = X25519Public::from(recipient.x25519);
    let x_ss = x_ephemeral.diffie_hellman(&x_recipient);
    
    // Derive hybrid key
    let key = derive_hybrid_key(pq_ss.as_bytes(), x_ss.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key.into());
    
    // Generate nonce and encrypt
    let mut nonce_bytes = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce_bytes);
    
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), data)
        .map_err(|_| CryptoError::Encrypt)?;
    
    Ok(EncryptedPayload {
        nonce: nonce_bytes,
        ciphertext,
        encap: EncapsulatedKey {
            pq_ciphertext: pq_ct.as_bytes().to_vec(),
            x25519_ephemeral: x_ephemeral_pub.to_bytes(),
        },
    })
}

/// Decrypt data using hybrid decryption
pub fn decrypt(payload: &EncryptedPayload, keypair: &HybridKeypair) -> Result<Vec<u8>, CryptoError> {
    // ML-KEM decapsulation
    let pq_sk = mlkem1024::SecretKey::from_bytes(&keypair.pq_secret)
        .map_err(|_| CryptoError::KeyExchange("invalid PQ secret key".into()))?;
    let pq_ct = mlkem1024::Ciphertext::from_bytes(&payload.encap.pq_ciphertext)
        .map_err(|_| CryptoError::KeyExchange("invalid PQ ciphertext".into()))?;
    let pq_ss = mlkem1024::decapsulate(&pq_ct, &pq_sk);
    
    // X25519 key exchange
    let x_secret = StaticSecret::from(keypair.x25519_secret);
    let x_ephemeral = X25519Public::from(payload.encap.x25519_ephemeral);
    let x_ss = x_secret.diffie_hellman(&x_ephemeral);
    
    // Derive hybrid key
    let key = derive_hybrid_key(pq_ss.as_bytes(), x_ss.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key.into());
    
    cipher.decrypt(Nonce::from_slice(&payload.nonce), payload.ciphertext.as_ref())
        .map_err(|_| CryptoError::Decrypt("authentication failed".into()))
}

// ============================================================================
// Session Keys (HKDF-SHA512)
// ============================================================================

/// Derived session keys for symmetric encryption
#[derive(Clone)]
pub struct SessionKeys {
    pub encryption_key: [u8; 32],
    pub hmac_key: [u8; 32],
    pub iv: [u8; 12],
}

impl SessionKeys {
    /// Derive session keys from shared secret
    pub fn derive_from_secret(shared_secret: &[u8]) -> Result<Self, CryptoError> {
        let hk = Hkdf::<Sha512>::new(Some(b"vortex-session-v1"), shared_secret);
        
        let mut encryption_key = [0u8; 32];
        let mut hmac_key = [0u8; 32];
        let mut iv = [0u8; 12];
        
        hk.expand(b"encryption", &mut encryption_key)
            .map_err(|_| CryptoError::KeyDerivation)?;
        hk.expand(b"hmac", &mut hmac_key)
            .map_err(|_| CryptoError::KeyDerivation)?;
        hk.expand(b"iv", &mut iv)
            .map_err(|_| CryptoError::KeyDerivation)?;
        
        Ok(Self { encryption_key, hmac_key, iv })
    }
}

// ============================================================================
// Simple Symmetric Encryption (for local data)
// ============================================================================

/// Encrypt data with password (Argon2id + ChaCha20-Poly1305)
pub fn encrypt_with_password(data: &[u8], password: &[u8]) -> Result<Vec<u8>, CryptoError> {
    use argon2::Argon2;
    
    let mut salt = [0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut salt);
    
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password, &salt, &mut key)
        .map_err(|_| CryptoError::KeyDerivation)?;
    
    let cipher = ChaCha20Poly1305::new(&key.into());
    let mut nonce_bytes = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce_bytes);
    
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), data)
        .map_err(|_| CryptoError::Encrypt)?;
    
    // Format: salt(16) + nonce(12) + ciphertext
    let mut out = Vec::with_capacity(16 + 12 + ciphertext.len());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt data with password
pub fn decrypt_with_password(data: &[u8], password: &[u8]) -> Result<Vec<u8>, CryptoError> {
    use argon2::Argon2;
    
    if data.len() < 28 {
        return Err(CryptoError::Decrypt("data too short".into()));
    }
    
    let salt: [u8; 16] = data[..16].try_into().unwrap();
    let nonce_bytes: [u8; 12] = data[16..28].try_into().unwrap();
    let ciphertext = &data[28..];
    
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password, &salt, &mut key)
        .map_err(|_| CryptoError::KeyDerivation)?;
    
    let cipher = ChaCha20Poly1305::new(&key.into());
    cipher.decrypt(Nonce::from_slice(&nonce_bytes), ciphertext)
        .map_err(|_| CryptoError::Decrypt("wrong password".into()))
}

/// Hash data with BLAKE3
pub fn hash_data(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

// ============================================================================
// Secure Token Storage (Machine-bound encryption)
// ============================================================================

/// Get machine-specific key material for token encryption
fn get_machine_key() -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"vortex-machine-key-v1");
    
    // Use machine ID if available
    if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
        hasher.update(id.trim().as_bytes());
    } else if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
        hasher.update(id.trim().as_bytes());
    } else {
        // Fallback: use hostname + username
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            hasher.update(hostname.as_bytes());
        }
        if let Ok(user) = std::env::var("USER") {
            hasher.update(user.as_bytes());
        }
    }
    
    // Add app-specific salt
    hasher.update(b"vortex-image-secure-storage");
    
    *hasher.finalize().as_bytes()
}

/// Encrypt sensitive token for secure storage (machine-bound)
pub fn encrypt_token(token: &str) -> Result<Vec<u8>, CryptoError> {
    let key = get_machine_key();
    let cipher = ChaCha20Poly1305::new(&key.into());
    
    let mut nonce_bytes = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce_bytes);
    
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), token.as_bytes())
        .map_err(|_| CryptoError::Encrypt)?;
    
    // Format: version(1) + nonce(12) + ciphertext
    let mut out = Vec::with_capacity(1 + 12 + ciphertext.len());
    out.push(0x02); // Version 2 for machine-bound
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt token from secure storage
pub fn decrypt_token(data: &[u8]) -> Result<String, CryptoError> {
    if data.len() < 14 {
        return Err(CryptoError::Decrypt("data too short".into()));
    }
    
    let version = data[0];
    if version != 0x02 {
        return Err(CryptoError::Decrypt("unsupported token version".into()));
    }
    
    let nonce_bytes: [u8; 12] = data[1..13].try_into().unwrap();
    let ciphertext = &data[13..];
    
    let key = get_machine_key();
    let cipher = ChaCha20Poly1305::new(&key.into());
    
    let plaintext = cipher.decrypt(Nonce::from_slice(&nonce_bytes), ciphertext)
        .map_err(|_| CryptoError::Decrypt("token decryption failed - may be from different machine".into()))?;
    
    String::from_utf8(plaintext)
        .map_err(|_| CryptoError::Decrypt("invalid token data".into()))
}

// ============================================================================
// Per-Item Encryption Settings
// ============================================================================

/// Encryption settings for photos/albums
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptionSettings {
    pub enabled: bool,
    pub use_password: bool,
    pub use_keypair: bool,
    pub recipient_bundle: Option<PublicBundle>,
}

impl Default for EncryptionSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            use_password: false,
            use_keypair: false,
            recipient_bundle: None,
        }
    }
}

/// Encrypt file data with settings
pub fn encrypt_file_data(
    data: &[u8],
    settings: &EncryptionSettings,
    password: Option<&str>,
    _keypair_bytes: Option<&[u8]>,
) -> Result<EncryptedFileData, CryptoError> {
    if !settings.enabled {
        return Ok(EncryptedFileData {
            data: data.to_vec(),
            encrypted: false,
            method: EncryptionMethod::None,
            metadata: None,
        });
    }
    
    if settings.use_password {
        let pwd = password.ok_or(CryptoError::Decrypt("password required".into()))?;
        let encrypted = encrypt_with_password(data, pwd.as_bytes())?;
        return Ok(EncryptedFileData {
            data: encrypted,
            encrypted: true,
            method: EncryptionMethod::Password,
            metadata: None,
        });
    }
    
    if settings.use_keypair {
        let bundle = settings.recipient_bundle.as_ref()
            .ok_or(CryptoError::Decrypt("recipient bundle required".into()))?;
        let payload = encrypt(data, bundle)?;
        let serialized = serde_json::to_vec(&payload)
            .map_err(|_| CryptoError::Encrypt)?;
        return Ok(EncryptedFileData {
            data: serialized,
            encrypted: true,
            method: EncryptionMethod::HybridPQ,
            metadata: None,
        });
    }
    
    Err(CryptoError::Decrypt("no encryption method specified".into()))
}

/// Decrypt file data
pub fn decrypt_file_data(
    encrypted: &EncryptedFileData,
    password: Option<&str>,
    keypair_bytes: Option<&[u8]>,
) -> Result<Vec<u8>, CryptoError> {
    if !encrypted.encrypted {
        return Ok(encrypted.data.clone());
    }
    
    match encrypted.method {
        EncryptionMethod::None => Ok(encrypted.data.clone()),
        EncryptionMethod::Password => {
            let pwd = password.ok_or(CryptoError::Decrypt("password required".into()))?;
            decrypt_with_password(&encrypted.data, pwd.as_bytes())
        }
        EncryptionMethod::HybridPQ => {
            let kp_bytes = keypair_bytes.ok_or(CryptoError::Decrypt("keypair required".into()))?;
            let keypair = HybridKeypair::from_bytes(kp_bytes)?;
            let payload: EncryptedPayload = serde_json::from_slice(&encrypted.data)
                .map_err(|_| CryptoError::Decrypt("invalid encrypted payload".into()))?;
            decrypt(&payload, &keypair)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EncryptionMethod {
    None,
    Password,
    HybridPQ,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedFileData {
    pub data: Vec<u8>,
    pub encrypted: bool,
    pub method: EncryptionMethod,
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// Tauri Commands
// ============================================================================

use crate::github::AppError;

/// Generate a new hybrid keypair and return serialized form
#[tauri::command]
pub async fn generate_keypair() -> Result<KeypairResult, AppError> {
    let keypair = HybridKeypair::generate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(KeypairResult {
        public_bundle: keypair.public_bundle(),
        keypair_bytes: keypair.to_bytes(),
    })
}

/// Result containing both public bundle and serialized keypair
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeypairResult {
    pub public_bundle: PublicBundle,
    pub keypair_bytes: Vec<u8>,
}

/// Encrypt data with password
#[tauri::command]
pub async fn encrypt_data_password(
    data: Vec<u8>,
    password: String,
) -> Result<Vec<u8>, AppError> {
    encrypt_with_password(&data, password.as_bytes())
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// Decrypt data with password
#[tauri::command]
pub async fn decrypt_data_password(
    data: Vec<u8>,
    password: String,
) -> Result<Vec<u8>, AppError> {
    decrypt_with_password(&data, password.as_bytes())
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// Encrypt keypair with password for secure storage
#[tauri::command]
pub async fn encrypt_keypair(
    keypair_bytes: Vec<u8>,
    password: String,
) -> Result<Vec<u8>, AppError> {
    let keypair = HybridKeypair::from_bytes(&keypair_bytes)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    keypair.to_encrypted_bytes(password.as_bytes())
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// Decrypt keypair from encrypted storage
#[tauri::command]
pub async fn decrypt_keypair(
    encrypted_bytes: Vec<u8>,
    password: String,
) -> Result<KeypairResult, AppError> {
    let keypair = HybridKeypair::from_encrypted_bytes(&encrypted_bytes, password.as_bytes())
        .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(KeypairResult {
        public_bundle: keypair.public_bundle(),
        keypair_bytes: keypair.to_bytes(),
    })
}

/// Encrypt data for a recipient using hybrid post-quantum encryption
#[tauri::command]
pub async fn encrypt_hybrid(
    data: Vec<u8>,
    recipient_bundle: PublicBundle,
) -> Result<Vec<u8>, AppError> {
    let payload = encrypt(&data, &recipient_bundle)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    // Serialize the payload
    serde_json::to_vec(&payload)
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// Decrypt data using hybrid post-quantum decryption
#[tauri::command]
pub async fn decrypt_hybrid(
    encrypted_data: Vec<u8>,
    keypair_bytes: Vec<u8>,
) -> Result<Vec<u8>, AppError> {
    let payload: EncryptedPayload = serde_json::from_slice(&encrypted_data)
        .map_err(|e| AppError::Validation(format!("Invalid encrypted data: {}", e)))?;
    let keypair = HybridKeypair::from_bytes(&keypair_bytes)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    decrypt(&payload, &keypair)
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// Sign data with ML-DSA-87 (Dilithium5)
#[tauri::command]
pub async fn sign_data(
    data: Vec<u8>,
    keypair_bytes: Vec<u8>,
) -> Result<Vec<u8>, AppError> {
    let keypair = HybridKeypair::from_bytes(&keypair_bytes)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(keypair.sign(&data))
}

/// Verify signature with ML-DSA-87 (Dilithium5)
#[tauri::command]
pub async fn verify_signature(
    data: Vec<u8>,
    signature: Vec<u8>,
    public_bundle: PublicBundle,
) -> Result<bool, AppError> {
    match public_bundle.verify(&data, &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Derive session keys from shared secret (for advanced use cases)
#[tauri::command]
pub async fn derive_session_keys(
    shared_secret: Vec<u8>,
) -> Result<SessionKeysResult, AppError> {
    let keys = SessionKeys::derive_from_secret(&shared_secret)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(SessionKeysResult {
        encryption_key: keys.encryption_key.to_vec(),
        hmac_key: keys.hmac_key.to_vec(),
        iv: keys.iv.to_vec(),
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionKeysResult {
    pub encryption_key: Vec<u8>,
    pub hmac_key: Vec<u8>,
    pub iv: Vec<u8>,
}

/// Hash data with BLAKE3
#[tauri::command]
pub fn hash_data_blake3(data: Vec<u8>) -> Vec<u8> {
    hash_data(&data).to_vec()
}

/// Get crypto algorithm info
#[tauri::command]
pub fn get_crypto_info() -> serde_json::Value {
    serde_json::json!({
        "key_exchange": "ML-KEM-1024 (Kyber) + X25519 Hybrid",
        "signatures": "ML-DSA-87 (Dilithium5)",
        "symmetric": "ChaCha20-Poly1305",
        "kdf": "Argon2id + HKDF-SHA512",
        "hash": "BLAKE3",
        "pq_security_level": "NIST Level 5 (256-bit)",
        "classical_security_level": "128-bit equivalent",
        "features": [
            "Post-quantum key exchange (ML-KEM-1024)",
            "Classical key exchange (X25519)",
            "Hybrid key derivation (BLAKE3)",
            "Post-quantum signatures (ML-DSA-87)",
            "Password-based encryption (Argon2id + ChaCha20-Poly1305)",
            "Session key derivation (HKDF-SHA512)",
            "Machine-bound token encryption",
            "Per-file encryption settings"
        ]
    })
}

/// Securely encrypt a token for storage (machine-bound)
#[tauri::command]
pub async fn secure_store_token(token: String) -> Result<Vec<u8>, AppError> {
    encrypt_token(&token)
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// Decrypt a securely stored token
#[tauri::command]
pub async fn secure_retrieve_token(encrypted: Vec<u8>) -> Result<String, AppError> {
    decrypt_token(&encrypted)
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// Encrypt file data with specified settings
#[tauri::command]
pub async fn encrypt_file(
    data: Vec<u8>,
    settings: EncryptionSettings,
    password: Option<String>,
    keypair_bytes: Option<Vec<u8>>,
) -> Result<EncryptedFileData, AppError> {
    encrypt_file_data(
        &data,
        &settings,
        password.as_deref(),
        keypair_bytes.as_deref(),
    )
    .map_err(|e| AppError::Validation(e.to_string()))
}

/// Decrypt file data
#[tauri::command]
pub async fn decrypt_file(
    encrypted: EncryptedFileData,
    password: Option<String>,
    keypair_bytes: Option<Vec<u8>>,
) -> Result<Vec<u8>, AppError> {
    decrypt_file_data(
        &encrypted,
        password.as_deref(),
        keypair_bytes.as_deref(),
    )
    .map_err(|e| AppError::Validation(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = HybridKeypair::generate().unwrap();
        assert_eq!(kp.pq_public.len(), 1568);
        assert_eq!(kp.pq_secret.len(), 3168);
        assert_eq!(kp.x25519_public.len(), 32);
        assert_eq!(kp.x25519_secret.len(), 32);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let bob = HybridKeypair::generate().unwrap();
        let message = b"Post-quantum secure message!";
        let encrypted = encrypt(message, &bob.public_bundle()).unwrap();
        let decrypted = decrypt(&encrypted, &bob).unwrap();
        assert_eq!(message.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_wrong_key_fails() {
        let alice = HybridKeypair::generate().unwrap();
        let bob = HybridKeypair::generate().unwrap();
        let encrypted = encrypt(b"secret", &bob.public_bundle()).unwrap();
        assert!(decrypt(&encrypted, &alice).is_err());
    }

    #[test]
    fn test_sign_verify() {
        let kp = HybridKeypair::generate().unwrap();
        let data = b"message to sign";
        let signature = kp.sign(data);
        assert!(kp.verify(data, &signature).is_ok());
        assert!(kp.public_bundle().verify(data, &signature).is_ok());
    }

    #[test]
    fn test_keypair_serialization() {
        let kp = HybridKeypair::generate().unwrap();
        let bytes = kp.to_bytes();
        let restored = HybridKeypair::from_bytes(&bytes).unwrap();
        assert_eq!(kp.pq_public, restored.pq_public);
        assert_eq!(kp.x25519_public, restored.x25519_public);
    }

    #[test]
    fn test_encrypted_keypair() {
        let kp = HybridKeypair::generate().unwrap();
        let password = b"secure_password";
        let encrypted = kp.to_encrypted_bytes(password).unwrap();
        let restored = HybridKeypair::from_encrypted_bytes(&encrypted, password).unwrap();
        assert_eq!(kp.pq_public, restored.pq_public);
    }

    #[test]
    fn test_password_encryption() {
        let data = b"secret data";
        let password = b"password123";
        let encrypted = encrypt_with_password(data, password).unwrap();
        let decrypted = decrypt_with_password(&encrypted, password).unwrap();
        assert_eq!(data.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_wrong_password_fails() {
        let data = b"secret";
        let encrypted = encrypt_with_password(data, b"correct").unwrap();
        assert!(decrypt_with_password(&encrypted, b"wrong").is_err());
    }
}
