//! Cryptography Module - Post-Quantum Secure
//! 
//! Platform-specific implementations:
//! 
//! ## Default (iOS + all platforms):
//! Uses pure-Rust implementations that work on ALL platforms:
//! - Kyber1024 for key encapsulation (pqc_kyber crate)
//! - Dilithium3 for digital signatures (pqc_dilithium crate)
//! 
//! ## With `pqcrypto-backend` feature (Android/Desktop only):
//! Uses pqcrypto C bindings with optimized assembly:
//! - ML-KEM-1024 via pqcrypto-mlkem
//! - Dilithium3 via pqcrypto-dilithium
//! 
//! Common across all:
//! - X25519 for classical key exchange (hybrid mode)
//! - Ed25519 for classical signatures (hybrid mode)
//! - ChaCha20-Poly1305 for symmetric encryption
//!
//! The pure-Rust pqc_kyber and pqc_dilithium crates have NO assembly dependencies,
//! making them compatible with iOS ARM builds.

use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Nonce};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use x25519_dalek::{StaticSecret, PublicKey as X25519Public};
use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier, Signature};
use hkdf::Hkdf;
use sha2::Sha512;

// ============================================================================
// Conditional PQ Crypto Imports
// ============================================================================

// Pure-Rust Kyber - DEFAULT for all platforms including iOS
#[cfg(not(feature = "pqcrypto-backend"))]
use pqc_kyber::{keypair as kyber_keypair, encapsulate, decapsulate, KYBER_PUBLICKEYBYTES, KYBER_SECRETKEYBYTES, KYBER_CIPHERTEXTBYTES};

// Pure-Rust Dilithium - DEFAULT for all platforms including iOS
#[cfg(not(feature = "pqcrypto-backend"))]
use pqc_dilithium::{Keypair as DilithiumKeypair, verify as dilithium_verify, PUBLICKEYBYTES as DIL_PUBLICKEYBYTES, SECRETKEYBYTES as DIL_SECRETKEYBYTES};

// pqcrypto C bindings - OPTIONAL for Android/Desktop (faster, but has assembly)
#[cfg(feature = "pqcrypto-backend")]
use pqcrypto_mlkem::mlkem1024;
#[cfg(feature = "pqcrypto-backend")]
use pqcrypto_dilithium::dilithium3;
#[cfg(feature = "pqcrypto-backend")]
use pqcrypto_traits::kem::{PublicKey as PqKemPubKey, SecretKey as PqKemSecKey, Ciphertext as PqCiphertext, SharedSecret as PqSharedSecret};
#[cfg(feature = "pqcrypto-backend")]
use pqcrypto_traits::sign::{PublicKey as PqSignPubKey, SecretKey as PqSignSecKey, DetachedSignature};

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
    #[error("not supported on this platform")]
    NotSupported,
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
// Hybrid Keypair - Pure Rust PQ + Classical Crypto
// ============================================================================

/// Hybrid keypair combining post-quantum and classical cryptography
/// - ML-KEM-1024 for key encapsulation (PQ-secure)
/// - X25519 for classical key exchange (defense in depth)
/// - ML-DSA-65 for signatures (PQ-secure)
/// - Ed25519 for classical signatures (defense in depth)
#[derive(Clone)]
pub struct HybridKeypair {
    // ML-KEM (Kyber) keys - post-quantum key encapsulation
    pub pq_encap_key: Vec<u8>,      // Public encapsulation key
    pub pq_decap_key: Vec<u8>,      // Secret decapsulation key
    // X25519 keys - classical ECDH
    pub x25519_secret: [u8; 32],
    pub x25519_public: [u8; 32],
    // ML-DSA (Dilithium) keys - post-quantum signatures
    pub pq_signing_key: Vec<u8>,    // Secret signing key
    pub pq_verifying_key: Vec<u8>,  // Public verifying key
    // Ed25519 keys - classical signatures
    pub ed_signing_key: [u8; 32],
    pub ed_verifying_key: [u8; 32],
}

/// Public bundle for sharing with others
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PublicBundle {
    pub pq_encap: Vec<u8>,      // ML-KEM encapsulation key
    pub x25519: [u8; 32],       // X25519 public key
    pub pq_verify: Vec<u8>,     // ML-DSA verifying key
    pub ed_verify: [u8; 32],    // Ed25519 verifying key
}

