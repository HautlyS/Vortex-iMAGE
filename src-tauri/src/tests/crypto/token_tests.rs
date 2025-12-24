//! Token Encryption Tests
//!
//! Tests for:
//! - Token encryption/decryption (v4 format)
//! - Token version handling
//! - Legacy token migration (v2 → v3 → v4)
//! - Token context and AAD

use crate::crypto::{
    decrypt_token, encrypt_token, encrypt_token_v4, CryptoError, TokenContext,
    TOKEN_VERSION_V3, TOKEN_VERSION_V4,
};
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce};
use rand::RngCore;
use std::collections::HashSet;

// Import the internal function for testing legacy migration
use crate::crypto::get_machine_key_with_salt;

// ============================================================================
// Token V4 Format Tests
// ============================================================================

#[test]
fn token_v4_roundtrip() {
    let plaintext = "secret_token_data_12345";
    
    let encrypted = encrypt_token(plaintext).expect("encryption");
    let (decrypted, upgraded) = decrypt_token(&encrypted).expect("decryption");
    
    assert_eq!(decrypted, plaintext);
    assert!(upgraded.is_none(), "v4 token should not need upgrade");
}

#[test]
fn token_v4_version_byte() {
    let plaintext = "test_token";
    let context = TokenContext::new();
    
    let token = encrypt_token_v4(plaintext, &context).expect("encryption");
    
    assert_eq!(token[0], TOKEN_VERSION_V4, "first byte should be version 0x04");
}

#[test]
fn token_v4_salt_uniqueness() {
    let plaintext = "same_token_data";
    let context = TokenContext::new();
    
    let mut salts = HashSet::new();
    for _ in 0..20 {
        let token = encrypt_token_v4(plaintext, &context).expect("encryption");
        // Salt is bytes 1-33 (32 bytes)
        let salt: [u8; 32] = token[1..33].try_into().unwrap();
        assert!(salts.insert(salt), "salt should be unique for each encryption");
    }
}

#[test]
fn token_v4_different_ciphertext_each_time() {
    let plaintext = "same_plaintext";
    
    let token1 = encrypt_token(plaintext).expect("encryption 1");
    let token2 = encrypt_token(plaintext).expect("encryption 2");
    
    // Tokens should be different due to random salt and nonce
    assert_ne!(token1, token2);
}

// ============================================================================
// Token Context Tests
// ============================================================================

#[test]
fn token_context_aad_contains_service_id() {
    let context = TokenContext::new();
    let aad = context.to_aad();
    
    // AAD should start with service ID
    assert!(
        aad.starts_with(b"com.vortex.image.crypto"),
        "AAD should start with service ID"
    );
}

#[test]
fn token_context_aad_contains_timestamp() {
    let context = TokenContext::new();
    let aad = context.to_aad();
    
    // AAD should be longer than just service ID (includes 8-byte timestamp)
    let service_id_len = "com.vortex.image.crypto".len();
    assert!(
        aad.len() >= service_id_len + 8,
        "AAD should contain service ID + timestamp"
    );
}

#[test]
fn token_context_timestamp_is_recent() {
    let context = TokenContext::new();
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    assert!(context.timestamp <= now, "timestamp should not be in the future");
    assert!(context.timestamp >= now - 60, "timestamp should be recent");
}

// ============================================================================
// Legacy Token Migration Tests
// ============================================================================

#[test]
fn token_v3_migration_to_v4() {
    let plaintext = "legacy_v3_token";
    
    // Create a v3 token manually
    let mut rng = rand::rngs::OsRng;
    let mut salt = [0u8; 32];
    let mut nonce = [0u8; 12];
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce);
    
    let key = get_machine_key_with_salt(&salt);
    let cipher = ChaCha20Poly1305::new(&key.into());
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
        .expect("v3 encryption");
    
    // Build v3 token: [version][salt][nonce][ciphertext]
    let mut v3_token = Vec::new();
    v3_token.push(TOKEN_VERSION_V3);
    v3_token.extend_from_slice(&salt);
    v3_token.extend_from_slice(&nonce);
    v3_token.extend_from_slice(&ciphertext);
    
    // Decrypt should work and return upgrade
    let (decrypted, upgraded) = decrypt_token(&v3_token).expect("v3 decryption");
    assert_eq!(decrypted, plaintext);
    assert!(upgraded.is_some(), "should return upgraded v4 token");
    
    // Upgraded token should be v4
    let v4_token = upgraded.unwrap();
    assert_eq!(v4_token[0], TOKEN_VERSION_V4, "upgraded token should be v4");
    
    // V4 token should decrypt correctly
    let (decrypted_v4, no_upgrade) = decrypt_token(&v4_token).expect("v4 decryption");
    assert_eq!(decrypted_v4, plaintext);
    assert!(no_upgrade.is_none(), "v4 should not need upgrade");
}

