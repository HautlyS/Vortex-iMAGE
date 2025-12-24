//! Hybrid Post-Quantum Cryptography Module - Security Hardened v4
//!
//! Provides defense-in-depth encryption using both classical and post-quantum algorithms:
//! - Key Exchange: ML-KEM-1024 (Kyber) + X25519 hybrid
//! - Signatures: ML-DSA-65 (Dilithium) + Ed25519 hybrid
//! - Symmetric: ChaCha20-Poly1305 (AEAD) with AAD support
//! - KDF: Argon2id (password) + HKDF-SHA512 (session)
//! - Hash: BLAKE3
//! - Storage: OS Keychain integration (macOS Keychain, Windows Credential Manager, Linux Secret Service)
//!
//! Security Features:
//! - Opaque keypair handles (no raw bytes to frontend)
//! - Key rotation support with backward compatibility
//! - Automatic v2→v3→v4 token migration
//! - Associated Authenticated Data (AAD) in AEAD
//! - No Clone on secret types (explicit clone_secret() only)
//! - Safe Dilithium signing (no unsafe transmute)
//!
//! Platform Support:
//! - iOS: Pure Rust backend (pqc_kyber, pqc_dilithium) - no assembly
//! - Android/Desktop: Optional pqcrypto backend with optimized assembly
//!
//! SECURITY NOTE: pqc_dilithium pinned to =0.2.0 and pqc_kyber to =0.7.1
//! to ensure memory layout compatibility with safe signing code.

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use thiserror::Error;
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use sha2::Sha512;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[cfg(not(feature = "pqcrypto-backend"))]
use pqc_kyber::{
    decapsulate, encapsulate, keypair as kyber_keypair, KYBER_CIPHERTEXTBYTES,
    KYBER_PUBLICKEYBYTES, KYBER_SECRETKEYBYTES,
};

#[cfg(not(feature = "pqcrypto-backend"))]
use pqc_dilithium::{
    verify as dilithium_verify, Keypair as DilithiumKeypair,
    PUBLICKEYBYTES as DIL_PUBLICKEYBYTES, SECRETKEYBYTES as DIL_SECRETKEYBYTES,
};

#[cfg(feature = "pqcrypto-backend")]
use pqcrypto_mlkem::mlkem1024;
#[cfg(feature = "pqcrypto-backend")]
use pqcrypto_dilithium::dilithium3;
#[cfg(feature = "pqcrypto-backend")]
use pqcrypto_traits::kem::{
    Ciphertext as PqCiphertext, PublicKey as PqKemPubKey, SecretKey as PqKemSecKey,
    SharedSecret as PqSharedSecret,
};
#[cfg(feature = "pqcrypto-backend")]
use pqcrypto_traits::sign::{
    DetachedSignature, PublicKey as PqSignPubKey, SecretKey as PqSignSecKey,
};

// ============================================================================
// Constants
// ============================================================================