/// Encapsulated key for key exchange
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncapsulatedKey {
    pub pq_ciphertext: Vec<u8>,      // ML-KEM ciphertext
    pub x25519_ephemeral: [u8; 32],  // X25519 ephemeral public key
}

/// Encrypted payload with all necessary data for decryption
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
    pub encap: EncapsulatedKey,
}


// ============================================================================
// HybridKeypair Implementation - Conditional PQ Crypto
// ============================================================================

impl HybridKeypair {
    /// Generate a new hybrid keypair with both PQ and classical keys
    #[cfg(not(feature = "pqcrypto-backend"))]
    pub fn generate() -> Result<Self, CryptoError> {
        let mut rng = rand::thread_rng();
        
        // Generate Kyber1024 keypair (post-quantum key encapsulation) - Pure Rust
        let kyber_keys = kyber_keypair(&mut rng)
            .map_err(|_| CryptoError::KeyGeneration("Kyber keypair generation failed".into()))?;
        
        // Generate X25519 keypair (classical ECDH)
        let x_secret = StaticSecret::random_from_rng(&mut rng);
        let x_public = X25519Public::from(&x_secret);
        
        // Generate Dilithium3 keypair (post-quantum signatures) - Pure Rust
        let dil_keys = DilithiumKeypair::generate();
        
        // Generate Ed25519 keypair (classical signatures)
        let ed_sign_key = SigningKey::generate(&mut rng);
        let ed_verify_key = ed_sign_key.verifying_key();
        
        Ok(Self {
            pq_encap_key: kyber_keys.public.to_vec(),
            pq_decap_key: kyber_keys.secret.to_vec(),
            x25519_secret: x_secret.to_bytes(),
            x25519_public: x_public.to_bytes(),
            pq_signing_key: dil_keys.expose_secret().to_vec(),
            pq_verifying_key: dil_keys.public.to_vec(),
            ed_signing_key: ed_sign_key.to_bytes(),
            ed_verifying_key: ed_verify_key.to_bytes(),
        })
    }

    /// Generate a new hybrid keypair - pqcrypto backend (Android/Desktop)
    #[cfg(feature = "pqcrypto-backend")]
    pub fn generate() -> Result<Self, CryptoError> {
        let mut rng = rand::thread_rng();
        
        // Generate ML-KEM-1024 keypair using pqcrypto
        let (pq_encap, pq_decap) = mlkem1024::keypair();
        
        // Generate X25519 keypair (classical ECDH)
        let x_secret = StaticSecret::random_from_rng(&mut rng);
        let x_public = X25519Public::from(&x_secret);
        
        // Generate Dilithium3 keypair using pqcrypto
        let (pq_verify, pq_sign) = dilithium3::keypair();
        
        // Generate Ed25519 keypair (classical signatures)
        let ed_sign_key = SigningKey::generate(&mut rng);
        let ed_verify_key = ed_sign_key.verifying_key();
        
        Ok(Self {
            pq_encap_key: pq_encap.as_bytes().to_vec(),
            pq_decap_key: pq_decap.as_bytes().to_vec(),
            x25519_secret: x_secret.to_bytes(),
            x25519_public: x_public.to_bytes(),
            pq_signing_key: pq_sign.as_bytes().to_vec(),
            pq_verifying_key: pq_verify.as_bytes().to_vec(),
            ed_signing_key: ed_sign_key.to_bytes(),
            ed_verifying_key: ed_verify_key.to_bytes(),
        })
    }

    /// Get the public bundle for sharing
    pub fn public_bundle(&self) -> PublicBundle {
        PublicBundle {
            pq_encap: self.pq_encap_key.clone(),
            x25519: self.x25519_public,
            pq_verify: self.pq_verifying_key.clone(),
            ed_verify: self.ed_verifying_key,
        }
    }

    /// Sign data using hybrid signatures - Pure Rust (Dilithium3 + Ed25519)
    #[cfg(not(feature = "pqcrypto-backend"))]
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        // Dilithium3 signature - reconstruct keypair using transmute
        // SAFETY: Keypair is repr(C) with public: [u8; PUBLICKEYBYTES], secret: [u8; SECRETKEYBYTES]
        let mut keypair_bytes = [0u8; DIL_PUBLICKEYBYTES + DIL_SECRETKEYBYTES];
        keypair_bytes[..DIL_PUBLICKEYBYTES].copy_from_slice(&self.pq_verifying_key[..DIL_PUBLICKEYBYTES]);
        keypair_bytes[DIL_PUBLICKEYBYTES..].copy_from_slice(&self.pq_signing_key[..DIL_SECRETKEYBYTES]);
        