// ============================================================================
// Invalid Token Tests
// ============================================================================

#[test]
fn token_invalid_version_rejected() {
    let invalid_versions = [0x00, 0x01, 0x05, 0x10, 0xFF];
    
    for invalid_version in invalid_versions {
        let mut fake_token = vec![invalid_version];
        fake_token.extend_from_slice(&[0u8; 100]); // Padding
        
        let result = decrypt_token(&fake_token);
        assert!(result.is_err(), "version 0x{:02X} should be rejected", invalid_version);
        
        // Verify it's the right error type
        if let Err(CryptoError::UnsupportedTokenVersion(v)) = result {
            assert_eq!(v, invalid_version);
        } else {
            panic!("expected UnsupportedTokenVersion error for version 0x{:02X}", invalid_version);
        }
    }
}

#[test]
fn token_empty_rejected() {
    let result = decrypt_token(&[]);
    assert!(result.is_err(), "empty token should be rejected");
}

#[test]
fn token_too_short_rejected() {
    // V4 minimum: version(1) + salt(32) + nonce(12) + aad_len(2) + tag(16) = 63 bytes
    let short_token = vec![TOKEN_VERSION_V4; 30];
    
    let result = decrypt_token(&short_token);
    assert!(result.is_err(), "too short token should be rejected");
}

#[test]
fn token_corrupted_ciphertext_rejected() {
    let plaintext = "test_token";
    let encrypted = encrypt_token(plaintext).expect("encryption");
    
    // Corrupt the ciphertext (last bytes)
    let mut corrupted = encrypted.clone();
    let len = corrupted.len();
    corrupted[len - 1] = corrupted[len - 1].wrapping_add(1);
    
    let result = decrypt_token(&corrupted);
    assert!(result.is_err(), "corrupted token should be rejected");
}

#[test]
fn token_corrupted_salt_rejected() {
    let plaintext = "test_token";
    let encrypted = encrypt_token(plaintext).expect("encryption");
    
    // Corrupt the salt (bytes 1-33)
    let mut corrupted = encrypted.clone();
    corrupted[5] = corrupted[5].wrapping_add(1);
    
    let result = decrypt_token(&corrupted);
    assert!(result.is_err(), "token with corrupted salt should be rejected");
}

#[test]
fn token_corrupted_nonce_rejected() {
    let plaintext = "test_token";
    let encrypted = encrypt_token(plaintext).expect("encryption");
    
    // Corrupt the nonce (bytes 33-45)
    let mut corrupted = encrypted.clone();
    corrupted[35] = corrupted[35].wrapping_add(1);
    
    let result = decrypt_token(&corrupted);
    assert!(result.is_err(), "token with corrupted nonce should be rejected");
}

// ============================================================================
// Token Content Tests
// ============================================================================

#[test]
fn token_preserves_unicode() {
    let plaintext = "Hello 世界 🔐 émojis";
    
    let encrypted = encrypt_token(plaintext).expect("encryption");
    let (decrypted, _) = decrypt_token(&encrypted).expect("decryption");
    
    assert_eq!(decrypted, plaintext);
}

#[test]
fn token_preserves_special_characters() {
    let plaintext = "token with\nnewlines\tand\ttabs\rand\rcarriage returns";
    
    let encrypted = encrypt_token(plaintext).expect("encryption");
    let (decrypted, _) = decrypt_token(&encrypted).expect("decryption");
    
    assert_eq!(decrypted, plaintext);
}

#[test]
fn token_handles_long_content() {
    // 10KB of data
    let plaintext: String = (0..10000).map(|i| ((i % 26) as u8 + b'a') as char).collect();
    
    let encrypted = encrypt_token(&plaintext).expect("encryption");
    let (decrypted, _) = decrypt_token(&encrypted).expect("decryption");
    
    assert_eq!(decrypted, plaintext);
}