/// Current token format version (v4 with AAD support)
pub(crate) const TOKEN_VERSION_V4: u8 = 0x04;
/// Legacy token versions for migration
pub(crate) const TOKEN_VERSION_V3: u8 = 0x03;
#[allow(dead_code)]
pub(crate) const TOKEN_VERSION_V2: u8 = 0x02;
/// Application identifier for keychain
const KEYCHAIN_SERVICE: &str = "com.vortex.image.crypto";
/// Domain separator for hybrid key derivation
const HYBRID_KDF_DOMAIN: &[u8] = b"vortex-hybrid-pq-v2";
/// Domain separator for session keys
#[allow(dead_code)]
const SESSION_KDF_DOMAIN: &[u8] = b"vortex-session-v3";
/// Domain separator for token encryption
const TOKEN_KDF_DOMAIN: &[u8] = b"vortex-token-v4";

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("encryption failed")]
    Encrypt(#[allow(dead_code)] String),
    #[error("decryption failed")]
    Decrypt(#[allow(dead_code)] String),
    #[error("key exchange failed")]
    KeyExchange(#[allow(dead_code)] String),
    #[error("signature verification failed")]
    SignatureInvalid,
    #[error("key derivation failed")]
    KeyDerivation(#[allow(dead_code)] String),
    #[error("key generation failed")]
    KeyGeneration(#[allow(dead_code)] String),
    #[error("invalid input data")]
    InvalidInput(#[allow(dead_code)] String),
    #[error("not supported on this platform")]
    #[allow(dead_code)]
    NotSupported,
    #[error("keypair not found")]
    KeypairNotFound,
    #[error("keychain error")]
    Keychain(#[allow(dead_code)] String),
    #[error("key rotation required")]
    #[allow(dead_code)]
    KeyRotationRequired,
    #[error("AAD mismatch")]
    AadMismatch,
    #[error("unsupported token version")]
    UnsupportedTokenVersion(u8),
}

impl Serialize for CryptoError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Generic error messages to prevent information leakage
        let msg = match self {
            CryptoError::Encrypt(_) => "encryption failed",
            CryptoError::Decrypt(_) => "decryption failed",
            CryptoError::KeyExchange(_) => "key exchange failed",
            CryptoError::SignatureInvalid => "signature verification failed",
            CryptoError::KeyDerivation(_) => "key derivation failed",
            CryptoError::KeyGeneration(_) => "key generation failed",
            CryptoError::InvalidInput(_) => "invalid input",
            CryptoError::NotSupported => "not supported",
            CryptoError::KeypairNotFound => "keypair not found",
            CryptoError::Keychain(_) => "keychain error",
            CryptoError::KeyRotationRequired => "key rotation required",
            CryptoError::AadMismatch => "authentication failed",
            CryptoError::UnsupportedTokenVersion(_) => "unsupported token version",
        };
        serializer.serialize_str(msg)
    }
}

// ============================================================================
// Argon2 Configuration - Secure Parameters
// ============================================================================

/// Secure Argon2id parameters following OWASP recommendations
/// Memory: 64 MiB, Iterations: 3, Parallelism: 4
fn get_argon2_params() -> argon2::Params {
    argon2::Params::new(
        64 * 1024, // 64 MiB memory cost
        3,         // 3 iterations
        4,         // 4 parallel lanes
        Some(32),  // 32-byte output
    )
    .expect("valid argon2 params")
}

fn get_argon2() -> argon2::Argon2<'static> {
    argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        get_argon2_params(),
    )
}

// ============================================================================
// Secure Key Types with Zeroization - NO CLONE
// ============================================================================

/// Wrapper for secret key material that zeroizes on drop
/// NOTE: Clone intentionally NOT derived to prevent accidental copies
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Explicit clone for when absolutely necessary (auditable)
    /// This method name makes cloning visible in code review
    #[allow(dead_code)]
    pub fn clone_secret(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Fixed-size secret key with zeroization
/// NOTE: Clone intentionally NOT derived
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretKey32([u8; 32]);

impl SecretKey32 {
    pub fn new(data: [u8; 32]) -> Self {
        Self(data)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Explicit clone for when absolutely necessary (auditable)
    #[allow(dead_code)]
    pub fn clone_secret(&self) -> Self {
        Self(self.0)
    }
}

// ============================================================================
// Opaque Keypair Handle System
// ============================================================================

/// Opaque handle to a keypair stored in memory
/// Frontend only sees this ID, never the actual key bytes
pub type KeypairHandle = u64;

// Global keypair store - keeps keypairs in memory with opaque handles
lazy_static::lazy_static! {
    static ref KEYPAIR_STORE: RwLock<KeypairStore> = RwLock::new(KeypairStore::new());
}

/// Internal keypair storage with rotation support
/// Made pub(crate) for testing in security_verify module
pub(crate) struct KeypairStore {
    keypairs: HashMap<KeypairHandle, Arc<Mutex<HybridKeypair>>>,
    next_handle: KeypairHandle,
    /// Previous keypairs for key rotation (handle -> old keypairs)
    rotated_keypairs: HashMap<KeypairHandle, Vec<Arc<Mutex<HybridKeypair>>>>,
}

impl KeypairStore {
    pub(crate) fn new() -> Self {
        Self {
            keypairs: HashMap::new(),
            next_handle: 1,
            rotated_keypairs: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, keypair: HybridKeypair) -> KeypairHandle {
        let handle = self.next_handle;
        self.next_handle += 1;
        self.keypairs.insert(handle, Arc::new(Mutex::new(keypair)));
        handle
    }

    pub(crate) fn get(&self, handle: KeypairHandle) -> Option<Arc<Mutex<HybridKeypair>>> {
        self.keypairs.get(&handle).cloned()
    }

    pub(crate) fn remove(&mut self, handle: KeypairHandle) -> Option<Arc<Mutex<HybridKeypair>>> {
        // Also remove any rotated keypairs
        self.rotated_keypairs.remove(&handle);
        self.keypairs.remove(&handle)
    }

    /// Rotate a keypair: generate new one, keep old for decryption
    pub(crate) fn rotate(&mut self, handle: KeypairHandle) -> Result<PublicBundle, CryptoError> {
        let old_keypair = self.keypairs.remove(&handle)
            .ok_or(CryptoError::KeypairNotFound)?;

        // Get rotation count from old keypair
        let old_rotation_count = {
            let kp = old_keypair.lock().unwrap();
            kp.rotation_count
        };

        // Store old keypair in rotation history
        self.rotated_keypairs
            .entry(handle)
            .or_insert_with(Vec::new)
            .push(old_keypair);

        // Generate new keypair with same handle
        let mut new_keypair = HybridKeypair::generate()?;
        new_keypair.rotation_count = old_rotation_count + 1;
        let public_bundle = new_keypair.public_bundle();
        
        self.keypairs.insert(handle, Arc::new(Mutex::new(new_keypair)));

        Ok(public_bundle)
    }

    /// Get all keypairs for a handle (current + rotated) for decryption attempts
    pub(crate) fn get_all_for_decryption(&self, handle: KeypairHandle) -> Vec<Arc<Mutex<HybridKeypair>>> {
        let mut result = Vec::new();

        // Current keypair first
        if let Some(current) = self.keypairs.get(&handle) {
            result.push(current.clone());
        }

        // Then rotated keypairs (newest first)
        if let Some(rotated) = self.rotated_keypairs.get(&handle) {
            for kp in rotated.iter().rev() {
                result.push(kp.clone());
            }
        }

        result
    }
}


// ============================================================================
// Hybrid Keypair - Main Cryptographic Identity
// ============================================================================

/// Hybrid post-quantum keypair combining classical and PQ algorithms
/// All secret material is zeroized on drop
/// NOTE: Clone intentionally NOT derived to prevent accidental copies of secret material
pub struct HybridKeypair {
    // Post-quantum KEM keys (ML-KEM-1024 / Kyber)
    pub pq_encap_key: Vec<u8>,
    pq_decap_key: SecretBytes,

    // Classical ECDH keys (X25519)
    x25519_secret: SecretKey32,
    pub x25519_public: [u8; 32],

    // Post-quantum signature keys (ML-DSA-65 / Dilithium)
    pq_signing_key: SecretBytes,
    pub pq_verifying_key: Vec<u8>,

    // Classical signature keys (Ed25519)
    ed_signing_key: SecretKey32,
    pub ed_verifying_key: [u8; 32],

    // Key metadata
    pub created_at: u64,
    pub rotation_count: u32,
}

impl Drop for HybridKeypair {
    fn drop(&mut self) {
        // SecretBytes and SecretKey32 auto-zeroize, but clear public keys too
        self.pq_encap_key.zeroize();
        self.x25519_public.zeroize();
        self.pq_verifying_key.zeroize();
        self.ed_verifying_key.zeroize();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PublicBundle {
    pub pq_encap: Vec<u8>,
    pub x25519: [u8; 32],
    pub pq_verify: Vec<u8>,
    pub ed_verify: [u8; 32],
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub key_id: String,
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
    /// BLAKE3 hash of AAD for verification
    #[serde(default)]
    pub aad_hash: Option<[u8; 32]>,
}

/// Result returned to frontend - contains handle, NOT keypair bytes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeypairInfo {
    pub handle: KeypairHandle,
    pub public_bundle: PublicBundle,
    pub created_at: u64,
    pub key_id: String,
}

impl HybridKeypair {
    /// Generate a new hybrid keypair (pure Rust backend for iOS compatibility)
    #[cfg(not(feature = "pqcrypto-backend"))]
    pub fn generate() -> Result<Self, CryptoError> {
        let mut rng = OsRng;

        // Generate ML-KEM-1024 (Kyber) keypair
        let kyber_keys = kyber_keypair(&mut rng)
            .map_err(|e| CryptoError::KeyGeneration(format!("Kyber: {}", e)))?;

        // Generate X25519 keypair
        let x_secret = StaticSecret::random_from_rng(&mut rng);
        let x_public = X25519Public::from(&x_secret);

        // Generate Dilithium keypair
        let dil_keys = DilithiumKeypair::generate();

        // Generate Ed25519 keypair
        let ed_sign_key = SigningKey::generate(&mut rng);
        let ed_verify_key = ed_sign_key.verifying_key();

        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(Self {
            pq_encap_key: kyber_keys.public.to_vec(),
            pq_decap_key: SecretBytes::new(kyber_keys.secret.to_vec()),
            x25519_secret: SecretKey32::new(x_secret.to_bytes()),
            x25519_public: x_public.to_bytes(),
            pq_signing_key: SecretBytes::new(dil_keys.expose_secret().to_vec()),
            pq_verifying_key: dil_keys.public.to_vec(),
            ed_signing_key: SecretKey32::new(ed_sign_key.to_bytes()),
            ed_verifying_key: ed_verify_key.to_bytes(),
            created_at,
            rotation_count: 0,
        })
    }

    /// Generate a new hybrid keypair (pqcrypto backend with optimized assembly)
    #[cfg(feature = "pqcrypto-backend")]
    pub fn generate() -> Result<Self, CryptoError> {
        let mut rng = OsRng;

        // Generate ML-KEM-1024 keypair
        let (pq_encap, pq_decap) = mlkem1024::keypair();

        // Generate X25519 keypair
        let x_secret = StaticSecret::random_from_rng(&mut rng);
        let x_public = X25519Public::from(&x_secret);

        // Generate Dilithium3 keypair
        let (pq_verify, pq_sign) = dilithium3::keypair();

        // Generate Ed25519 keypair
        let ed_sign_key = SigningKey::generate(&mut rng);
        let ed_verify_key = ed_sign_key.verifying_key();

        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(Self {
            pq_encap_key: pq_encap.as_bytes().to_vec(),
            pq_decap_key: SecretBytes::new(pq_decap.as_bytes().to_vec()),
            x25519_secret: SecretKey32::new(x_secret.to_bytes()),
            x25519_public: x_public.to_bytes(),
            pq_signing_key: SecretBytes::new(pq_sign.as_bytes().to_vec()),
            pq_verifying_key: pq_verify.as_bytes().to_vec(),
            ed_signing_key: SecretKey32::new(ed_sign_key.to_bytes()),
            ed_verifying_key: ed_verify_key.to_bytes(),
            created_at,
            rotation_count: 0,
        })
    }

    /// Generate a unique key ID from public key material
    fn key_id(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.pq_encap_key);
        hasher.update(&self.x25519_public);
        hasher.update(&self.ed_verifying_key);
        hex::encode(&hasher.finalize().as_bytes()[..8])
    }

    /// Extract the public bundle for sharing
    pub fn public_bundle(&self) -> PublicBundle {
        PublicBundle {
            pq_encap: self.pq_encap_key.clone(),
            x25519: self.x25519_public,
            pq_verify: self.pq_verifying_key.clone(),
            ed_verify: self.ed_verifying_key,
            created_at: self.created_at,
            key_id: self.key_id(),
        }
    }

    /// Safe Dilithium signing with documented unsafe block
    /// 
    /// # Safety Invariants
    /// 
    /// This function contains an unsafe block that relies on the following invariants:
    /// 
    /// 1. **Version Pinning**: The `pqc_dilithium` crate is pinned to exact version `=0.2.0`
    ///    in Cargo.toml. This ensures the internal struct layout remains consistent.
    ///    See: `pqc_dilithium = { version = "=0.2.0", ... }` in Cargo.toml
    /// 
    /// 2. **Struct Layout**: The `DilithiumKeypair` struct has layout:
    ///    `{ public: [u8; 1952], secret: [u8; 4016] }` (total 5968 bytes for mode3)
    ///    This is verified at compile-time by the size assertion below.
    /// 
    /// 3. **Byte Length Validation**: Before reconstruction, we validate:
    ///    - `pq_signing_key.len() == DIL_SECRETKEYBYTES (4016)`
    ///    - `pq_verifying_key.len() == DIL_PUBLICKEYBYTES (1952)`
    /// 
    /// 4. **Memory Alignment**: The keypair_bytes array is stack-allocated with
    ///    natural alignment. The `std::ptr::read` operation handles unaligned reads.
    /// 
    /// 5. **No Padding**: The Keypair struct contains only fixed-size byte arrays,
    ///    so there is no padding between fields.
    /// 
    /// # When to Update
    /// 
    /// If `pqc_dilithium` is updated, you MUST:
    /// 1. Verify the new Keypair struct layout in the crate's source
    /// 2. Update the compile-time size assertion if needed
    /// 3. Run the full test suite including property-based tests
    /// 4. Update the version pin in Cargo.toml
    /// 5. Update this documentation with the new layout details
    /// 
    /// # Alternative
    /// 
    /// For production deployments on platforms that support it, consider using
    /// the `pqcrypto-backend` feature which provides a safe API via C bindings.
    #[cfg(not(feature = "pqcrypto-backend"))]
    fn sign_dilithium_safe(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Validate byte lengths before reconstruction
        if self.pq_signing_key.len() != DIL_SECRETKEYBYTES {
            return Err(CryptoError::InvalidInput(format!(
                "invalid signing key length: expected {}, got {}",
                DIL_SECRETKEYBYTES,
                self.pq_signing_key.len()
            )));
        }
        if self.pq_verifying_key.len() != DIL_PUBLICKEYBYTES {
            return Err(CryptoError::InvalidInput(format!(
                "invalid verifying key length: expected {}, got {}",
                DIL_PUBLICKEYBYTES,
                self.pq_verifying_key.len()
            )));
        }

        // Compile-time assertion: verify Keypair struct size matches our expectation
        // This will fail to compile if pqc_dilithium changes its Keypair layout
        // Expected: DIL_PUBLICKEYBYTES (1952) + DIL_SECRETKEYBYTES (4016) = 5968 bytes
        const _: () = assert!(
            std::mem::size_of::<DilithiumKeypair>() == DIL_PUBLICKEYBYTES + DIL_SECRETKEYBYTES,
            "pqc_dilithium Keypair size changed - update required"
        );

        // Reconstruct keypair bytes in the expected layout: [public][secret]
        let mut keypair_bytes = [0u8; DIL_PUBLICKEYBYTES + DIL_SECRETKEYBYTES];
        keypair_bytes[..DIL_PUBLICKEYBYTES].copy_from_slice(&self.pq_verifying_key[..DIL_PUBLICKEYBYTES]);
        keypair_bytes[DIL_PUBLICKEYBYTES..].copy_from_slice(self.pq_signing_key.as_slice());

        // SAFETY: This unsafe block is sound because:
        // 1. Byte lengths are validated above (invariant 3)
        // 2. Compile-time assertion verifies struct size (invariant 2)
        // 3. pqc_dilithium version is pinned to =0.2.0 (invariant 1)
        // 4. DilithiumKeypair layout is { public: [u8; 1952], secret: [u8; 4016] }
        // 5. std::ptr::read handles any alignment requirements
        // 6. The resulting Keypair is immediately used and not stored
        let keypair: DilithiumKeypair = unsafe {
            std::ptr::read(keypair_bytes.as_ptr() as *const DilithiumKeypair)
        };

        Ok(keypair.sign(data).to_vec())
    }

    /// Sign data using hybrid signatures (Dilithium3 + Ed25519)
    #[cfg(not(feature = "pqcrypto-backend"))]
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Dilithium3 signature using safe method
        let pq_sig = self.sign_dilithium_safe(data)?;

        // Ed25519 signature
        let ed_sign_key = SigningKey::from_bytes(self.ed_signing_key.as_bytes());
        let ed_sig = ed_sign_key.sign(data);

        // Combine signatures: [pq_sig_len (4 bytes)][pq_sig][ed_sig]
        let mut combined = Vec::with_capacity(4 + pq_sig.len() + 64);
        combined.extend_from_slice(&(pq_sig.len() as u32).to_le_bytes());
        combined.extend_from_slice(&pq_sig);
        combined.extend_from_slice(&ed_sig.to_bytes());
        Ok(combined)
    }

    /// Sign data using hybrid signatures - pqcrypto backend
    #[cfg(feature = "pqcrypto-backend")]
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Dilithium3 signature using pqcrypto
        let pq_sign_key = dilithium3::SecretKey::from_bytes(self.pq_signing_key.as_slice())
            .map_err(|_| CryptoError::InvalidInput("invalid dilithium secret key".into()))?;
        let pq_sig = dilithium3::detached_sign(data, &pq_sign_key);

        // Ed25519 signature
        let ed_sign_key = SigningKey::from_bytes(self.ed_signing_key.as_bytes());
        let ed_sig = ed_sign_key.sign(data);

        // Combine signatures
        let pq_sig_bytes = pq_sig.as_bytes();
        let mut combined = Vec::with_capacity(4 + pq_sig_bytes.len() + 64);
        combined.extend_from_slice(&(pq_sig_bytes.len() as u32).to_le_bytes());
        combined.extend_from_slice(pq_sig_bytes);
        combined.extend_from_slice(&ed_sig.to_bytes());
        Ok(combined)
    }
}


// ============================================================================
// PublicBundle Implementation - Signature Verification
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
        let ed_sig_arr: [u8; 64] = ed_sig_bytes
            .try_into()
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let ed_sig = Signature::from_bytes(&ed_sig_arr);
        ed_verify_key
            .verify(data, &ed_sig)
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
        let pq_verify_key = dilithium3::PublicKey::from_bytes(&self.pq_verify)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let pq_sig = dilithium3::DetachedSignature::from_bytes(pq_sig_bytes)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        dilithium3::verify_detached_signature(&pq_sig, data, &pq_verify_key)
            .map_err(|_| CryptoError::SignatureInvalid)?;

        // Verify Ed25519 signature
        let ed_verify_key = VerifyingKey::from_bytes(&self.ed_verify)
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let ed_sig_arr: [u8; 64] = ed_sig_bytes
            .try_into()
            .map_err(|_| CryptoError::SignatureInvalid)?;
        let ed_sig = Signature::from_bytes(&ed_sig_arr);
        ed_verify_key
            .verify(data, &ed_sig)
            .map_err(|_| CryptoError::SignatureInvalid)?;

        Ok(())
    }
}

// ============================================================================
// Hybrid Key Derivation
// ============================================================================

/// Derive a symmetric key from hybrid shared secrets
fn derive_hybrid_key(pq_ss: &[u8], x25519_ss: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HYBRID_KDF_DOMAIN);
    hasher.update(pq_ss);
    hasher.update(x25519_ss);
    *hasher.finalize().as_bytes()
}

