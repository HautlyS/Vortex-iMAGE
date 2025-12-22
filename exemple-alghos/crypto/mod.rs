//! Hybrid Post-Quantum + Classical Cryptography
//! ML-KEM-1024 (Kyber) + X25519 + ChaCha20-Poly1305 + Ed25519 Signatures
pub mod privacy;
pub mod enhanced;

pub use enhanced::{
    SessionKeys, HmacResult, KeyCycler, EnhancedCryptoError, 
    pad_to_2kb, unpad_2kb,
    PQKeyPair, MlKem1024PublicKey, MlKem1024PrivateKey,
    X25519KeyPair, HybridKeyExchange, SharedSecret,
    MlDsa87KeyPair, MlDsa87PublicKey, MlDsa87PrivateKey, MlDsa87Signature,
};

use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Nonce};
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature};
use pqcrypto_mlkem::mlkem1024;
use pqcrypto_traits::kem::{Ciphertext, PublicKey, SecretKey, SharedSecret as KemSharedSecret};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519Public, StaticSecret};

pub use privacy::{pad_message, unpad_message, SessionKey, TimingSafeChannel};

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("encryption failed")]
    Encrypt,
    #[error("decryption failed: {0}")]
    Decrypt(String),
    #[error("key exchange failed")]
    KeyExchange,
    #[error("signature verification failed")]
    SignatureInvalid,
    #[error("key derivation failed")]
    KeyDerivation,
}

/// Hybrid keypair: PQ (ML-KEM-1024) + Classical (X25519) + Ed25519 Signing
#[derive(Clone)]
pub struct HybridKeypair {
    pub pq_public: Vec<u8>,
    pub pq_secret: Vec<u8>,
    pub x25519_secret: [u8; 32],
    pub x25519_public: [u8; 32],
    pub signing_key: [u8; 32],
    pub verifying_key: [u8; 32],
}

impl HybridKeypair {
    pub fn generate() -> Self {
        let (pq_pk, pq_sk) = mlkem1024::keypair();
        let x_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let x_public = X25519Public::from(&x_secret);
        
        // Generate Ed25519 signing keypair
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let verifying_key = signing_key.verifying_key();
        
        Self {
            pq_public: pq_pk.as_bytes().to_vec(),
            pq_secret: pq_sk.as_bytes().to_vec(),
            x25519_secret: x_secret.to_bytes(),
            x25519_public: x_public.to_bytes(),
            signing_key: signing_key.to_bytes(),
            verifying_key: verifying_key.to_bytes(),
        }
    }

    pub fn public_bundle(&self) -> PublicBundle {
        PublicBundle { 
            pq: self.pq_public.clone(), 
            x25519: self.x25519_public,
            verifying_key: self.verifying_key,
        }
    }