        // SAFETY: The Keypair struct layout is { public: [u8; PUBLICKEYBYTES], secret: [u8; SECRETKEYBYTES] }
        let dil_keys: DilithiumKeypair = unsafe { std::mem::transmute(keypair_bytes) };
        let pq_sig = dil_keys.sign(data);
        
        // Ed25519 signature
        let ed_sign_key = SigningKey::from_bytes(&self.ed_signing_key);
        let ed_sig = ed_sign_key.sign(data);
        
        // Combine signatures: [pq_sig_len (4 bytes)][pq_sig][ed_sig]
        let mut combined = Vec::with_capacity(4 + pq_sig.len() + 64);
        combined.extend_from_slice(&(pq_sig.len() as u32).to_le_bytes());
        combined.extend_from_slice(&pq_sig);
        combined.extend_from_slice(&ed_sig.to_bytes());
        combined
    }

    /// Sign data using hybrid signatures - pqcrypto backend (Dilithium3 + Ed25519)
    #[cfg(feature = "pqcrypto-backend")]
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        // Dilithium3 signature using pqcrypto
        let pq_sign_key = dilithium3::SecretKey::from_bytes(&self.pq_signing_key)
            .expect("invalid dilithium secret key");
        let pq_sig = dilithium3::detached_sign(data, &pq_sign_key);
        
        // Ed25519 signature
        let ed_sign_key = SigningKey::from_bytes(&self.ed_signing_key);
        let ed_sig = ed_sign_key.sign(data);
        
        // Combine signatures: [pq_sig_len (4 bytes)][pq_sig][ed_sig]
        let pq_sig_bytes = pq_sig.as_bytes();
        let mut combined = Vec::with_capacity(4 + pq_sig_bytes.len() + 64);
        combined.extend_from_slice(&(pq_sig_bytes.len() as u32).to_le_bytes());
        combined.extend_from_slice(pq_sig_bytes);
        combined.extend_from_slice(&ed_sig.to_bytes());
        combined
    }

    /// Verify a hybrid signature - Pure Rust
    #[cfg(not(feature = "pqcrypto-backend"))]
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        if signature.len() < 68 {
            return Err(CryptoError::SignatureInvalid);
        }
        
        // Parse combined signature
        let pq_sig_len = u32::from_le_bytes(signature[..4].try_into().unwrap()) as usize;
        if signature.len() < 4 + pq_sig_len + 64 {
            return Err(CryptoError::SignatureInvalid);
        }
        let pq_sig_bytes = &signature[4..4 + pq_sig_len];
        let ed_sig_bytes = &signature[4 + pq_sig_len..4 + pq_sig_len + 64];
        
        // Verify Dilithium3 signature
        let mut pk = [0u8; DIL_PUBLICKEYBYTES];
        pk.copy_from_slice(&self.pq_verifying_key[..DIL_PUBLICKEYBYTES]);
        dilithium_verify(pq_sig_bytes, data, &pk)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        
        // Verify Ed25519 signature
        let ed_verify_key = VerifyingKey::from_bytes(&self.ed_verifying_key)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let ed_sig_arr: [u8; 64] = ed_sig_bytes.try_into()
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let ed_sig = Signature::from_bytes(&ed_sig_arr);
        ed_verify_key.verify(data, &ed_sig)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        
        Ok(())
    }

    /// Verify a hybrid signature - pqcrypto backend
    #[cfg(feature = "pqcrypto-backend")]
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        if signature.len() < 68 {
            return Err(CryptoError::SignatureInvalid);
        }
        
        // Parse combined signature
        let pq_sig_len = u32::from_le_bytes(signature[..4].try_into().unwrap()) as usize;
        if signature.len() < 4 + pq_sig_len + 64 {
            return Err(CryptoError::SignatureInvalid);
        }
        let pq_sig_bytes = &signature[4..4 + pq_sig_len];
        let ed_sig_bytes = &signature[4 + pq_sig_len..4 + pq_sig_len + 64];
        
        // Verify Dilithium3 signature using pqcrypto
        let pq_verify_key = dilithium3::PublicKey::from_bytes(&self.pq_verifying_key)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let pq_sig = dilithium3::DetachedSignature::from_bytes(pq_sig_bytes)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        dilithium3::verify_detached_signature(&pq_sig, data, &pq_verify_key)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        
        // Verify Ed25519 signature
        let ed_verify_key = VerifyingKey::from_bytes(&self.ed_verifying_key)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let ed_sig_arr: [u8; 64] = ed_sig_bytes.try_into()
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let ed_sig = Signature::from_bytes(&ed_sig_arr);
        ed_verify_key.verify(data, &ed_sig)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        
        Ok(())
    }
}


