//! Enhanced Post-Quantum Cryptography Module
//! Implements FIPS 203/204 compliant algorithms with key cycling and HMAC verification
//! Based on Requirements 1.1-1.10 from examples-integration-analysis spec

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::thread;
use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Nonce};
use hkdf::Hkdf;
use hmac::Hmac;
use sha2::Sha512;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use pqcrypto_mlkem::mlkem1024;
use pqcrypto_dilithium::dilithium5 as mldsa87;
use pqcrypto_traits::kem::{PublicKey as KemPublicKey, SecretKey as KemSecretKey, SharedSecret as KemSharedSecret};
use pqcrypto_traits::sign::{PublicKey as SignPublicKey, SecretKey as SignSecretKey, DetachedSignature};
use x25519_dalek::{StaticSecret, PublicKey as X25519PublicKey};

type HmacSha512 = Hmac<Sha512>;

/// Stack size for PQ key generation thread (8MB as per Req 1.6)
const PQ_KEY_GEN_STACK_SIZE: usize = 8 * 1024 * 1024;

/// ML-KEM-1024 Public Key wrapper
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MlKem1024PublicKey(pub Vec<u8>);

/// ML-KEM-1024 Private Key wrapper
#[derive(Clone)]
pub struct MlKem1024PrivateKey(pub Vec<u8>);

/// Post-quantum key pair for ML-KEM-1024 (Req 1.1)
#[derive(Clone)]
pub struct PQKeyPair {
    pub public: MlKem1024PublicKey,
    pub private: MlKem1024PrivateKey,
}

/// X25519 key pair for classical key exchange
#[derive(Clone)]
pub struct X25519KeyPair {
    pub public: [u8; 32],
    pub private: [u8; 32],
}

/// Hybrid key exchange combining ML-KEM and X25519 (Req 1.1)
#[derive(Clone)]
pub struct HybridKeyExchange {
    pub pq_keypair: PQKeyPair,
    pub classical_keypair: X25519KeyPair,
}

/// Shared secret from key exchange
pub struct SharedSecret(pub Vec<u8>);