    /// Sign data with Ed25519
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let signing_key = SigningKey::from_bytes(&self.signing_key);
        signing_key.sign(data).to_bytes().to_vec()
    }

    /// Verify signature with this keypair's verifying key
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        let verifying_key = VerifyingKey::from_bytes(&self.verifying_key)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let sig = Signature::from_slice(signature)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        verifying_key.verify(data, &sig)
            .map_err(|_| CryptoError::SignatureInvalid)
    }

    /// Serialize keypair for storage (unencrypted)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.pq_public.len() + self.pq_secret.len() + 128);
        out.extend_from_slice(&(self.pq_public.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.pq_public);
        out.extend_from_slice(&(self.pq_secret.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.pq_secret);
        out.extend_from_slice(&self.x25519_secret);
        out.extend_from_slice(&self.x25519_public);
        out.extend_from_slice(&self.signing_key);
        out.extend_from_slice(&self.verifying_key);
        out
    }

    /// Serialize keypair with encryption (for secure storage)
    /// Uses Argon2id for memory-hard password-based key derivation (CRITICAL FIX #1)
    pub fn to_encrypted_bytes(&self, password: &[u8]) -> Result<Vec<u8>, CryptoError> {
        use argon2::Argon2;
        
        let plaintext = self.to_bytes();
        
        // Generate random salt (16 bytes)
        let mut salt = [0u8; 16];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut salt);
        
        // Derive key from password using Argon2id (memory-hard, brute-force resistant)
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
        out.push(0x01); // Version byte for future compatibility
        out.extend_from_slice(&salt);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    /// Deserialize keypair from encrypted bytes
    /// Uses Argon2id for memory-hard password-based key derivation (CRITICAL FIX #1)
    pub fn from_encrypted_bytes(data: &[u8], password: &[u8]) -> Result<Self, CryptoError> {
        use argon2::Argon2;
        
        // Minimum: version(1) + salt(16) + nonce(12) + tag(16) = 45 bytes
        if data.len() < 45 {
            return Err(CryptoError::Decrypt("data too short".into()));
        }
        
        let version = data[0];
        if version != 0x01 {
            return Err(CryptoError::Decrypt("unsupported encryption version".into()));
        }
        
        let salt: [u8; 16] = data[1..17].try_into()
            .map_err(|_| CryptoError::Decrypt("invalid salt".into()))?;
        let nonce_bytes: [u8; 12] = data[17..29].try_into()
            .map_err(|_| CryptoError::Decrypt("invalid nonce".into()))?;
        let ciphertext = &data[29..];
        
        // Derive key from password using Argon2id
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password, &salt, &mut key)
            .map_err(|_| CryptoError::KeyDerivation)?;
        
        let cipher = ChaCha20Poly1305::new(&key.into());
        let plaintext = cipher.decrypt(Nonce::from_slice(&nonce_bytes), ciphertext)
            .map_err(|_| CryptoError::Decrypt("decryption failed - wrong password?".into()))?;
        
        Self::from_bytes(&plaintext)
    }

    /// Deserialize keypair (unencrypted)
    pub fn from_bytes(data: &[u8]) -> Result<Self, CryptoError> {
        let mut offset = 0;
        
        let pq_pub_len = u32::from_le_bytes(data[offset..offset+4].try_into().map_err(|_| CryptoError::KeyExchange)?) as usize;
        offset += 4;
        let pq_public = data[offset..offset+pq_pub_len].to_vec();
        offset += pq_pub_len;
        
        let pq_sec_len = u32::from_le_bytes(data[offset..offset+4].try_into().map_err(|_| CryptoError::KeyExchange)?) as usize;
        offset += 4;
        let pq_secret = data[offset..offset+pq_sec_len].to_vec();
        offset += pq_sec_len;
        
        let x25519_secret: [u8; 32] = data[offset..offset+32].try_into().map_err(|_| CryptoError::KeyExchange)?;
        offset += 32;
        let x25519_public: [u8; 32] = data[offset..offset+32].try_into().map_err(|_| CryptoError::KeyExchange)?;
        offset += 32;
        
        // Handle legacy keypairs without signing keys
        let (signing_key, verifying_key) = if offset + 64 <= data.len() {
            let sk: [u8; 32] = data[offset..offset+32].try_into().map_err(|_| CryptoError::KeyExchange)?;
            offset += 32;
            let vk: [u8; 32] = data[offset..offset+32].try_into().map_err(|_| CryptoError::KeyExchange)?;
            (sk, vk)
        } else {
            // Generate new signing keys for legacy keypairs
            let sk = SigningKey::generate(&mut rand::thread_rng());
            let vk = sk.verifying_key();
            (sk.to_bytes(), vk.to_bytes())
        };
        
        Ok(Self { pq_public, pq_secret, x25519_secret, x25519_public, signing_key, verifying_key })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicBundle {
    pub pq: Vec<u8>,
    pub x25519: [u8; 32],
    #[serde(default)]
    pub verifying_key: [u8; 32],
}

impl PublicBundle {
    /// Verify a signature against this bundle's verifying key
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        let verifying_key = VerifyingKey::from_bytes(&self.verifying_key)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let sig = Signature::from_slice(signature)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        verifying_key.verify(data, &sig)
            .map_err(|_| CryptoError::SignatureInvalid)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncapsulatedKey {
    pub pq_ciphertext: Vec<u8>,
    pub x25519_ephemeral: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
    pub encap: EncapsulatedKey,
}

fn derive_hybrid_key(pq_ss: &[u8], x25519_ss: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"vortex-hybrid-v1");
    hasher.update(pq_ss);
    hasher.update(x25519_ss);
    *hasher.finalize().as_bytes()
}

pub fn encrypt(data: &[u8], recipient: &PublicBundle) -> Result<EncryptedPayload, CryptoError> {
    let pq_pk = mlkem1024::PublicKey::from_bytes(&recipient.pq).map_err(|_| CryptoError::KeyExchange)?;
    let (pq_ss, pq_ct) = mlkem1024::encapsulate(&pq_pk);
    
    let x_ephemeral = EphemeralSecret::random_from_rng(rand::thread_rng());
    let x_ephemeral_pub = X25519Public::from(&x_ephemeral);
    let x_recipient = X25519Public::from(recipient.x25519);
    let x_ss = x_ephemeral.diffie_hellman(&x_recipient);
    
    let key = derive_hybrid_key(pq_ss.as_bytes(), x_ss.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key.into());
    let mut nonce_bytes = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce_bytes);
    
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), data).map_err(|_| CryptoError::Encrypt)?;
    
    Ok(EncryptedPayload {
        nonce: nonce_bytes,
        ciphertext,
        encap: EncapsulatedKey { pq_ciphertext: pq_ct.as_bytes().to_vec(), x25519_ephemeral: x_ephemeral_pub.to_bytes() },
    })
}