// ============================================================================
// Serialization for HybridKeypair
// ============================================================================

impl HybridKeypair {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        // PQ encapsulation key
        out.extend_from_slice(&(self.pq_encap_key.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.pq_encap_key);
        // PQ decapsulation key
        out.extend_from_slice(&(self.pq_decap_key.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.pq_decap_key);
        // X25519 keys
        out.extend_from_slice(&self.x25519_secret);
        out.extend_from_slice(&self.x25519_public);
        // PQ signing key
        out.extend_from_slice(&(self.pq_signing_key.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.pq_signing_key);
        // PQ verifying key
        out.extend_from_slice(&(self.pq_verifying_key.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.pq_verifying_key);
        // Ed25519 keys
        out.extend_from_slice(&self.ed_signing_key);
        out.extend_from_slice(&self.ed_verifying_key);
        out
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, CryptoError> {
        let mut offset = 0;
        
        // PQ encapsulation key
        let pq_encap_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        let pq_encap_key = data[offset..offset+pq_encap_len].to_vec();
        offset += pq_encap_len;
        
        // PQ decapsulation key
        let pq_decap_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        let pq_decap_key = data[offset..offset+pq_decap_len].to_vec();
        offset += pq_decap_len;
        
        // X25519 keys
        let x25519_secret: [u8; 32] = data[offset..offset+32].try_into()
            .map_err(|_| CryptoError::KeyExchange("invalid x25519 secret".into()))?;
        offset += 32;
        let x25519_public: [u8; 32] = data[offset..offset+32].try_into()
            .map_err(|_| CryptoError::KeyExchange("invalid x25519 public".into()))?;
        offset += 32;
        
        // PQ signing key
        let pq_sign_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        let pq_signing_key = data[offset..offset+pq_sign_len].to_vec();
        offset += pq_sign_len;
        
        // PQ verifying key
        let pq_verify_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;
        let pq_verifying_key = data[offset..offset+pq_verify_len].to_vec();
        offset += pq_verify_len;
        
        // Ed25519 keys
        let ed_signing_key: [u8; 32] = data[offset..offset+32].try_into()
            .map_err(|_| CryptoError::KeyExchange("invalid ed25519 signing key".into()))?;
        offset += 32;
        let ed_verifying_key: [u8; 32] = data[offset..offset+32].try_into()
            .map_err(|_| CryptoError::KeyExchange("invalid ed25519 verifying key".into()))?;
        
        Ok(Self {
            pq_encap_key,
            pq_decap_key,
            x25519_secret,
            x25519_public,
            pq_signing_key,
            pq_verifying_key,
            ed_signing_key,
            ed_verifying_key,
        })
    }

    pub fn to_encrypted_bytes(&self, password: &[u8]) -> Result<Vec<u8>, CryptoError> {
        encrypt_with_password(&self.to_bytes(), password)
    }

    pub fn from_encrypted_bytes(data: &[u8], password: &[u8]) -> Result<Self, CryptoError> {
        let plaintext = decrypt_with_password(data, password)?;
        Self::from_bytes(&plaintext)
    }
}


// ============================================================================
// PublicBundle Implementation
// ============================================================================

impl PublicBundle {
    /// Verify a hybrid signature using this public bundle - Pure Rust
    #[cfg(not(feature = "pqcrypto-backend"))]
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        if signature.len() < 68 {
            return Err(CryptoError::SignatureInvalid);
        }
        
        // Parse combined signature
        let pq_sig_len = u32::from_le_bytes(signature[..4].try_into().unwrap()) as usize;
        if signature.len() < 4 + pq_sig_len + 64 {
            return Err(CryptoError::SignatureInvalid);
        }
        let pq_sig_bytes = &signature[4..4 + pq_sig_len];
        let ed_sig_bytes = &signature[4 + pq_sig_len..4 + pq_sig_len + 64];
        
        // Verify Dilithium3 signature
        let mut pk = [0u8; DIL_PUBLICKEYBYTES];
        pk.copy_from_slice(&self.pq_verify[..DIL_PUBLICKEYBYTES]);
        dilithium_verify(pq_sig_bytes, data, &pk)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        
        // Verify Ed25519 signature
        let ed_verify_key = VerifyingKey::from_bytes(&self.ed_verify)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let ed_sig_arr: [u8; 64] = ed_sig_bytes.try_into()
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let ed_sig = Signature::from_bytes(&ed_sig_arr);
        ed_verify_key.verify(data, &ed_sig)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        
        Ok(())
    }

    /// Verify a hybrid signature using this public bundle - pqcrypto backend
    #[cfg(feature = "pqcrypto-backend")]
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        if signature.len() < 68 {
            return Err(CryptoError::SignatureInvalid);
        }
        
        // Parse combined signature
        let pq_sig_len = u32::from_le_bytes(signature[..4].try_into().unwrap()) as usize;
        if signature.len() < 4 + pq_sig_len + 64 {
            return Err(CryptoError::SignatureInvalid);
        }
        let pq_sig_bytes = &signature[4..4 + pq_sig_len];
        let ed_sig_bytes = &signature[4 + pq_sig_len..4 + pq_sig_len + 64];
        
        // Verify Dilithium3 signature using pqcrypto
        let pq_verify_key = dilithium3::PublicKey::from_bytes(&self.pq_verify)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let pq_sig = dilithium3::DetachedSignature::from_bytes(pq_sig_bytes)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        dilithium3::verify_detached_signature(&pq_sig, data, &pq_verify_key)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        
        // Verify Ed25519 signature
        let ed_verify_key = VerifyingKey::from_bytes(&self.ed_verify)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let ed_sig_arr: [u8; 64] = ed_sig_bytes.try_into()
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let ed_sig = Signature::from_bytes(&ed_sig_arr);
        ed_verify_key.verify(data, &ed_sig)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        
        Ok(())
    }
}

// ============================================================================
// Hybrid Encryption/Decryption - Kyber + X25519
// ============================================================================

/// Derive a symmetric key from hybrid shared secrets
fn derive_hybrid_key(pq_ss: &[u8], x25519_ss: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"vortex-hybrid-pq-v1");
    hasher.update(pq_ss);
    hasher.update(x25519_ss);
    *hasher.finalize().as_bytes()
}

/// Encrypt data for a recipient using hybrid PQ + classical key exchange - Pure Rust
#[cfg(not(feature = "pqcrypto-backend"))]
pub fn encrypt(data: &[u8], recipient: &PublicBundle) -> Result<EncryptedPayload, CryptoError> {
    let mut rng = rand::thread_rng();
    
    // Kyber encapsulation
    let mut pk = [0u8; KYBER_PUBLICKEYBYTES];
    pk.copy_from_slice(&recipient.pq_encap[..KYBER_PUBLICKEYBYTES]);
    let (pq_ciphertext, pq_shared_secret) = encapsulate(&pk, &mut rng)
        .map_err(|_| CryptoError::KeyExchange("Kyber encapsulation failed".into()))?;
    
    // X25519 key exchange
    let x_ephemeral = StaticSecret::random_from_rng(&mut rng);
    let x_ephemeral_pub = X25519Public::from(&x_ephemeral);
    let x_recipient = X25519Public::from(recipient.x25519);
    let x_ss = x_ephemeral.diffie_hellman(&x_recipient);
    
    // Derive symmetric key from both shared secrets
    let key = derive_hybrid_key(&pq_shared_secret, x_ss.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key.into());
    
    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rng, &mut nonce_bytes);
    