impl MlKem1024PublicKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl MlKem1024PrivateKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl PQKeyPair {
    /// Generate ML-KEM-1024 keypair on dedicated thread with 8MB stack (Req 1.6)
    pub fn generate() -> Result<Self, EnhancedCryptoError> {
        let handle = thread::Builder::new()
            .stack_size(PQ_KEY_GEN_STACK_SIZE)
            .spawn(|| {
                let (pk, sk) = mlkem1024::keypair();
                (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
            })
            .map_err(|e| EnhancedCryptoError::KeyGeneration(e.to_string()))?;
        
        let (pk_bytes, sk_bytes) = handle.join()
            .map_err(|_| EnhancedCryptoError::KeyGeneration("thread panicked".into()))?;
        
        Ok(Self {
            public: MlKem1024PublicKey(pk_bytes),
            private: MlKem1024PrivateKey(sk_bytes),
        })
    }
    
    /// Serialize keypair for storage (Req 1.7)
    pub fn serialize(&self) -> Result<Vec<u8>, EnhancedCryptoError> {
        let mut out = Vec::with_capacity(self.public.0.len() + self.private.0.len() + 8);
        out.extend_from_slice(&(self.public.0.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.public.0);
        out.extend_from_slice(&(self.private.0.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.private.0);
        Ok(out)
    }
    
    /// Deserialize keypair from bytes (Req 1.8)
    pub fn deserialize(data: &[u8]) -> Result<Self, EnhancedCryptoError> {
        if data.len() < 8 {
            return Err(EnhancedCryptoError::Decryption("data too short".into()));
        }
        
        let mut offset = 0;
        let pk_len = u32::from_le_bytes(data[offset..offset+4].try_into()
            .map_err(|_| EnhancedCryptoError::Decryption("invalid pk length".into()))?) as usize;
        offset += 4;
        
        if data.len() < offset + pk_len + 4 {
            return Err(EnhancedCryptoError::Decryption("data too short for pk".into()));
        }
        let pk_bytes = data[offset..offset+pk_len].to_vec();
        offset += pk_len;
        
        let sk_len = u32::from_le_bytes(data[offset..offset+4].try_into()
            .map_err(|_| EnhancedCryptoError::Decryption("invalid sk length".into()))?) as usize;
        offset += 4;
        
        if data.len() < offset + sk_len {
            return Err(EnhancedCryptoError::Decryption("data too short for sk".into()));
        }
        let sk_bytes = data[offset..offset+sk_len].to_vec();
        
        Ok(Self {
            public: MlKem1024PublicKey(pk_bytes),
            private: MlKem1024PrivateKey(sk_bytes),
        })
    }
}

impl X25519KeyPair {
    /// Generate X25519 keypair
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(rand::thread_rng());
        let public = X25519PublicKey::from(&secret);
        Self {
            public: public.to_bytes(),
            private: secret.to_bytes(),
        }
    }
}

impl HybridKeyExchange {
    /// Generate hybrid keypair (ML-KEM + X25519) (Req 1.1)
    pub fn generate() -> Result<Self, EnhancedCryptoError> {
        Ok(Self {
            pq_keypair: PQKeyPair::generate()?,
            classical_keypair: X25519KeyPair::generate(),
        })
    }
    
    /// Perform hybrid key exchange (Req 1.1)
    pub fn key_exchange(
        &self,
        remote_pq_public: &MlKem1024PublicKey,
        remote_classical_public: &[u8; 32],
    ) -> Result<SharedSecret, EnhancedCryptoError> {
        // ML-KEM encapsulation
        let pq_pk = mlkem1024::PublicKey::from_bytes(&remote_pq_public.0)
            .map_err(|_| EnhancedCryptoError::KeyExchange("invalid PQ public key".into()))?;
        let (pq_ss, _pq_ct) = mlkem1024::encapsulate(&pq_pk);
        
        // X25519 key exchange
        let x_secret = StaticSecret::from(self.classical_keypair.private);
        let x_remote = X25519PublicKey::from(*remote_classical_public);
        let x_ss = x_secret.diffie_hellman(&x_remote);
        
        // Combine secrets using BLAKE3
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"vortex-hybrid-kex-v1");
        hasher.update(pq_ss.as_bytes());
        hasher.update(x_ss.as_bytes());
        
        Ok(SharedSecret(hasher.finalize().as_bytes().to_vec()))
    }
}

/// ML-DSA-87 (Dilithium5) Public Key wrapper (Req 1.2)
#[derive(Clone, Debug)]
pub struct MlDsa87PublicKey(pub Vec<u8>);

/// ML-DSA-87 (Dilithium5) Private Key wrapper
#[derive(Clone)]
pub struct MlDsa87PrivateKey(pub Vec<u8>);

/// ML-DSA-87 Signature
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MlDsa87Signature(pub Vec<u8>);

/// ML-DSA-87 Key Pair for post-quantum signatures (Req 1.2)
#[derive(Clone)]
pub struct MlDsa87KeyPair {
    pub public: MlDsa87PublicKey,
    pub private: MlDsa87PrivateKey,
}

impl MlDsa87PublicKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl MlDsa87PrivateKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl MlDsa87KeyPair {
    /// Generate ML-DSA-87 keypair on dedicated thread with 8MB stack (Req 1.6)
    pub fn generate() -> Result<Self, EnhancedCryptoError> {
        let handle = thread::Builder::new()
            .stack_size(PQ_KEY_GEN_STACK_SIZE)
            .spawn(|| {
                let (pk, sk) = mldsa87::keypair();
                (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
            })
            .map_err(|e| EnhancedCryptoError::KeyGeneration(e.to_string()))?;
        
        let (pk_bytes, sk_bytes) = handle.join()
            .map_err(|_| EnhancedCryptoError::KeyGeneration("thread panicked".into()))?;
        
        Ok(Self {
            public: MlDsa87PublicKey(pk_bytes),
            private: MlDsa87PrivateKey(sk_bytes),
        })
    }
    
    /// Sign data with ML-DSA-87 (Req 1.2)
    pub fn sign(&self, data: &[u8]) -> MlDsa87Signature {
        let sk = mldsa87::SecretKey::from_bytes(&self.private.0)
            .expect("valid secret key");
        let sig = mldsa87::detached_sign(data, &sk);
        MlDsa87Signature(sig.as_bytes().to_vec())
    }
    
    /// Verify ML-DSA-87 signature (Req 1.2)
    pub fn verify(&self, data: &[u8], signature: &MlDsa87Signature) -> bool {
        Self::verify_with_public(&self.public, data, signature)
    }
    
    /// Verify signature with public key only
    pub fn verify_with_public(public: &MlDsa87PublicKey, data: &[u8], signature: &MlDsa87Signature) -> bool {
        let pk = match mldsa87::PublicKey::from_bytes(&public.0) {
            Ok(pk) => pk,
            Err(_) => return false,
        };
        let sig = match mldsa87::DetachedSignature::from_bytes(&signature.0) {
            Ok(sig) => sig,
            Err(_) => return false,
        };
        mldsa87::verify_detached_signature(&sig, data, &pk).is_ok()
    }
    
    /// Serialize keypair for storage (Req 1.7)
    pub fn serialize(&self) -> Result<Vec<u8>, EnhancedCryptoError> {
        let mut out = Vec::with_capacity(self.public.0.len() + self.private.0.len() + 8);
        out.extend_from_slice(&(self.public.0.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.public.0);
        out.extend_from_slice(&(self.private.0.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.private.0);
        Ok(out)
    }
    
    /// Deserialize keypair from bytes (Req 1.8)
    pub fn deserialize(data: &[u8]) -> Result<Self, EnhancedCryptoError> {
        if data.len() < 8 {
            return Err(EnhancedCryptoError::Decryption("data too short".into()));
        }
        
        let mut offset = 0;
        let pk_len = u32::from_le_bytes(data[offset..offset+4].try_into()
            .map_err(|_| EnhancedCryptoError::Decryption("invalid pk length".into()))?) as usize;
        offset += 4;
        
        if data.len() < offset + pk_len + 4 {
            return Err(EnhancedCryptoError::Decryption("data too short for pk".into()));
        }
        let pk_bytes = data[offset..offset+pk_len].to_vec();
        offset += pk_len;
        
        let sk_len = u32::from_le_bytes(data[offset..offset+4].try_into()
            .map_err(|_| EnhancedCryptoError::Decryption("invalid sk length".into()))?) as usize;
        offset += 4;
        
        if data.len() < offset + sk_len {
            return Err(EnhancedCryptoError::Decryption("data too short for sk".into()));
        }
        let sk_bytes = data[offset..offset+sk_len].to_vec();
        
        Ok(Self {
            public: MlDsa87PublicKey(pk_bytes),
            private: MlDsa87PrivateKey(sk_bytes),
        })
    }
}

/// HKDF-SHA512 derived session keys (Req 1.3)
#[derive(Clone)]
pub struct SessionKeys {
    pub encryption_key: [u8; 32],
    pub hmac_key: [u8; 32],
    pub iv: [u8; 12],
}

/// HMAC-SHA512 result (Req 1.9)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HmacResult {
    pub tag: [u8; 64],
}

impl Serialize for HmacResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.tag)
    }
}