pub fn decrypt(payload: &EncryptedPayload, keypair: &HybridKeypair) -> Result<Vec<u8>, CryptoError> {
    let pq_sk = mlkem1024::SecretKey::from_bytes(&keypair.pq_secret).map_err(|_| CryptoError::KeyExchange)?;
    let pq_ct = mlkem1024::Ciphertext::from_bytes(&payload.encap.pq_ciphertext).map_err(|_| CryptoError::KeyExchange)?;
    let pq_ss = mlkem1024::decapsulate(&pq_ct, &pq_sk);
    
    let x_secret = StaticSecret::from(keypair.x25519_secret);
    let x_ephemeral = X25519Public::from(payload.encap.x25519_ephemeral);
    let x_ss = x_secret.diffie_hellman(&x_ephemeral);
    
    let key = derive_hybrid_key(pq_ss.as_bytes(), x_ss.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key.into());
    
    cipher.decrypt(Nonce::from_slice(&payload.nonce), payload.ciphertext.as_ref())
        .map_err(|_| CryptoError::Decrypt("authentication failed".into()))
}

/// Encrypt with compression (for large data)
pub async fn encrypt_compressed(data: &[u8], recipient: &PublicBundle) -> Result<EncryptedPayload, CryptoError> {
    let compressed = crate::compress::compress_segmented(data, 3).await.map_err(|_| CryptoError::Encrypt)?;
    encrypt(&compressed, recipient)
}

/// Decrypt and decompress
pub async fn decrypt_decompressed(payload: &EncryptedPayload, keypair: &HybridKeypair) -> Result<Vec<u8>, CryptoError> {
    let compressed = decrypt(payload, keypair)?;
    crate::compress::decompress_segmented(&compressed).await
        .map(|b| b.to_vec())
        .map_err(|e| CryptoError::Decrypt(e.to_string()))
}

/// Encrypt any serializable data with ML-KEM
pub fn encrypt_data<T: serde::Serialize>(data: &T, recipient: &PublicBundle) -> Result<EncryptedPayload, CryptoError> {
    let bytes = bincode::serialize(data).map_err(|_| CryptoError::Encrypt)?;
    encrypt(&bytes, recipient)
}