    // Encrypt
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), data)
        .map_err(|_| CryptoError::Encrypt)?;
    
    Ok(EncryptedPayload {
        nonce: nonce_bytes,
        ciphertext,
        encap: EncapsulatedKey {
            pq_ciphertext: pq_ciphertext.to_vec(),
            x25519_ephemeral: x_ephemeral_pub.to_bytes(),
        },
    })
}

/// Encrypt data for a recipient - pqcrypto backend
#[cfg(feature = "pqcrypto-backend")]
pub fn encrypt(data: &[u8], recipient: &PublicBundle) -> Result<EncryptedPayload, CryptoError> {
    let mut rng = rand::thread_rng();
    
    // ML-KEM encapsulation using pqcrypto
    let pq_encap_key = mlkem1024::PublicKey::from_bytes(&recipient.pq_encap)
        .map_err(|_| CryptoError::KeyExchange("invalid ML-KEM public key".into()))?;
    let (pq_shared_secret, pq_ciphertext) = mlkem1024::encapsulate(&pq_encap_key);
    
    // X25519 key exchange
    let x_ephemeral = StaticSecret::random_from_rng(&mut rng);
    let x_ephemeral_pub = X25519Public::from(&x_ephemeral);
    let x_recipient = X25519Public::from(recipient.x25519);
    let x_ss = x_ephemeral.diffie_hellman(&x_recipient);
    
    // Derive symmetric key from both shared secrets
    let key = derive_hybrid_key(pq_shared_secret.as_bytes(), x_ss.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key.into());
    
    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rng, &mut nonce_bytes);
    
    // Encrypt
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), data)
        .map_err(|_| CryptoError::Encrypt)?;
    
    Ok(EncryptedPayload {
        nonce: nonce_bytes,
        ciphertext,
        encap: EncapsulatedKey {
            pq_ciphertext: pq_ciphertext.as_bytes().to_vec(),
            x25519_ephemeral: x_ephemeral_pub.to_bytes(),
        },
    })
}