impl<'de> Deserialize<'de> for HmacResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("expected 64 bytes for HMAC tag"));
        }
        let mut tag = [0u8; 64];
        tag.copy_from_slice(&bytes);
        Ok(HmacResult { tag })
    }
}

/// Key cycling state with atomic counter and history (Req 1.5, 1.10)
pub struct KeyCycler {
    current_key: [u8; 32],
    counter: AtomicU64,
    last_rotation: Instant,
    /// History of previous 3 keys for decrypting recent messages
    key_history: VecDeque<[u8; 32]>,
    /// Maximum messages before rotation
    max_messages: u64,
    /// Maximum duration before rotation
    max_duration: Duration,
}

#[derive(Error, Debug)]
pub enum EnhancedCryptoError {
    #[error("Key generation failed: {0}")]
    KeyGeneration(String),
    #[error("Key exchange failed: {0}")]
    KeyExchange(String),
    #[error("HKDF expansion failed")]
    HkdfExpansion,
    #[error("HMAC verification failed")]
    HmacVerification,
    #[error("Key not found in history")]
    KeyNotInHistory,
    #[error("Decryption failed: {0}")]
    Decryption(String),
    #[error("Signature verification failed")]
    SignatureVerification,
}


impl SessionKeys {
    /// Derive session keys from shared secret using HKDF-SHA512 (Req 1.3)
    pub fn derive_from_secret(shared_secret: &[u8]) -> Result<Self, EnhancedCryptoError> {
        let hk = Hkdf::<Sha512>::new(Some(b"vortex-session-v1"), shared_secret);
        
        let mut encryption_key = [0u8; 32];
        let mut hmac_key = [0u8; 32];
        let mut iv = [0u8; 12];
        
        hk.expand(b"encryption", &mut encryption_key)
            .map_err(|_| EnhancedCryptoError::HkdfExpansion)?;
        hk.expand(b"hmac", &mut hmac_key)
            .map_err(|_| EnhancedCryptoError::HkdfExpansion)?;
        hk.expand(b"iv", &mut iv)
            .map_err(|_| EnhancedCryptoError::HkdfExpansion)?;
        
        Ok(Self { encryption_key, hmac_key, iv })
    }
    