/// Decrypt to any deserializable type
pub fn decrypt_data<T: serde::de::DeserializeOwned>(payload: &EncryptedPayload, keypair: &HybridKeypair) -> Result<T, CryptoError> {
    let bytes = decrypt(payload, keypair)?;
    bincode::deserialize(&bytes).map_err(|e| CryptoError::Decrypt(e.to_string()))
}

/// Encrypt for multiple recipients (group encryption)
pub fn encrypt_for_group<T: serde::Serialize>(data: &T, recipients: &[PublicBundle]) -> Result<Vec<EncryptedPayload>, CryptoError> {
    recipients.iter().map(|r| encrypt_data(data, r)).collect()
}

/// Verify data integrity with BLAKE3 hash
pub fn hash_data(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// Verify hash matches data
pub fn verify_hash(data: &[u8], expected: &[u8; 32]) -> bool {
    &hash_data(data) == expected
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_hybrid_encrypt_decrypt() {
        let bob = HybridKeypair::generate();
        let message = b"Post-quantum secure message!";
        let encrypted = encrypt(message, &bob.public_bundle()).unwrap();
        let decrypted = decrypt(&encrypted, &bob).unwrap();
        assert_eq!(message.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_wrong_key_fails() {
        let alice = HybridKeypair::generate();
        let bob = HybridKeypair::generate();
        let encrypted = encrypt(b"secret", &bob.public_bundle()).unwrap();
        assert!(decrypt(&encrypted, &alice).is_err());
    }

    #[test]
    fn test_keypair_serialization() {
        let kp = HybridKeypair::generate();
        let bytes = kp.to_bytes();
        let restored = HybridKeypair::from_bytes(&bytes).unwrap();
        assert_eq!(kp.pq_public, restored.pq_public);
        assert_eq!(kp.x25519_public, restored.x25519_public);
        assert_eq!(kp.signing_key, restored.signing_key);
        assert_eq!(kp.verifying_key, restored.verifying_key);
    }

    #[test]
    fn test_large_message() {
        let bob = HybridKeypair::generate();
        let message = vec![0xABu8; 1_000_000];
        let encrypted = encrypt(&message, &bob.public_bundle()).unwrap();
        let decrypted = decrypt(&encrypted, &bob).unwrap();
        assert_eq!(message, decrypted);
    }

    #[tokio::test]
    async fn test_encrypt_compressed() {
        let bob = HybridKeypair::generate();
        let message = vec![42u8; 500_000];
        let encrypted = encrypt_compressed(&message, &bob.public_bundle()).await.unwrap();
        let decrypted = decrypt_decompressed(&encrypted, &bob).await.unwrap();
        assert_eq!(message, decrypted);
    }

    #[test]
    fn test_pq_key_sizes() {
        // Property 2: Keypair Generation Validity
        let kp = HybridKeypair::generate();
        assert_eq!(kp.pq_public.len(), 1568);
        assert_eq!(kp.pq_secret.len(), 3168);
        assert_eq!(kp.x25519_public.len(), 32);
        assert_eq!(kp.x25519_secret.len(), 32);
        assert_eq!(kp.signing_key.len(), 32);
        assert_eq!(kp.verifying_key.len(), 32);
    }

    #[test]
    fn test_multiple_encryptions() {
        let bob = HybridKeypair::generate();
        let msg = b"test";
        let e1 = encrypt(msg, &bob.public_bundle()).unwrap();
        let e2 = encrypt(msg, &bob.public_bundle()).unwrap();
        assert_ne!(e1.nonce, e2.nonce);
        assert_ne!(e1.ciphertext, e2.ciphertext);
        assert_eq!(decrypt(&e1, &bob).unwrap(), msg);
        assert_eq!(decrypt(&e2, &bob).unwrap(), msg);
    }

    #[test]
    fn test_tampered_ciphertext() {
        let bob = HybridKeypair::generate();
        let mut encrypted = encrypt(b"secret", &bob.public_bundle()).unwrap();
        encrypted.ciphertext[0] ^= 0xFF;
        assert!(decrypt(&encrypted, &bob).is_err());
    }

    #[test]
    fn test_tampered_nonce() {
        let bob = HybridKeypair::generate();
        let mut encrypted = encrypt(b"secret", &bob.public_bundle()).unwrap();
        encrypted.nonce[0] ^= 0xFF;
        assert!(decrypt(&encrypted, &bob).is_err());
    }

    #[test]
    fn test_empty_message() {
        let bob = HybridKeypair::generate();
        let encrypted = encrypt(b"", &bob.public_bundle()).unwrap();
        let decrypted = decrypt(&encrypted, &bob).unwrap();
        assert_eq!(decrypted, b"");
    }

    #[test]
    fn test_binary_data() {
        let bob = HybridKeypair::generate();
        let data: Vec<u8> = (0..=255).collect();
        let encrypted = encrypt(&data, &bob.public_bundle()).unwrap();
        let decrypted = decrypt(&encrypted, &bob).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_hybrid_key_derivation() {
        let pq_ss = vec![0xAAu8; 32];
        let x_ss = vec![0xBBu8; 32];
        let key1 = derive_hybrid_key(&pq_ss, &x_ss);
        let key2 = derive_hybrid_key(&pq_ss, &x_ss);
        assert_eq!(key1, key2);
        
        let x_ss_diff = vec![0xCCu8; 32];
        let key3 = derive_hybrid_key(&pq_ss, &x_ss_diff);
        assert_ne!(key1, key3);
    }

    #[tokio::test]
    async fn test_compressed_large_data() {
        let bob = HybridKeypair::generate();
        let message = vec![42u8; 5_000_000];
        let encrypted = encrypt_compressed(&message, &bob.public_bundle()).await.unwrap();
        assert!(encrypted.ciphertext.len() < message.len() / 50);
        let decrypted = decrypt_decompressed(&encrypted, &bob).await.unwrap();
        assert_eq!(message, decrypted);
    }

    #[test]
    fn test_public_bundle_serialization() {
        let kp = HybridKeypair::generate();
        let bundle = kp.public_bundle();
        let json = serde_json::to_string(&bundle).unwrap();
        let restored: PublicBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(bundle.pq, restored.pq);
        assert_eq!(bundle.x25519, restored.x25519);
        assert_eq!(bundle.verifying_key, restored.verifying_key);
    }

    #[test]
    fn test_encrypted_payload_serialization() {
        let bob = HybridKeypair::generate();
        let encrypted = encrypt(b"test", &bob.public_bundle()).unwrap();
        let json = serde_json::to_string(&encrypted).unwrap();
        let restored: EncryptedPayload = serde_json::from_str(&json).unwrap();
        let decrypted = decrypt(&restored, &bob).unwrap();
        assert_eq!(decrypted, b"test");
    }

    #[test]
    fn test_sign_and_verify() {
        let kp = HybridKeypair::generate();
        let data = b"message to sign";
        let signature = kp.sign(data);
        assert!(kp.verify(data, &signature).is_ok());
        
        // Verify with public bundle
        let bundle = kp.public_bundle();
        assert!(bundle.verify(data, &signature).is_ok());
    }

    #[test]
    fn test_signature_wrong_data_fails() {
        let kp = HybridKeypair::generate();
        let signature = kp.sign(b"original");
        assert!(kp.verify(b"tampered", &signature).is_err());
    }

    #[test]
    fn test_signature_wrong_key_fails() {
        let alice = HybridKeypair::generate();
        let bob = HybridKeypair::generate();
        let signature = alice.sign(b"message");
        assert!(bob.verify(b"message", &signature).is_err());
    }

    #[test]
    fn test_encrypted_keypair_storage() {
        let kp = HybridKeypair::generate();
        let password = b"secure_password_123";
        
        let encrypted = kp.to_encrypted_bytes(password).unwrap();
        let restored = HybridKeypair::from_encrypted_bytes(&encrypted, password).unwrap();
        
        assert_eq!(kp.pq_public, restored.pq_public);
        assert_eq!(kp.pq_secret, restored.pq_secret);
        assert_eq!(kp.x25519_public, restored.x25519_public);
        assert_eq!(kp.x25519_secret, restored.x25519_secret);
        assert_eq!(kp.signing_key, restored.signing_key);
        assert_eq!(kp.verifying_key, restored.verifying_key);
    }

    #[test]
    fn test_encrypted_keypair_wrong_password_fails() {
        let kp = HybridKeypair::generate();
        let encrypted = kp.to_encrypted_bytes(b"correct").unwrap();
        assert!(HybridKeypair::from_encrypted_bytes(&encrypted, b"wrong").is_err());
    }

    // Property-based tests
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 1: Keypair Serialization Round-Trip
        #[test]
        fn prop_keypair_serialization_roundtrip(_seed in 0u64..1000) {
            let kp = HybridKeypair::generate();
            let bytes = kp.to_bytes();
            let restored = HybridKeypair::from_bytes(&bytes).unwrap();
            
            prop_assert_eq!(kp.pq_public, restored.pq_public);
            prop_assert_eq!(kp.pq_secret, restored.pq_secret);
            prop_assert_eq!(kp.x25519_public, restored.x25519_public);
            prop_assert_eq!(kp.x25519_secret, restored.x25519_secret);
            prop_assert_eq!(kp.signing_key, restored.signing_key);
            prop_assert_eq!(kp.verifying_key, restored.verifying_key);
        }

        /// Property 2: Keypair Generation Validity
        #[test]
        fn prop_keypair_generation_validity(_seed in 0u64..100) {
            let kp = HybridKeypair::generate();
            
            // ML-KEM-1024 key sizes
            prop_assert_eq!(kp.pq_public.len(), 1568);
            prop_assert_eq!(kp.pq_secret.len(), 3168);
            
            // X25519 key sizes
            prop_assert_eq!(kp.x25519_public.len(), 32);
            prop_assert_eq!(kp.x25519_secret.len(), 32);
            
            // Ed25519 key sizes
            prop_assert_eq!(kp.signing_key.len(), 32);
            prop_assert_eq!(kp.verifying_key.len(), 32);
        }

        /// Property 3: Message Encryption Round-Trip
        #[test]
        fn prop_message_encryption_roundtrip(
            data in prop::collection::vec(any::<u8>(), 0..1000),
        ) {
            let kp = HybridKeypair::generate();
            let encrypted = encrypt(&data, &kp.public_bundle()).unwrap();
            let decrypted = decrypt(&encrypted, &kp).unwrap();
            prop_assert_eq!(data, decrypted);
        }

        /// Property 4: Wrong Key Decryption Fails
        #[test]
        fn prop_wrong_key_decryption_fails(
            data in prop::collection::vec(any::<u8>(), 1..100),
        ) {
            let alice = HybridKeypair::generate();
            let bob = HybridKeypair::generate();
            
            let encrypted = encrypt(&data, &bob.public_bundle()).unwrap();
            let result = decrypt(&encrypted, &alice);
            
            prop_assert!(result.is_err());
        }

        /// Property: Signature Round-Trip
        #[test]
        fn prop_signature_roundtrip(
            data in prop::collection::vec(any::<u8>(), 0..1000),
        ) {
            let kp = HybridKeypair::generate();
            let signature = kp.sign(&data);
            prop_assert!(kp.verify(&data, &signature).is_ok());
            prop_assert!(kp.public_bundle().verify(&data, &signature).is_ok());
        }

        /// Property: Encrypted Keypair Storage Round-Trip
        #[test]
        fn prop_encrypted_keypair_roundtrip(
            password in prop::collection::vec(any::<u8>(), 8..32),
        ) {
            let kp = HybridKeypair::generate();
            let encrypted = kp.to_encrypted_bytes(&password).unwrap();
            let restored = HybridKeypair::from_encrypted_bytes(&encrypted, &password).unwrap();
            
            prop_assert_eq!(kp.pq_public, restored.pq_public);
            prop_assert_eq!(kp.signing_key, restored.signing_key);
        }
    }
}