// ============================================================================
// Hybrid Encryption with AAD Support
// ============================================================================

/// Encrypt data for a recipient using hybrid PQ + classical key exchange
/// Optionally binds Associated Authenticated Data (AAD) to prevent ciphertext substitution
#[cfg(not(feature = "pqcrypto-backend"))]
pub fn encrypt_with_aad(
    data: &[u8],
    recipient: &PublicBundle,
    aad: Option<&[u8]>,
) -> Result<EncryptedPayload, CryptoError> {
    let mut rng = OsRng;

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
    rng.fill_bytes(&mut nonce_bytes);

    // Encrypt with or without AAD
    let (ciphertext, aad_hash) = if let Some(aad_data) = aad {
        let ct = cipher
            .encrypt(
                Nonce::from_slice(&nonce_bytes),
                Payload {
                    msg: data,
                    aad: aad_data,
                },
            )
            .map_err(|_| CryptoError::Encrypt("AEAD encryption failed".into()))?;
        let hash = *blake3::hash(aad_data).as_bytes();
        (ct, Some(hash))
    } else {
        let ct = cipher
            .encrypt(Nonce::from_slice(&nonce_bytes), data)
            .map_err(|_| CryptoError::Encrypt("encryption failed".into()))?;
        (ct, None)
    };

    Ok(EncryptedPayload {
        nonce: nonce_bytes,
        ciphertext,
        encap: EncapsulatedKey {
            pq_ciphertext: pq_ciphertext.to_vec(),
            x25519_ephemeral: x_ephemeral_pub.to_bytes(),
        },
        aad_hash,
    })
}

/// Encrypt without AAD (backward compatible)
#[cfg(not(feature = "pqcrypto-backend"))]
pub fn encrypt(data: &[u8], recipient: &PublicBundle) -> Result<EncryptedPayload, CryptoError> {
    encrypt_with_aad(data, recipient, None)
}