    /// Serialize keys for storage
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(76);
        out.extend_from_slice(&self.encryption_key);
        out.extend_from_slice(&self.hmac_key);
        out.extend_from_slice(&self.iv);
        out
    }
    
    /// Deserialize keys from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 76 {
            return None;
        }
        let encryption_key: [u8; 32] = data[0..32].try_into().ok()?;
        let hmac_key: [u8; 32] = data[32..64].try_into().ok()?;
        let iv: [u8; 12] = data[64..76].try_into().ok()?;
        Some(Self { encryption_key, hmac_key, iv })
    }
}

impl HmacResult {
    /// Compute HMAC-SHA512 for message integrity (Req 1.9)
    pub fn compute(key: &[u8], message: &[u8]) -> Self {
        let mut mac = <HmacSha512 as hmac::Mac>::new_from_slice(key)
            .expect("HMAC can take key of any size");
        hmac::Mac::update(&mut mac, message);
        let result = hmac::Mac::finalize(mac);
        let mut tag = [0u8; 64];
        tag.copy_from_slice(&result.into_bytes());
        Self { tag }
    }
    
    /// Verify HMAC-SHA512 signature (Req 1.9)
    pub fn verify(key: &[u8], message: &[u8], expected: &HmacResult) -> bool {
        let mut mac = <HmacSha512 as hmac::Mac>::new_from_slice(key)
            .expect("HMAC can take key of any size");
        hmac::Mac::update(&mut mac, message);
        hmac::Mac::verify_slice(mac, &expected.tag).is_ok()
    }
}


impl KeyCycler {
    /// Create a new key cycler from initial key (Req 1.5)
    pub fn new(initial_key: [u8; 32]) -> Self {
        Self {
            current_key: initial_key,
            counter: AtomicU64::new(0),
            last_rotation: Instant::now(),
            key_history: VecDeque::with_capacity(3),
            max_messages: 1000,
            max_duration: Duration::from_secs(3600), // 1 hour
        }
    }
    
    /// Check if key rotation is needed (Req 1.5)
    pub fn should_rotate(&self) -> bool {
        let count = self.counter.load(Ordering::SeqCst);
        let elapsed = self.last_rotation.elapsed();
        count >= self.max_messages || elapsed >= self.max_duration
    }
    
    /// Cycle the key, maintaining history of previous 3 keys (Req 1.5, 1.10)
    pub fn cycle(&mut self) -> [u8; 32] {
        // Store current key in history
        if self.key_history.len() >= 3 {
            self.key_history.pop_front();
        }
        self.key_history.push_back(self.current_key);
        
        // Derive new key using BLAKE3
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        let mut hasher = blake3::Hasher::new_keyed(&self.current_key);
        hasher.update(&counter.to_be_bytes());
        hasher.update(b"vortex-key-cycle-v1");
        self.current_key = *hasher.finalize().as_bytes();
        self.last_rotation = Instant::now();
        
        self.current_key
    }
    
    /// Get current key without cycling
    pub fn current_key(&self) -> &[u8; 32] {
        &self.current_key
    }
    