/// Decrypt data using hybrid PQ + classical key exchange - Pure Rust
#[cfg(not(feature = "pqcrypto-backend"))]
pub fn decrypt(payload: &EncryptedPayload, keypair: &HybridKeypair) -> Result<Vec<u8>, CryptoError> {
    // Kyber decapsulation
    let mut ct = [0u8; KYBER_CIPHERTEXTBYTES];
    ct.copy_from_slice(&payload.encap.pq_ciphertext[..KYBER_CIPHERTEXTBYTES]);
    let mut sk = [0u8; KYBER_SECRETKEYBYTES];
    sk.copy_from_slice(&keypair.pq_decap_key[..KYBER_SECRETKEYBYTES]);
    let pq_shared_secret = decapsulate(&ct, &sk)
        .map_err(|_| CryptoError::KeyExchange("Kyber decapsulation failed".into()))?;
    
    // X25519 key exchange
    let x_secret = StaticSecret::from(keypair.x25519_secret);
    let x_ephemeral = X25519Public::from(payload.encap.x25519_ephemeral);
    let x_ss = x_secret.diffie_hellman(&x_ephemeral);
    
    // Derive symmetric key
    let key = derive_hybrid_key(&pq_shared_secret, x_ss.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key.into());
    
    // Decrypt
    cipher.decrypt(Nonce::from_slice(&payload.nonce), payload.ciphertext.as_ref())
        .map_err(|_| CryptoError::Decrypt("authentication failed".into()))
}

/// Decrypt data - pqcrypto backend
#[cfg(feature = "pqcrypto-backend")]
pub fn decrypt(payload: &EncryptedPayload, keypair: &HybridKeypair) -> Result<Vec<u8>, CryptoError> {
    // ML-KEM decapsulation using pqcrypto
    let pq_decap_key = mlkem1024::SecretKey::from_bytes(&keypair.pq_decap_key)
        .map_err(|_| CryptoError::KeyExchange("invalid ML-KEM secret key".into()))?;
    let pq_ciphertext = mlkem1024::Ciphertext::from_bytes(&payload.encap.pq_ciphertext)
        .map_err(|_| CryptoError::KeyExchange("invalid ML-KEM ciphertext".into()))?;
    let pq_shared_secret = mlkem1024::decapsulate(&pq_ciphertext, &pq_decap_key);
    
    // X25519 key exchange
    let x_secret = StaticSecret::from(keypair.x25519_secret);
    let x_ephemeral = X25519Public::from(payload.encap.x25519_ephemeral);
    let x_ss = x_secret.diffie_hellman(&x_ephemeral);
    
    // Derive symmetric key
    let key = derive_hybrid_key(pq_shared_secret.as_bytes(), x_ss.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key.into());
    
    // Decrypt
    cipher.decrypt(Nonce::from_slice(&payload.nonce), payload.ciphertext.as_ref())
        .map_err(|_| CryptoError::Decrypt("authentication failed".into()))
}