/// Encrypt with AAD - pqcrypto backend
#[cfg(feature = "pqcrypto-backend")]
pub fn encrypt_with_aad(
    data: &[u8],
    recipient: &PublicBundle,
    aad: Option<&[u8]>,
) -> Result<EncryptedPayload, CryptoError> {
    let mut rng = OsRng;

    // ML-KEM encapsulation using pqcrypto
    let pq_encap_key = mlkem1024::PublicKey::from_bytes(&recipient.pq_encap)
        .map_err(|_| CryptoError::KeyExchange("invalid ML-KEM public key".into()))?;
    let (pq_shared_secret, pq_ciphertext) = mlkem1024::encapsulate(&pq_encap_key);

    // X25519 key exchange
    let x_ephemeral = StaticSecret::random_from_rng(&mut rng);
    let x_ephemeral_pub = X25519Public::from(&x_ephemeral);
    let x_recipient = X25519Public::from(recipient.x25519);
    let x_ss = x_ephemeral.diffie_hellman(&x_recipient);

    // Derive symmetric key
    let key = derive_hybrid_key(pq_shared_secret.as_bytes(), x_ss.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key.into());

    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    rng.fill_bytes(&mut nonce_bytes);

    // Encrypt with or without AAD
    let (ciphertext, aad_hash) = if let Some(aad_data) = aad {
        let ct = cipher
            .encrypt(
                Nonce::from_slice(&nonce_bytes),
                Payload {
                    msg: data,
                    aad: aad_data,
                },
            )
            .map_err(|_| CryptoError::Encrypt("AEAD encryption failed".into()))?;
        let hash = *blake3::hash(aad_data).as_bytes();
        (ct, Some(hash))
    } else {
        let ct = cipher
            .encrypt(Nonce::from_slice(&nonce_bytes), data)
            .map_err(|_| CryptoError::Encrypt("encryption failed".into()))?;
        (ct, None)
    };

    Ok(EncryptedPayload {
        nonce: nonce_bytes,
        ciphertext,
        encap: EncapsulatedKey {
            pq_ciphertext: pq_ciphertext.as_bytes().to_vec(),
            x25519_ephemeral: x_ephemeral_pub.to_bytes(),
        },
        aad_hash,
    })
}

#[cfg(feature = "pqcrypto-backend")]
pub fn encrypt(data: &[u8], recipient: &PublicBundle) -> Result<EncryptedPayload, CryptoError> {
    encrypt_with_aad(data, recipient, None)
}


// ============================================================================
// Hybrid Decryption with AAD Verification
// ============================================================================

/// Decrypt data with optional AAD verification - Pure Rust
#[cfg(not(feature = "pqcrypto-backend"))]
pub fn decrypt_with_aad(
    payload: &EncryptedPayload,
    keypair: &HybridKeypair,
    aad: Option<&[u8]>,
) -> Result<Vec<u8>, CryptoError> {
    // Verify AAD hash if present
    if let Some(expected_hash) = &payload.aad_hash {
        match aad {
            Some(aad_data) => {
                let actual_hash = blake3::hash(aad_data);
                if actual_hash.as_bytes() != expected_hash {
                    return Err(CryptoError::AadMismatch);
                }
            }
            None => return Err(CryptoError::AadMismatch),
        }
    }

    // Kyber decapsulation
    let mut ct = [0u8; KYBER_CIPHERTEXTBYTES];
    ct.copy_from_slice(&payload.encap.pq_ciphertext[..KYBER_CIPHERTEXTBYTES]);
    let mut sk = [0u8; KYBER_SECRETKEYBYTES];
    sk.copy_from_slice(&keypair.pq_decap_key.as_slice()[..KYBER_SECRETKEYBYTES]);
    let pq_shared_secret = decapsulate(&ct, &sk)
        .map_err(|_| CryptoError::KeyExchange("Kyber decapsulation failed".into()))?;

    // X25519 key exchange
    let x_secret = StaticSecret::from(*keypair.x25519_secret.as_bytes());
    let x_ephemeral = X25519Public::from(payload.encap.x25519_ephemeral);
    let x_ss = x_secret.diffie_hellman(&x_ephemeral);

    // Derive symmetric key
    let key = derive_hybrid_key(&pq_shared_secret, x_ss.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key.into());

    // Decrypt with or without AAD
    if let Some(aad_data) = aad {
        cipher
            .decrypt(
                Nonce::from_slice(&payload.nonce),
                Payload {
                    msg: payload.ciphertext.as_ref(),
                    aad: aad_data,
                },
            )
            .map_err(|_| CryptoError::Decrypt("authentication failed".into()))
    } else {
        cipher
            .decrypt(
                Nonce::from_slice(&payload.nonce),
                payload.ciphertext.as_ref(),
            )
            .map_err(|_| CryptoError::Decrypt("authentication failed".into()))
    }
}

#[cfg(not(feature = "pqcrypto-backend"))]
pub fn decrypt(payload: &EncryptedPayload, keypair: &HybridKeypair) -> Result<Vec<u8>, CryptoError> {
    decrypt_with_aad(payload, keypair, None)
}

/// Decrypt with AAD - pqcrypto backend
#[cfg(feature = "pqcrypto-backend")]
pub fn decrypt_with_aad(
    payload: &EncryptedPayload,
    keypair: &HybridKeypair,
    aad: Option<&[u8]>,
) -> Result<Vec<u8>, CryptoError> {
    // Verify AAD hash if present
    if let Some(expected_hash) = &payload.aad_hash {
        match aad {
            Some(aad_data) => {
                let actual_hash = blake3::hash(aad_data);
                if actual_hash.as_bytes() != expected_hash {
                    return Err(CryptoError::AadMismatch);
                }
            }
            None => return Err(CryptoError::AadMismatch),
        }
    }

    // ML-KEM decapsulation
    let pq_decap_key = mlkem1024::SecretKey::from_bytes(keypair.pq_decap_key.as_slice())
        .map_err(|_| CryptoError::KeyExchange("invalid ML-KEM secret key".into()))?;
    let pq_ciphertext = mlkem1024::Ciphertext::from_bytes(&payload.encap.pq_ciphertext)
        .map_err(|_| CryptoError::KeyExchange("invalid ML-KEM ciphertext".into()))?;
    let pq_shared_secret = mlkem1024::decapsulate(&pq_ciphertext, &pq_decap_key);

    // X25519 key exchange
    let x_secret = StaticSecret::from(*keypair.x25519_secret.as_bytes());
    let x_ephemeral = X25519Public::from(payload.encap.x25519_ephemeral);
    let x_ss = x_secret.diffie_hellman(&x_ephemeral);

    // Derive symmetric key
    let key = derive_hybrid_key(pq_shared_secret.as_bytes(), x_ss.as_bytes());
    let cipher = ChaCha20Poly1305::new(&key.into());

    // Decrypt with or without AAD
    if let Some(aad_data) = aad {
        cipher
            .decrypt(
                Nonce::from_slice(&payload.nonce),
                Payload {
                    msg: payload.ciphertext.as_ref(),
                    aad: aad_data,
                },
            )
            .map_err(|_| CryptoError::Decrypt("authentication failed".into()))
    } else {
        cipher
            .decrypt(
                Nonce::from_slice(&payload.nonce),
                payload.ciphertext.as_ref(),
            )
            .map_err(|_| CryptoError::Decrypt("authentication failed".into()))
    }
}

#[cfg(feature = "pqcrypto-backend")]
pub fn decrypt(payload: &EncryptedPayload, keypair: &HybridKeypair) -> Result<Vec<u8>, CryptoError> {
    decrypt_with_aad(payload, keypair, None)
}

// ============================================================================
// OS Keychain Integration
// ============================================================================

/// Store a value in the OS keychain
/// 
/// Supported backends:
/// - macOS: Keychain Services
/// - Windows: Credential Manager  
/// - Linux: Secret Service (GNOME Keyring, KWallet)
/// 
/// Falls back to machine-key encryption if keychain unavailable
pub fn keychain_store(key: &str, value: &[u8]) -> Result<(), CryptoError> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, key)
        .map_err(|e| {
            log::debug!("Failed to create keychain entry '{}': {}", key, e);
            CryptoError::Keychain(format!("failed to create entry: {}", e))
        })?;

    // Store as base64 since keyring expects strings
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, value);
    
    entry
        .set_password(&encoded)
        .map_err(|e| {
            log::warn!("Keychain storage failed for '{}': {}", key, e);
            CryptoError::Keychain(format!("failed to store: {}", e))
        })?;
    
    log::debug!("Successfully stored '{}' in OS keychain", key);
    Ok(())
}