    /// Get message count
    pub fn message_count(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
    
    /// Decrypt using current or historical keys (Req 1.10)
    pub fn decrypt_with_history(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EnhancedCryptoError> {
        // Try current key first
        if let Ok(plaintext) = Self::try_decrypt(&self.current_key, ciphertext) {
            return Ok(plaintext);
        }
        
        // Try historical keys (most recent first)
        for key in self.key_history.iter().rev() {
            if let Ok(plaintext) = Self::try_decrypt(key, ciphertext) {
                return Ok(plaintext);
            }
        }
        
        Err(EnhancedCryptoError::KeyNotInHistory)
    }
    
    fn try_decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>, EnhancedCryptoError> {
        if data.len() < 12 {
            return Err(EnhancedCryptoError::Decryption("data too short".into()));
        }
        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];
        let cipher = ChaCha20Poly1305::new(key.into());
        cipher.decrypt(nonce, ciphertext)
            .map_err(|_| EnhancedCryptoError::Decryption("decryption failed".into()))
    }
    
    /// Encrypt with current key
    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        let cipher = ChaCha20Poly1305::new((&self.current_key).into());
        let mut nonce_bytes = [0u8; 12];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce_bytes);
        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), plaintext)
            .expect("encryption should not fail");
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        result
    }
}


/// Pad message to 2KB boundary for traffic analysis resistance (Req 1.4)
pub fn pad_to_2kb(message: &[u8]) -> Vec<u8> {
    const BLOCK_SIZE: usize = 2048;
    let padding_needed = BLOCK_SIZE - (message.len() % BLOCK_SIZE);
    let total_len = message.len() + padding_needed;
    
    let mut padded = Vec::with_capacity(total_len + 2);
    padded.extend_from_slice(message);
    
    // Random padding bytes
    let mut padding = vec![0u8; padding_needed];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut padding);
    padded.extend_from_slice(&padding);
    
    // Last 2 bytes encode padding length
    padded.extend_from_slice(&(padding_needed as u16).to_be_bytes());
    padded
}