// ============================================================================
// Session Keys (HKDF-SHA512)
// ============================================================================

#[derive(Clone)]
pub struct SessionKeys {
    pub encryption_key: [u8; 32],
    pub hmac_key: [u8; 32],
    pub iv: [u8; 12],
}

impl SessionKeys {
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
    
    let mut out = Vec::with_capacity(16 + 12 + ciphertext.len());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

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

pub fn hash_data(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

// ============================================================================
// Secure Token Storage
// ============================================================================

fn get_machine_key() -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"vortex-machine-key-v1");
    
    if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
        hasher.update(id.trim().as_bytes());
    } else if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
        hasher.update(id.trim().as_bytes());
    } else {
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            hasher.update(hostname.as_bytes());
        }
        if let Ok(user) = std::env::var("USER") {
            hasher.update(user.as_bytes());
        }
    }
    
    hasher.update(b"vortex-image-secure-storage");
    *hasher.finalize().as_bytes()
}

pub fn encrypt_token(token: &str) -> Result<Vec<u8>, CryptoError> {
    let key = get_machine_key();
    let cipher = ChaCha20Poly1305::new(&key.into());
    
    let mut nonce_bytes = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce_bytes);
    
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), token.as_bytes())
        .map_err(|_| CryptoError::Encrypt)?;
    
    let mut out = Vec::with_capacity(1 + 12 + ciphertext.len());
    out.push(0x02);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

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
        .map_err(|_| CryptoError::Decrypt("token decryption failed".into()))?;
    
    String::from_utf8(plaintext)
        .map_err(|_| CryptoError::Decrypt("invalid token data".into()))
}


// ============================================================================
// Per-Item Encryption Settings
// ============================================================================

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


// ============================================================================
// Tauri Commands
// ============================================================================

use crate::github::AppError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeypairResult {
    pub public_bundle: PublicBundle,
    pub keypair_bytes: Vec<u8>,
}

#[tauri::command]
pub async fn generate_keypair() -> Result<KeypairResult, AppError> {
    let keypair = HybridKeypair::generate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(KeypairResult {
        public_bundle: keypair.public_bundle(),
        keypair_bytes: keypair.to_bytes(),
    })
}

#[tauri::command]
pub async fn encrypt_data_password(data: Vec<u8>, password: String) -> Result<Vec<u8>, AppError> {
    encrypt_with_password(&data, password.as_bytes())
        .map_err(|e| AppError::Validation(e.to_string()))
}

#[tauri::command]
pub async fn decrypt_data_password(data: Vec<u8>, password: String) -> Result<Vec<u8>, AppError> {
    decrypt_with_password(&data, password.as_bytes())
        .map_err(|e| AppError::Validation(e.to_string()))
}

#[tauri::command]
pub async fn encrypt_keypair(keypair_bytes: Vec<u8>, password: String) -> Result<Vec<u8>, AppError> {
    let keypair = HybridKeypair::from_bytes(&keypair_bytes)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    keypair.to_encrypted_bytes(password.as_bytes())
        .map_err(|e| AppError::Validation(e.to_string()))
}

#[tauri::command]
pub async fn decrypt_keypair(encrypted_bytes: Vec<u8>, password: String) -> Result<KeypairResult, AppError> {
    let keypair = HybridKeypair::from_encrypted_bytes(&encrypted_bytes, password.as_bytes())
        .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(KeypairResult {
        public_bundle: keypair.public_bundle(),
        keypair_bytes: keypair.to_bytes(),
    })
}

#[tauri::command]
pub async fn encrypt_hybrid(data: Vec<u8>, recipient_bundle: PublicBundle) -> Result<Vec<u8>, AppError> {
    let payload = encrypt(&data, &recipient_bundle)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    serde_json::to_vec(&payload)
        .map_err(|e| AppError::Validation(e.to_string()))
}

