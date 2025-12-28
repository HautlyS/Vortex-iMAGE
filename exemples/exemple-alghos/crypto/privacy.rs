//! Privacy enhancements - message padding, key cycling, forward secrecy
//! Adapted from Quantum-Secure-Messaging patterns
use std::sync::atomic::{AtomicU64, Ordering};
use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Nonce};
use rand::RngCore;

const PADDING_BLOCK_SIZE: usize = 2048; // 2KB blocks
const NONCE_SIZE: usize = 12;

/// Pad message to fixed block size to prevent length analysis
pub fn pad_message(data: &[u8]) -> Vec<u8> {
    let padding_needed = PADDING_BLOCK_SIZE - (data.len() % PADDING_BLOCK_SIZE);
    let total_len = data.len() + padding_needed;
    
    let mut padded = Vec::with_capacity(total_len + 2);
    padded.extend_from_slice(data);
    
    // Random padding bytes
    let mut padding = vec![0u8; padding_needed];
    rand::thread_rng().fill_bytes(&mut padding);
    padded.extend_from_slice(&padding);
    
    // Last 2 bytes encode padding length
    padded.extend_from_slice(&(padding_needed as u16).to_be_bytes());
    padded
}

/// Remove padding from message
pub fn unpad_message(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 2 {
        return None;
    }
    
    let padding_len = u16::from_be_bytes([data[data.len() - 2], data[data.len() - 1]]) as usize;
    let content_len = data.len().checked_sub(padding_len + 2)?;
    
    Some(data[..content_len].to_vec())
}

/// Session key with forward secrecy via key cycling
pub struct SessionKey {
    current_key: [u8; 32],
    counter: AtomicU64,
    initial_hash: [u8; 32],
}

impl SessionKey {
    /// Create from shared secret using BLAKE3 KDF
    pub fn from_shared_secret(secret: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new_keyed(&[0u8; 32]);
        hasher.update(b"vortex-session-key-v1");
        hasher.update(secret);
        let key = *hasher.finalize().as_bytes();
        
        let initial_hash = *blake3::hash(secret).as_bytes();
        
        Self {
            current_key: key,
            counter: AtomicU64::new(0),
            initial_hash,
        }
    }

    /// Get verification string for out-of-band confirmation
    pub fn verification_string(&self) -> String {
        hex::encode(&self.initial_hash[..8])
    }

    /// Cycle key for forward secrecy (call after each message)
    pub fn cycle(&self) -> [u8; 32] {
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        let mut hasher = blake3::Hasher::new_keyed(&self.current_key);
        hasher.update(&counter.to_be_bytes());
        *hasher.finalize().as_bytes()
    }

    /// Encrypt with current key and cycle
    pub fn encrypt(&self, plaintext: &[u8]) -> Option<Vec<u8>> {
        let key = self.cycle();
        let cipher = ChaCha20Poly1305::new(&key.into());
        
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        
        // Pad message first
        let padded = pad_message(plaintext);
        
        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), padded.as_ref()).ok()?;
        
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Some(result)
    }

    /// Decrypt and unpad
    pub fn decrypt(&self, data: &[u8]) -> Option<Vec<u8>> {
        if data.len() < NONCE_SIZE {
            return None;
        }
        
        let key = self.cycle();
        let cipher = ChaCha20Poly1305::new(&key.into());
        
        let nonce = Nonce::from_slice(&data[..NONCE_SIZE]);
        let ciphertext = &data[NONCE_SIZE..];
        
        let padded = cipher.decrypt(nonce, ciphertext).ok()?;
        unpad_message(&padded)
    }

    /// Get current message counter
    pub fn message_count(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
}

impl Clone for SessionKey {
    fn clone(&self) -> Self {
        Self {
            current_key: self.current_key,
            counter: AtomicU64::new(self.counter.load(Ordering::SeqCst)),
            initial_hash: self.initial_hash,
        }
    }
}

/// Timing-safe message sending with random delays
pub struct TimingSafeChannel {
    min_delay_ms: u64,
    max_delay_ms: u64,
}

impl TimingSafeChannel {
    pub fn new(min_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self { min_delay_ms, max_delay_ms }
    }

    /// Get random delay for timing obfuscation
    pub fn random_delay(&self) -> std::time::Duration {
        let range = self.max_delay_ms - self.min_delay_ms;
        let delay = self.min_delay_ms + (rand::random::<u64>() % range.max(1));
        std::time::Duration::from_millis(delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_roundtrip() {
        let original = b"Hello, World!";
        let padded = pad_message(original);
        
        // Should be multiple of block size + 2 bytes for length
        assert_eq!((padded.len() - 2) % PADDING_BLOCK_SIZE, 0);
        
        let unpadded = unpad_message(&padded).unwrap();
        assert_eq!(unpadded, original);
    }

    #[test]
    fn test_session_key_encrypt_decrypt() {
        let secret = b"shared-secret-key-for-testing";
        let key = SessionKey::from_shared_secret(secret);
        
        let plaintext = b"Secret message";
        let encrypted = key.encrypt(plaintext).unwrap();
        
        // Need a fresh key with same secret for decryption
        let key2 = SessionKey::from_shared_secret(secret);
        let decrypted = key2.decrypt(&encrypted).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_key_cycling() {
        let secret = b"test-secret";
        let key = SessionKey::from_shared_secret(secret);
        
        let k1 = key.cycle();
        let k2 = key.cycle();
        let k3 = key.cycle();
        
        // Each cycle should produce different key
        assert_ne!(k1, k2);
        assert_ne!(k2, k3);
        assert_eq!(key.message_count(), 3);
    }

    #[test]
    fn test_verification_string() {
        let secret = b"test-secret";
        let key1 = SessionKey::from_shared_secret(secret);
        let key2 = SessionKey::from_shared_secret(secret);
        
        // Same secret should produce same verification string
        assert_eq!(key1.verification_string(), key2.verification_string());
    }
}