/// Remove 2KB padding from message
pub fn unpad_2kb(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 2 {
        return None;
    }
    let padding_len = u16::from_be_bytes([data[data.len() - 2], data[data.len() - 1]]) as usize;
    let content_len = data.len().checked_sub(padding_len + 2)?;
    Some(data[..content_len].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_session_keys_derivation() {
        let secret = b"test-shared-secret-for-key-derivation";
        let keys = SessionKeys::derive_from_secret(secret).unwrap();
        
        assert_eq!(keys.encryption_key.len(), 32);
        assert_eq!(keys.hmac_key.len(), 32);
        assert_eq!(keys.iv.len(), 12);
        
        // Keys should be different
        assert_ne!(keys.encryption_key, keys.hmac_key);
    }

    #[test]
    fn test_session_keys_roundtrip() {
        let secret = b"test-secret";
        let keys = SessionKeys::derive_from_secret(secret).unwrap();
        let bytes = keys.to_bytes();
        let restored = SessionKeys::from_bytes(&bytes).unwrap();
        
        assert_eq!(keys.encryption_key, restored.encryption_key);
        assert_eq!(keys.hmac_key, restored.hmac_key);
        assert_eq!(keys.iv, restored.iv);
    }


    #[test]
    fn test_hmac_compute_verify() {
        let key = b"test-hmac-key";
        let message = b"Hello, World!";
        
        let hmac = HmacResult::compute(key, message);
        assert!(HmacResult::verify(key, message, &hmac));
        
        // Wrong message should fail
        assert!(!HmacResult::verify(key, b"Wrong message", &hmac));
        
        // Wrong key should fail
        assert!(!HmacResult::verify(b"wrong-key", message, &hmac));
    }

    #[test]
    fn test_key_cycler_rotation() {
        let initial_key = [42u8; 32];
        let mut cycler = KeyCycler::new(initial_key);
        
        let k1 = cycler.cycle();
        let k2 = cycler.cycle();
        let k3 = cycler.cycle();
        
        // Each cycle produces different key
        assert_ne!(k1, k2);
        assert_ne!(k2, k3);
        assert_eq!(cycler.message_count(), 3);
    }

    #[test]
    fn test_key_cycler_history() {
        let initial_key = [42u8; 32];
        let mut cycler = KeyCycler::new(initial_key);
        
        // Encrypt with initial key
        let plaintext = b"secret message";
        let ciphertext = cycler.encrypt(plaintext);
        
        // Cycle keys multiple times
        cycler.cycle();
        cycler.cycle();
        
        // Should still be able to decrypt with history
        let decrypted = cycler.decrypt_with_history(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_key_cycler_history_limit() {
        let initial_key = [42u8; 32];
        let mut cycler = KeyCycler::new(initial_key);
        
        let plaintext = b"secret";
        let ciphertext = cycler.encrypt(plaintext);
        
        // Cycle 4 times - initial key should be evicted from history
        for _ in 0..4 {
            cycler.cycle();
        }
        
        // Should fail to decrypt - key no longer in history
        assert!(cycler.decrypt_with_history(&ciphertext).is_err());
    }

    #[test]
    fn test_2kb_padding() {
        let message = b"Hello, World!";
        let padded = pad_to_2kb(message);
        
        // Should be multiple of 2KB + 2 bytes for length
        assert_eq!((padded.len() - 2) % 2048, 0);
        
        let unpadded = unpad_2kb(&padded).unwrap();
        assert_eq!(unpadded, message);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Feature: examples-integration-analysis, Property 4: Key Derivation Determinism**
        #[test]
        fn prop_key_derivation_determinism(
            secret in prop::collection::vec(any::<u8>(), 16..64),
        ) {
            let keys1 = SessionKeys::derive_from_secret(&secret).unwrap();
            let keys2 = SessionKeys::derive_from_secret(&secret).unwrap();
            
            prop_assert_eq!(keys1.encryption_key, keys2.encryption_key);
            prop_assert_eq!(keys1.hmac_key, keys2.hmac_key);
            prop_assert_eq!(keys1.iv, keys2.iv);
        }

        /// **Feature: examples-integration-analysis, Property 5: Message Padding Invariant**
        #[test]
        fn prop_message_padding_invariant(
            message in prop::collection::vec(any::<u8>(), 0..10000),
        ) {
            let padded = pad_to_2kb(&message);
            
            // Output length is multiple of 2048 + 2 bytes
            prop_assert_eq!((padded.len() - 2) % 2048, 0);
            // Output length >= original length
            prop_assert!(padded.len() >= message.len());
            
            let unpadded = unpad_2kb(&padded).unwrap();
            prop_assert_eq!(unpadded, message);
        }

        /// **Feature: examples-integration-analysis, Property 6: Key Cycling Determinism**
        #[test]
        fn prop_key_cycling_determinism(
            initial_key in prop::collection::vec(any::<u8>(), 32..33),
            cycles in 1usize..10,
        ) {
            let key: [u8; 32] = initial_key.try_into().unwrap();
            let mut cycler1 = KeyCycler::new(key);
            let mut cycler2 = KeyCycler::new(key);
            
            for _ in 0..cycles {
                let k1 = cycler1.cycle();
                let k2 = cycler2.cycle();
                prop_assert_eq!(k1, k2);
            }
        }

        /// **Feature: examples-integration-analysis, Property 7: HMAC Verification Correctness**
        #[test]
        fn prop_hmac_verification_correctness(
            key in prop::collection::vec(any::<u8>(), 16..64),
            message in prop::collection::vec(any::<u8>(), 0..1000),
        ) {
            let hmac = HmacResult::compute(&key, &message);
            prop_assert!(HmacResult::verify(&key, &message, &hmac));
            
            // Modifying any byte should fail verification
            if !message.is_empty() {
                let mut modified = message.clone();
                modified[0] ^= 0xFF;
                prop_assert!(!HmacResult::verify(&key, &modified, &hmac));
            }
        }

        /// **Feature: examples-integration-analysis, Property 1: Cryptographic Key Round-Trip**
        #[test]
        fn prop_session_keys_roundtrip(
            secret in prop::collection::vec(any::<u8>(), 16..64),
        ) {
            let keys = SessionKeys::derive_from_secret(&secret).unwrap();
            let bytes = keys.to_bytes();
            let restored = SessionKeys::from_bytes(&bytes).unwrap();
            
            prop_assert_eq!(keys.encryption_key, restored.encryption_key);
            prop_assert_eq!(keys.hmac_key, restored.hmac_key);
            prop_assert_eq!(keys.iv, restored.iv);
        }
    }
}