#[tauri::command]
pub async fn decrypt_hybrid(encrypted_data: Vec<u8>, keypair_bytes: Vec<u8>) -> Result<Vec<u8>, AppError> {
    let payload: EncryptedPayload = serde_json::from_slice(&encrypted_data)
        .map_err(|e| AppError::Validation(format!("Invalid encrypted data: {}", e)))?;
    let keypair = HybridKeypair::from_bytes(&keypair_bytes)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    decrypt(&payload, &keypair)
        .map_err(|e| AppError::Validation(e.to_string()))
}

#[tauri::command]
pub async fn sign_data(data: Vec<u8>, keypair_bytes: Vec<u8>) -> Result<Vec<u8>, AppError> {
    let keypair = HybridKeypair::from_bytes(&keypair_bytes)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(keypair.sign(&data))
}

#[tauri::command]
pub async fn verify_signature(data: Vec<u8>, signature: Vec<u8>, public_bundle: PublicBundle) -> Result<bool, AppError> {
    match public_bundle.verify(&data, &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionKeysResult {
    pub encryption_key: Vec<u8>,
    pub hmac_key: Vec<u8>,
    pub iv: Vec<u8>,
}

#[tauri::command]
pub async fn derive_session_keys(shared_secret: Vec<u8>) -> Result<SessionKeysResult, AppError> {
    let keys = SessionKeys::derive_from_secret(&shared_secret)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(SessionKeysResult {
        encryption_key: keys.encryption_key.to_vec(),
        hmac_key: keys.hmac_key.to_vec(),
        iv: keys.iv.to_vec(),
    })
}

#[tauri::command]
pub fn hash_data_blake3(data: Vec<u8>) -> Vec<u8> {
    hash_data(&data).to_vec()
}

#[tauri::command]
pub fn get_crypto_info() -> serde_json::Value {
    #[cfg(not(feature = "pqcrypto-backend"))]
    {
        serde_json::json!({
            "key_exchange": "ML-KEM-1024 (Kyber) + X25519 Hybrid",
            "signatures": "ML-DSA-65 (Dilithium) + Ed25519 Hybrid",
            "symmetric": "ChaCha20-Poly1305",
            "kdf": "Argon2id + HKDF-SHA512",
            "hash": "BLAKE3",
            "pq_security_level": "NIST Level 3 (192-bit post-quantum)",
            "classical_security_level": "128-bit classical",
            "implementation": "Pure Rust (ml-kem, ml-dsa crates)",
            "backend": "pure-rust",
            "ios_compatible": true,
            "android_compatible": true
        })
    }
    #[cfg(feature = "pqcrypto-backend")]
    {
        serde_json::json!({
            "key_exchange": "ML-KEM-1024 (Kyber) + X25519 Hybrid",
            "signatures": "Dilithium3 + Ed25519 Hybrid",
            "symmetric": "ChaCha20-Poly1305",
            "kdf": "Argon2id + HKDF-SHA512",
            "hash": "BLAKE3",
            "pq_security_level": "NIST Level 3 (192-bit post-quantum)",
            "classical_security_level": "128-bit classical",
            "implementation": "pqcrypto C bindings (optimized assembly)",
            "backend": "pqcrypto",
            "ios_compatible": false,
            "android_compatible": true
        })
    }
}

#[tauri::command]
pub async fn secure_store_token(token: String) -> Result<Vec<u8>, AppError> {
    encrypt_token(&token).map_err(|e| AppError::Validation(e.to_string()))
}

#[tauri::command]
pub async fn secure_retrieve_token(encrypted: Vec<u8>) -> Result<String, AppError> {
    decrypt_token(&encrypted).map_err(|e| AppError::Validation(e.to_string()))
}

#[tauri::command]
pub async fn encrypt_file(
    data: Vec<u8>,
    settings: EncryptionSettings,
    password: Option<String>,
    keypair_bytes: Option<Vec<u8>>,
) -> Result<EncryptedFileData, AppError> {
    encrypt_file_data(&data, &settings, password.as_deref(), keypair_bytes.as_deref())
        .map_err(|e| AppError::Validation(e.to_string()))
}

#[tauri::command]
pub async fn decrypt_file(
    encrypted: EncryptedFileData,
    password: Option<String>,
    keypair_bytes: Option<Vec<u8>>,
) -> Result<Vec<u8>, AppError> {
    decrypt_file_data(&encrypted, password.as_deref(), keypair_bytes.as_deref())
        .map_err(|e| AppError::Validation(e.to_string()))
}