/// Retrieve a value from the OS keychain
pub fn keychain_retrieve(key: &str) -> Result<Vec<u8>, CryptoError> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, key)
        .map_err(|e| {
            log::debug!("Failed to create keychain entry for retrieval '{}': {}", key, e);
            CryptoError::Keychain(format!("failed to create entry: {}", e))
        })?;

    let encoded = entry
        .get_password()
        .map_err(|e| {
            log::debug!("Failed to retrieve '{}' from keychain: {}", key, e);
            CryptoError::Keychain(format!("failed to retrieve: {}", e))
        })?;

    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encoded)
        .map_err(|e| {
            log::warn!("Failed to decode keychain value for '{}': {}", key, e);
            CryptoError::Keychain(format!("failed to decode: {}", e))
        })
}

/// Delete a value from the OS keychain
pub fn keychain_delete(key: &str) -> Result<(), CryptoError> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, key)
        .map_err(|e| CryptoError::Keychain(format!("failed to create entry: {}", e)))?;

    entry
        .delete_password()
        .map_err(|e| {
            log::debug!("Failed to delete '{}' from keychain: {}", key, e);
            CryptoError::Keychain(format!("failed to delete: {}", e))
        })?;
    
    log::debug!("Successfully deleted '{}' from OS keychain", key);
    Ok(())
}

/// Check if keychain is available and functional on this platform
/// 
/// This performs a lightweight test to verify the keychain backend is working.
/// Returns false if:
/// - No keychain backend is available (e.g., headless Linux without Secret Service)
/// - The keychain is locked and requires user interaction
/// - There are permission issues
pub fn keychain_available() -> bool {
    match keyring::Entry::new(KEYCHAIN_SERVICE, "__keychain_test__") {
        Ok(entry) => {
            // Try to set and delete a test value
            let test_result = entry.set_password("test")
                .and_then(|_| entry.delete_password());
            
            match test_result {
                Ok(_) => {
                    log::debug!("OS keychain is available and functional");
                    true
                }
                Err(e) => {
                    log::debug!("OS keychain test failed: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            log::debug!("OS keychain not available: {}", e);
            false
        }
    }
}

// ============================================================================
// Token Encryption v4 with AAD
// ============================================================================

/// Context for token encryption AAD
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenContext {
    pub service_id: String,
    pub timestamp: u64,
}

impl TokenContext {
    pub fn new() -> Self {
        Self {
            service_id: KEYCHAIN_SERVICE.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    pub fn to_aad(&self) -> Vec<u8> {
        let mut aad = Vec::new();
        aad.extend_from_slice(self.service_id.as_bytes());
        aad.extend_from_slice(&self.timestamp.to_le_bytes());
        aad
    }
}

impl Default for TokenContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Get machine key with enhanced entropy for fallback
/// 
/// SECURITY WARNING: This is a fallback when keychain is unavailable.
/// The key is derived from machine identifiers which may be predictable.
/// 
/// Security levels:
/// - STRONG: /etc/machine-id (Linux), IOPlatformUUID (macOS), MachineGuid (Windows)
/// - WEAK: hostname, username, home directory (logged with warning)
/// 
/// The random salt parameter ensures unique keys even with weak identifiers,
/// mitigating key reuse attacks. However, an attacker with local access
/// could potentially derive the key on systems without strong identifiers.
pub(crate) fn get_machine_key_with_salt(salt: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(TOKEN_KDF_DOMAIN);
    hasher.update(salt);

    // Try to get strong machine identifier
    let mut has_strong_id = false;

    // Linux: /etc/machine-id (systemd standard, highly reliable)
    #[cfg(target_os = "linux")]
    if let Ok(machine_id) = std::fs::read_to_string("/etc/machine-id") {
        let trimmed = machine_id.trim();
        if !trimmed.is_empty() && trimmed.len() >= 32 {
            hasher.update(trimmed.as_bytes());
            has_strong_id = true;
            log::debug!("Using /etc/machine-id for token encryption key");
        }
    }

    // macOS: IOPlatformUUID (hardware UUID, very reliable)
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                if let Some(uuid_line) = stdout.lines().find(|l| l.contains("IOPlatformUUID")) {
                    // Extract just the UUID value
                    if let Some(uuid) = uuid_line.split('"').nth(3) {
                        hasher.update(uuid.as_bytes());
                        has_strong_id = true;
                        log::debug!("Using IOPlatformUUID for token encryption key");
                    }
                }
            }
        }
    }

    // Windows: MachineGuid from registry
    #[cfg(target_os = "windows")]
    {
        // Try to read MachineGuid from registry via reg query
        if let Ok(output) = std::process::Command::new("reg")
            .args(["query", "HKLM\\SOFTWARE\\Microsoft\\Cryptography", "/v", "MachineGuid"])
            .output()
        {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                // Parse the output to extract the GUID
                for line in stdout.lines() {
                    if line.contains("MachineGuid") {
                        if let Some(guid) = line.split_whitespace().last() {
                            hasher.update(guid.as_bytes());
                            has_strong_id = true;
                            log::debug!("Using Windows MachineGuid for token encryption key");
                        }
                    }
                }
            }
        }
    }

    // Fallback: use weaker identifiers with warning
    if !has_strong_id {
        log::warn!(
            "SECURITY WARNING: Using weak machine identifiers for token encryption. \
             This may occur in containers, VMs, or systems without /etc/machine-id. \
             Consider using OS keychain for better security. \
             Tokens encrypted with weak identifiers may be vulnerable to local attacks."
        );
        hasher.update(b"WEAK_FALLBACK_WITH_SALT_V2");

        // Add multiple entropy sources to increase difficulty
        if let Ok(hostname) = std::env::var("HOSTNAME").or_else(|_| std::env::var("COMPUTERNAME")) {
            hasher.update(hostname.as_bytes());
        }
        if let Some(home) = dirs::home_dir() {
            hasher.update(home.to_string_lossy().as_bytes());
        }
        if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
            hasher.update(user.as_bytes());
        }
        // Add process start time for additional entropy
        hasher.update(&std::process::id().to_le_bytes());
        // Add current time as additional entropy (makes offline attacks harder)
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        hasher.update(&time.to_le_bytes());
    }

    *hasher.finalize().as_bytes()
}

/// Encrypt a token using v4 format with AAD
/// Format: [version: 1][salt: 32][nonce: 12][aad_len: 2][aad: var][ciphertext: var]
pub fn encrypt_token_v4(plaintext: &str, context: &TokenContext) -> Result<Vec<u8>, CryptoError> {
    let mut rng = OsRng;

    // Generate random salt
    let mut salt = [0u8; 32];
    rng.fill_bytes(&mut salt);

    // Derive key from machine key + salt
    let key = get_machine_key_with_salt(&salt);
    let cipher = ChaCha20Poly1305::new(&key.into());

    // Generate random nonce
    let mut nonce = [0u8; 12];
    rng.fill_bytes(&mut nonce);

    // Create AAD from context
    let aad = context.to_aad();

    // Encrypt with AAD
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: plaintext.as_bytes(),
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError::Encrypt("token encryption failed".into()))?;

    // Build token: [version][salt][nonce][aad_len][aad][ciphertext]
    let mut token = Vec::with_capacity(1 + 32 + 12 + 2 + aad.len() + ciphertext.len());
    token.push(TOKEN_VERSION_V4);
    token.extend_from_slice(&salt);
    token.extend_from_slice(&nonce);
    token.extend_from_slice(&(aad.len() as u16).to_le_bytes());
    token.extend_from_slice(&aad);
    token.extend_from_slice(&ciphertext);

    Ok(token)
}

/// Decrypt a v4 token
fn decrypt_token_v4(data: &[u8]) -> Result<String, CryptoError> {
    // Minimum size: version(1) + salt(32) + nonce(12) + aad_len(2) + tag(16)
    if data.len() < 63 {
        return Err(CryptoError::InvalidInput("token too short".into()));
    }

    let salt = &data[1..33];
    let nonce = &data[33..45];
    let aad_len = u16::from_le_bytes([data[45], data[46]]) as usize;

    // Use saturating arithmetic to prevent overflow on 32-bit systems
    // when aad_len is maliciously large (e.g., u16::MAX = 65535)
    if aad_len > data.len().saturating_sub(47) {
        return Err(CryptoError::InvalidInput("invalid aad length".into()));
    }

    let aad = &data[47..47 + aad_len];
    let ciphertext = &data[47 + aad_len..];

    // Derive key
    let key = get_machine_key_with_salt(salt);
    let cipher = ChaCha20Poly1305::new(&key.into());

    // Decrypt with AAD
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| CryptoError::Decrypt("token decryption failed".into()))?;

    String::from_utf8(plaintext)
        .map_err(|_| CryptoError::Decrypt("invalid utf8 in token".into()))
}


/// Decrypt a v3 token (legacy format without AAD)
fn decrypt_token_v3(data: &[u8]) -> Result<String, CryptoError> {
    // v3 format: [version: 1][salt: 32][nonce: 12][ciphertext: var]
    if data.len() < 45 {
        return Err(CryptoError::InvalidInput("token too short".into()));
    }

    let salt = &data[1..33];
    let nonce = &data[33..45];
    let ciphertext = &data[45..];

    let key = get_machine_key_with_salt(salt);
    let cipher = ChaCha20Poly1305::new(&key.into());

    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| CryptoError::Decrypt("token decryption failed".into()))?;

    String::from_utf8(plaintext)
        .map_err(|_| CryptoError::Decrypt("invalid utf8 in token".into()))
}

/// Decrypt a v2 token (legacy format with deterministic key - INSECURE)
fn decrypt_token_v2(data: &[u8]) -> Result<String, CryptoError> {
    // v2 format: [version: 1][nonce: 12][ciphertext: var]
    // Uses deterministic machine key without salt - vulnerable to key reuse
    if data.len() < 29 {
        return Err(CryptoError::InvalidInput("token too short".into()));
    }

    let nonce = &data[1..13];
    let ciphertext = &data[13..];

    // v2 used deterministic key without salt
    let key = get_machine_key_with_salt(&[0u8; 32]);
    let cipher = ChaCha20Poly1305::new(&key.into());

    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| CryptoError::Decrypt("token decryption failed".into()))?;

    String::from_utf8(plaintext)
        .map_err(|_| CryptoError::Decrypt("invalid utf8 in token".into()))
}

/// Decrypt a token, automatically handling version migration
/// Returns (plaintext, Option<upgraded_token>)
/// If upgraded_token is Some, the caller should store it to replace the old token
pub fn decrypt_token(data: &[u8]) -> Result<(String, Option<Vec<u8>>), CryptoError> {
    if data.is_empty() {
        return Err(CryptoError::InvalidInput("empty token".into()));
    }

    let version = data[0];
    match version {
        TOKEN_VERSION_V4 => {
            let plaintext = decrypt_token_v4(data)?;
            Ok((plaintext, None))
        }
        TOKEN_VERSION_V3 => {
            let plaintext = decrypt_token_v3(data)?;
            // Auto-migrate to v4
            let context = TokenContext::new();
            let upgraded = encrypt_token_v4(&plaintext, &context)?;
            log::info!("Migrated v3 token to v4 format");
            Ok((plaintext, Some(upgraded)))
        }
        TOKEN_VERSION_V2 => {
            let plaintext = decrypt_token_v2(data)?;
            // Auto-migrate to v4
            let context = TokenContext::new();
            let upgraded = encrypt_token_v4(&plaintext, &context)?;
            log::info!("Migrated v2 token to v4 format");
            Ok((plaintext, Some(upgraded)))
        }
        _ => Err(CryptoError::UnsupportedTokenVersion(version)),
    }
}

/// Encrypt a token (always uses v4 format)
pub fn encrypt_token(plaintext: &str) -> Result<Vec<u8>, CryptoError> {
    let context = TokenContext::new();
    encrypt_token_v4(plaintext, &context)
}

// ============================================================================
// Password-Based Encryption
// ============================================================================

/// Encrypt data with a password using Argon2id + ChaCha20-Poly1305
pub fn encrypt_with_password(data: &[u8], password: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mut rng = OsRng;

    // Generate random salt
    let mut salt = [0u8; 16];
    rng.fill_bytes(&mut salt);

    // Derive key using Argon2id
    let mut key = [0u8; 32];
    get_argon2()
        .hash_password_into(password, &salt, &mut key)
        .map_err(|_| CryptoError::KeyDerivation("argon2 failed".into()))?;

    let cipher = ChaCha20Poly1305::new(&key.into());

    // Generate random nonce
    let mut nonce = [0u8; 12];
    rng.fill_bytes(&mut nonce);

    // Encrypt
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), data)
        .map_err(|_| CryptoError::Encrypt("encryption failed".into()))?;

    // Zeroize key
    key.zeroize();

    // Output: [salt: 16][nonce: 12][ciphertext: var]
    let mut out = Vec::with_capacity(16 + 12 + ciphertext.len());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt data with a password
pub fn decrypt_with_password(data: &[u8], password: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if data.len() < 28 {
        return Err(CryptoError::InvalidInput("data too short".into()));
    }

    let salt = &data[..16];
    let nonce = &data[16..28];
    let ciphertext = &data[28..];

    // Derive key
    let mut key = [0u8; 32];
    get_argon2()
        .hash_password_into(password, salt, &mut key)
        .map_err(|_| CryptoError::KeyDerivation("argon2 failed".into()))?;

    let cipher = ChaCha20Poly1305::new(&key.into());

    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| CryptoError::Decrypt("wrong password or corrupted data".into()))?;

    // Zeroize key
    key.zeroize();

    Ok(plaintext)
}

// ============================================================================
// Session Keys (HKDF-SHA512)
// ============================================================================

/// Session keys derived from a shared secret using HKDF-SHA512
/// Used for establishing secure communication channels
/// NOTE: Implements Zeroize to clear sensitive key material from memory
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
#[allow(dead_code)]
pub struct SessionKeys {
    pub encryption_key: [u8; 32],
    pub hmac_key: [u8; 32],
    pub iv: [u8; 12],
}

#[allow(dead_code)]
impl SessionKeys {
    pub fn derive_from_secret(shared_secret: &[u8]) -> Result<Self, CryptoError> {
        let hk = Hkdf::<Sha512>::new(Some(SESSION_KDF_DOMAIN), shared_secret);

        let mut encryption_key = [0u8; 32];
        let mut hmac_key = [0u8; 32];
        let mut iv = [0u8; 12];

        hk.expand(b"encryption", &mut encryption_key)
            .map_err(|_| CryptoError::KeyDerivation("hkdf expand failed".into()))?;
        hk.expand(b"hmac", &mut hmac_key)
            .map_err(|_| CryptoError::KeyDerivation("hkdf expand failed".into()))?;
        hk.expand(b"iv", &mut iv)
            .map_err(|_| CryptoError::KeyDerivation("hkdf expand failed".into()))?;

        Ok(Self {
            encryption_key,
            hmac_key,
            iv,
        })
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Hash data using BLAKE3
pub fn hash_data(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// Check if pqcrypto backend is available
pub fn is_pqcrypto_backend() -> bool {
    #[cfg(feature = "pqcrypto-backend")]
    {
        true
    }
    #[cfg(not(feature = "pqcrypto-backend"))]
    {
        false
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Generate a new keypair and return opaque handle + public bundle
#[tauri::command]
pub fn generate_keypair() -> Result<KeypairInfo, CryptoError> {
    let keypair = HybridKeypair::generate()?;
    let public_bundle = keypair.public_bundle();
    let created_at = keypair.created_at;
    let key_id = public_bundle.key_id.clone();

    // Handle lock poisoning gracefully instead of panicking
    let handle = KEYPAIR_STORE
        .write()
        .map_err(|_| CryptoError::KeyGeneration("keypair store lock poisoned".into()))?
        .insert(keypair);

    Ok(KeypairInfo {
        handle,
        public_bundle,
        created_at,
        key_id,
    })
}

/// Release a keypair handle (removes from memory)
#[tauri::command]
pub fn release_keypair(handle: KeypairHandle) -> Result<(), CryptoError> {
    KEYPAIR_STORE
        .write()
        .map_err(|_| CryptoError::KeyGeneration("keypair store lock poisoned".into()))?
        .remove(handle)
        .ok_or(CryptoError::KeypairNotFound)?;
    Ok(())
}

/// Rotate a keypair (generate new, keep old for decryption)
#[tauri::command]
pub fn rotate_keypair(handle: KeypairHandle) -> Result<PublicBundle, CryptoError> {
    KEYPAIR_STORE
        .write()
        .map_err(|_| CryptoError::KeyGeneration("keypair store lock poisoned".into()))?
        .rotate(handle)
}

/// Validate that a keypair handle is still valid in the store
/// 
/// Used by frontend to verify stored handles before attempting crypto operations.
/// Returns true if the handle exists, false otherwise.
#[tauri::command]
pub fn validate_keypair_handle(handle: KeypairHandle) -> Result<bool, CryptoError> {
    let store = KEYPAIR_STORE
        .read()
        .map_err(|_| CryptoError::KeyGeneration("keypair store lock poisoned".into()))?;
    Ok(store.get(handle).is_some())
}

/// Sign data using a keypair handle
#[tauri::command]
pub fn sign_data(data: Vec<u8>, handle: KeypairHandle) -> Result<Vec<u8>, CryptoError> {
    let store = KEYPAIR_STORE
        .read()
        .map_err(|_| CryptoError::KeyGeneration("keypair store lock poisoned".into()))?;
    let keypair_arc = store.get(handle).ok_or(CryptoError::KeypairNotFound)?;
    let keypair = keypair_arc
        .lock()
        .map_err(|_| CryptoError::KeyGeneration("keypair mutex poisoned".into()))?;
    keypair.sign(&data)
}

/// Verify a signature using a public bundle
#[tauri::command]
pub fn verify_signature(
    data: Vec<u8>,
    signature: Vec<u8>,
    public_bundle: PublicBundle,
) -> Result<bool, CryptoError> {
    match public_bundle.verify(&data, &signature) {
        Ok(()) => Ok(true),
        Err(CryptoError::SignatureInvalid) => Ok(false),
        Err(e) => Err(e),
    }
}

/// Encrypt data for a recipient
#[tauri::command]
pub fn encrypt_hybrid(
    data: Vec<u8>,
    recipient_bundle: PublicBundle,
    aad: Option<Vec<u8>>,
) -> Result<EncryptedPayload, CryptoError> {
    encrypt_with_aad(&data, &recipient_bundle, aad.as_deref())
}

/// Decrypt data using a keypair handle (tries current + rotated keys)
#[tauri::command]
pub fn decrypt_hybrid(
    encrypted_data: EncryptedPayload,
    handle: KeypairHandle,
    aad: Option<Vec<u8>>,
) -> Result<Vec<u8>, CryptoError> {
    let store = KEYPAIR_STORE
        .read()
        .map_err(|_| CryptoError::Decrypt("keypair store lock poisoned".into()))?;
    let keypairs = store.get_all_for_decryption(handle);

    if keypairs.is_empty() {
        return Err(CryptoError::KeypairNotFound);
    }

    // Try each keypair (current first, then rotated)
    for keypair_arc in keypairs {
        if let Ok(keypair) = keypair_arc.lock() {
            if let Ok(plaintext) = decrypt_with_aad(&encrypted_data, &keypair, aad.as_deref()) {
                return Ok(plaintext);
            }
        }
    }

    Err(CryptoError::Decrypt(
        "decryption failed with all available keys".into(),
    ))
}

/// Encrypt data with password
#[tauri::command]
pub fn encrypt_data_password(data: Vec<u8>, password: String) -> Result<Vec<u8>, CryptoError> {
    encrypt_with_password(&data, password.as_bytes())
}

/// Decrypt data with password
#[tauri::command]
pub fn decrypt_data_password(data: Vec<u8>, password: String) -> Result<Vec<u8>, CryptoError> {
    decrypt_with_password(&data, password.as_bytes())
}

/// Store a token securely (tries keychain first, falls back to machine-key)
/// 
/// Storage priority:
/// 1. OS Keychain (most secure - uses platform-specific secure storage)
/// 2. Encrypted file with machine-key (fallback - less secure on weak systems)
#[tauri::command]
pub fn secure_store_token(key: String, value: String) -> Result<(), CryptoError> {
    // Try keychain first (preferred)
    if keychain_available() {
        match keychain_store(&key, value.as_bytes()) {
            Ok(()) => {
                log::info!("Token '{}' stored in OS keychain", key);
                return Ok(());
            }
            Err(e) => {
                log::warn!("Keychain storage failed for '{}', falling back to file: {}", key, e);
            }
        }
    } else {
        log::info!("OS keychain not available, using encrypted file storage for '{}'", key);
    }

    // Fall back to encrypted file storage
    let encrypted = encrypt_token(&value)?;
    let path = dirs::data_local_dir()
        .ok_or_else(|| CryptoError::Keychain("no local data dir".into()))?
        .join("vortex-image")
        .join("tokens");

    std::fs::create_dir_all(&path)
        .map_err(|e| CryptoError::Keychain(format!("failed to create dir: {}", e)))?;

    std::fs::write(path.join(&key), &encrypted)
        .map_err(|e| CryptoError::Keychain(format!("failed to write: {}", e)))?;

    log::info!("Token '{}' stored in encrypted file (keychain unavailable)", key);
    Ok(())
}

/// Retrieve a token securely
/// 
/// Retrieval priority:
/// 1. OS Keychain
/// 2. Encrypted file with machine-key
/// 
/// Automatically migrates legacy token formats (v2, v3) to v4
#[tauri::command]
pub fn secure_retrieve_token(key: String) -> Result<String, CryptoError> {
    // Try keychain first
    if keychain_available() {
        match keychain_retrieve(&key) {
            Ok(data) => {
                return String::from_utf8(data)
                    .map_err(|_| CryptoError::Keychain("invalid utf8".into()));
            }
            Err(e) => {
                log::debug!("Keychain retrieval failed for '{}', trying file: {}", key, e);
            }
        }
    }

    // Fall back to encrypted file storage
    let path = dirs::data_local_dir()
        .ok_or_else(|| CryptoError::Keychain("no local data dir".into()))?
        .join("vortex-image")
        .join("tokens")
        .join(&key);

    let encrypted = std::fs::read(&path)
        .map_err(|e| CryptoError::Keychain(format!("failed to read: {}", e)))?;

    let (plaintext, upgraded) = decrypt_token(&encrypted)?;

    // If token was upgraded from legacy format, save the new version
    if let Some(new_token) = upgraded {
        if let Err(e) = std::fs::write(&path, &new_token) {
            log::warn!("Failed to save upgraded token: {}", e);
        } else {
            log::info!("Token '{}' migrated to v4 format", key);
        }
    }

    Ok(plaintext)
}

/// Delete a token from secure storage
/// 
/// Removes from both keychain and file storage to ensure complete cleanup
#[tauri::command]
pub fn secure_delete_token(key: String) -> Result<(), CryptoError> {
    let mut deleted = false;
    
    // Try to delete from keychain
    if keychain_available() {
        if keychain_delete(&key).is_ok() {
            deleted = true;
            log::debug!("Deleted '{}' from keychain", key);
        }
    }
    
    // Also try to delete from file storage
    let path = dirs::data_local_dir()
        .ok_or_else(|| CryptoError::Keychain("no local data dir".into()))?
        .join("vortex-image")
        .join("tokens")
        .join(&key);
    
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| CryptoError::Keychain(format!("failed to delete file: {}", e)))?;
        deleted = true;
        log::debug!("Deleted '{}' from file storage", key);
    }
    
    if deleted {
        Ok(())
    } else {
        Err(CryptoError::Keychain(format!("token '{}' not found", key)))
    }
}

/// Hash data using BLAKE3
#[tauri::command]
pub fn hash_data_blake3(data: Vec<u8>) -> Vec<u8> {
    hash_data(&data).to_vec()
}

/// Get crypto module info
#[tauri::command]
pub fn get_crypto_info() -> serde_json::Value {
    let keychain_status = if keychain_available() {
        "available"
    } else {
        "unavailable (using encrypted file fallback)"
    };
    
    serde_json::json!({
        "key_exchange": "ML-KEM-1024 (Kyber) + X25519 hybrid",
        "signatures": "ML-DSA-65 (Dilithium) + Ed25519 hybrid",
        "symmetric": "ChaCha20-Poly1305 (AEAD with AAD)",
        "kdf": "Argon2id (password) + HKDF-SHA512 (session)",
        "hash": "BLAKE3",
        "pq_security_level": "NIST Level 5 (256-bit)",
        "classical_security_level": "128-bit",
        "token_version": "v4 (with AAD)",
        "keychain_status": keychain_status,
        "features": [
            "Opaque keypair handles",
            "Key rotation support",
            "OS keychain integration",
            "Automatic token migration (v2→v3→v4)",
            "AAD in AEAD encryption",
            "Zeroizing secret types",
            "Safe Dilithium signing (no unsafe transmute in pqcrypto backend)"
        ],
        "backend": if is_pqcrypto_backend() { "pqcrypto (optimized assembly)" } else { "pure-rust (iOS compatible)" },
        "pinned_versions": {
            "pqc_kyber": "=0.7.1",
            "pqc_dilithium": "=0.2.0"
        }
    })
}

// ============================================================================
// File Encryption (for backward compatibility)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EncryptionMethod {
    None,
    Password,
    HybridPQ,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptionSettings {
    pub enabled: bool,
    pub use_password: bool,
    pub use_keypair: bool,
    pub recipient_bundle: Option<PublicBundle>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedFileData {
    pub data: Vec<u8>,
    pub encrypted: bool,
    pub method: EncryptionMethod,
    pub metadata: Option<serde_json::Value>,
}

#[tauri::command]
pub fn encrypt_file(
    data: Vec<u8>,
    settings: EncryptionSettings,
    password: Option<String>,
    _handle: Option<KeypairHandle>, // Reserved for future use (e.g., signing)
) -> Result<EncryptedFileData, CryptoError> {
    if !settings.enabled {
        return Ok(EncryptedFileData {
            data,
            encrypted: false,
            method: EncryptionMethod::None,
            metadata: None,
        });
    }

    if settings.use_password {
        let pwd = password.ok_or_else(|| CryptoError::InvalidInput("password required".into()))?;
        let encrypted = encrypt_with_password(&data, pwd.as_bytes())?;
        return Ok(EncryptedFileData {
            data: encrypted,
            encrypted: true,
            method: EncryptionMethod::Password,
            metadata: None,
        });
    }

    if settings.use_keypair {
        let recipient = settings
            .recipient_bundle
            .ok_or_else(|| CryptoError::InvalidInput("recipient bundle required".into()))?;
        let payload = encrypt(&data, &recipient)?;
        let serialized = serde_json::to_vec(&payload)
            .map_err(|e| CryptoError::Encrypt(format!("serialization failed: {}", e)))?;
        return Ok(EncryptedFileData {
            data: serialized,
            encrypted: true,
            method: EncryptionMethod::HybridPQ,
            metadata: None,
        });
    }

    Ok(EncryptedFileData {
        data,
        encrypted: false,
        method: EncryptionMethod::None,
        metadata: None,
    })
}

#[tauri::command]
pub fn decrypt_file(
    encrypted: EncryptedFileData,
    password: Option<String>,
    handle: Option<KeypairHandle>,
) -> Result<Vec<u8>, CryptoError> {
    if !encrypted.encrypted {
        return Ok(encrypted.data);
    }

    match encrypted.method {
        EncryptionMethod::None => Ok(encrypted.data),
        EncryptionMethod::Password => {
            let pwd =
                password.ok_or_else(|| CryptoError::InvalidInput("password required".into()))?;
            decrypt_with_password(&encrypted.data, pwd.as_bytes())
        }
        EncryptionMethod::HybridPQ => {
            let h = handle.ok_or_else(|| CryptoError::InvalidInput("keypair handle required".into()))?;
            let payload: EncryptedPayload = serde_json::from_slice(&encrypted.data)
                .map_err(|e| CryptoError::Decrypt(format!("deserialization failed: {}", e)))?;

            let store = KEYPAIR_STORE
                .read()
                .map_err(|_| CryptoError::Decrypt("keypair store lock poisoned".into()))?;
            let keypairs = store.get_all_for_decryption(h);

            if keypairs.is_empty() {
                return Err(CryptoError::KeypairNotFound);
            }

            for keypair_arc in keypairs {
                if let Ok(keypair) = keypair_arc.lock() {
                    if let Ok(plaintext) = decrypt(&payload, &keypair) {
                        return Ok(plaintext);
                    }
                }
            }

            Err(CryptoError::Decrypt(
                "decryption failed with all available keys".into(),
            ))
        }
    }
}


// ============================================================================
// Legacy Compatibility Functions (for github.rs)
// ============================================================================

/// Legacy function to decrypt using keypair bytes directly
/// Used by github.rs for backward compatibility
pub fn decrypt_with_keypair_bytes(
    payload: &EncryptedPayload,
    keypair_bytes: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let keypair = HybridKeypair::from_bytes(keypair_bytes)?;
    decrypt(payload, &keypair)
}

/// Serialize keypair to bytes (for storage)
/// Used by decrypt_with_keypair_bytes for legacy compatibility
impl HybridKeypair {
    #[allow(dead_code)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        // PQ encapsulation key
        out.extend_from_slice(&(self.pq_encap_key.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.pq_encap_key);
        // PQ decapsulation key
        out.extend_from_slice(&(self.pq_decap_key.len() as u32).to_le_bytes());
        out.extend_from_slice(self.pq_decap_key.as_slice());
        // X25519 keys
        out.extend_from_slice(self.x25519_secret.as_bytes());
        out.extend_from_slice(&self.x25519_public);
        // PQ signing key
        out.extend_from_slice(&(self.pq_signing_key.len() as u32).to_le_bytes());
        out.extend_from_slice(self.pq_signing_key.as_slice());
        // PQ verifying key
        out.extend_from_slice(&(self.pq_verifying_key.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.pq_verifying_key);
        // Ed25519 keys
        out.extend_from_slice(self.ed_signing_key.as_bytes());
        out.extend_from_slice(&self.ed_verifying_key);
        // Metadata
        out.extend_from_slice(&self.created_at.to_le_bytes());
        out.extend_from_slice(&self.rotation_count.to_le_bytes());
        out
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, CryptoError> {
        let mut offset = 0;

        // PQ encapsulation key
        if data.len() < offset + 4 {
            return Err(CryptoError::InvalidInput("data too short".into()));
        }
        let pq_encap_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if data.len() < offset + pq_encap_len {
            return Err(CryptoError::InvalidInput("data too short".into()));
        }
        let pq_encap_key = data[offset..offset + pq_encap_len].to_vec();
        offset += pq_encap_len;

        // PQ decapsulation key
        if data.len() < offset + 4 {
            return Err(CryptoError::InvalidInput("data too short".into()));
        }
        let pq_decap_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if data.len() < offset + pq_decap_len {
            return Err(CryptoError::InvalidInput("data too short".into()));
        }
        let pq_decap_key = SecretBytes::new(data[offset..offset + pq_decap_len].to_vec());
        offset += pq_decap_len;

        // X25519 keys
        if data.len() < offset + 64 {
            return Err(CryptoError::InvalidInput("data too short".into()));
        }
        let x25519_secret: [u8; 32] = data[offset..offset + 32]
            .try_into()
            .map_err(|_| CryptoError::InvalidInput("invalid x25519 secret".into()))?;
        offset += 32;
        let x25519_public: [u8; 32] = data[offset..offset + 32]
            .try_into()
            .map_err(|_| CryptoError::InvalidInput("invalid x25519 public".into()))?;
        offset += 32;

        // PQ signing key
        if data.len() < offset + 4 {
            return Err(CryptoError::InvalidInput("data too short".into()));
        }
        let pq_sign_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if data.len() < offset + pq_sign_len {
            return Err(CryptoError::InvalidInput("data too short".into()));
        }
        let pq_signing_key = SecretBytes::new(data[offset..offset + pq_sign_len].to_vec());
        offset += pq_sign_len;

        // PQ verifying key
        if data.len() < offset + 4 {
            return Err(CryptoError::InvalidInput("data too short".into()));
        }
        let pq_verify_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if data.len() < offset + pq_verify_len {
            return Err(CryptoError::InvalidInput("data too short".into()));
        }
        let pq_verifying_key = data[offset..offset + pq_verify_len].to_vec();
        offset += pq_verify_len;

        // Ed25519 keys
        if data.len() < offset + 64 {
            return Err(CryptoError::InvalidInput("data too short".into()));
        }
        let ed_signing_key: [u8; 32] = data[offset..offset + 32]
            .try_into()
            .map_err(|_| CryptoError::InvalidInput("invalid ed25519 signing key".into()))?;
        offset += 32;
        let ed_verifying_key: [u8; 32] = data[offset..offset + 32]
            .try_into()
            .map_err(|_| CryptoError::InvalidInput("invalid ed25519 verifying key".into()))?;
        offset += 32;

        // Metadata (optional for backward compatibility)
        let (created_at, rotation_count) = if data.len() >= offset + 12 {
            let created_at = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
            offset += 8;
            let rotation_count = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
            (created_at, rotation_count)
        } else {
            (0, 0)
        };

        Ok(Self {
            pq_encap_key,
            pq_decap_key,
            x25519_secret: SecretKey32::new(x25519_secret),
            x25519_public,
            pq_signing_key,
            pq_verifying_key,
            ed_signing_key: SecretKey32::new(ed_signing_key),
            ed_verifying_key,
            created_at,
            rotation_count,
        })
    }
}

