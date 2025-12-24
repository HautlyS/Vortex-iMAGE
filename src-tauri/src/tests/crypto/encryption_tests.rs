//! Encryption Tests
//!
//! Tests for:
//! - Hybrid PQ + classical encryption roundtrip
//! - Associated Authenticated Data (AAD) binding
//! - Password-based encryption
//! - Edge cases (empty data, large data)

use crate::crypto::{
    decrypt, decrypt_with_aad, decrypt_with_password, encrypt, encrypt_with_aad,
    encrypt_with_password, HybridKeypair, KeypairStore,
};

// ============================================================================
// Hybrid Encryption Roundtrip Tests
// ============================================================================

#[test]
fn hybrid_encryption_roundtrip_basic() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"Hello, hybrid encryption!";
    let encrypted = encrypt(data, &bundle).expect("encryption");
    let decrypted = decrypt(&encrypted, &keypair).expect("decryption");
    
    assert_eq!(decrypted, data);
}

#[test]
fn hybrid_encryption_roundtrip_binary_data() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    // Binary data with all byte values
    let data: Vec<u8> = (0..=255).collect();
    let encrypted = encrypt(&data, &bundle).expect("encryption");
    let decrypted = decrypt(&encrypted, &keypair).expect("decryption");
    
    assert_eq!(decrypted, data);
}

#[test]
fn hybrid_encryption_empty_data() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let encrypted = encrypt(&[], &bundle).expect("encryption");
    let decrypted = decrypt(&encrypted, &keypair).expect("decryption");
    
    assert!(decrypted.is_empty());
}

#[test]
fn hybrid_encryption_large_data() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    // 1MB of data
    let data = vec![0xABu8; 1024 * 1024];
    let encrypted = encrypt(&data, &bundle).expect("encryption");
    let decrypted = decrypt(&encrypted, &keypair).expect("decryption");
    
    assert_eq!(decrypted, data);
}

#[test]
fn hybrid_encryption_produces_different_ciphertext_each_time() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"same plaintext";
    let encrypted1 = encrypt(data, &bundle).expect("encryption 1");
    let encrypted2 = encrypt(data, &bundle).expect("encryption 2");
    
    // Ciphertexts should be different due to random nonce and ephemeral keys
    assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    assert_ne!(encrypted1.nonce, encrypted2.nonce);
}

#[test]
fn hybrid_encryption_wrong_keypair_fails() {
    let keypair1 = HybridKeypair::generate().expect("keypair generation 1");
    let keypair2 = HybridKeypair::generate().expect("keypair generation 2");
    let bundle1 = keypair1.public_bundle();
    
    let data = b"secret message";
    let encrypted = encrypt(data, &bundle1).expect("encryption");
    
    // Decryption with wrong keypair should fail
    let result = decrypt(&encrypted, &keypair2);
    assert!(result.is_err(), "decryption with wrong keypair should fail");
}

// ============================================================================
// AAD (Associated Authenticated Data) Tests
// ============================================================================

#[test]
fn aad_encryption_roundtrip() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"sensitive data with context binding";
    let aad = b"recipient:user@example.com;timestamp:1234567890";
    
    let encrypted = encrypt_with_aad(data, &bundle, Some(aad)).expect("encryption");
    
    // AAD hash should be present
    assert!(encrypted.aad_hash.is_some(), "AAD hash should be present");
    
    // Decrypt with correct AAD
    let decrypted = decrypt_with_aad(&encrypted, &keypair, Some(aad)).expect("decryption");
    assert_eq!(decrypted, data);
}

#[test]
fn aad_wrong_aad_fails_decryption() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"sensitive data";
    let aad = b"correct context";
    let wrong_aad = b"wrong context";
    
    let encrypted = encrypt_with_aad(data, &bundle, Some(aad)).expect("encryption");
    
    let result = decrypt_with_aad(&encrypted, &keypair, Some(wrong_aad));
    assert!(result.is_err(), "decryption with wrong AAD should fail");
}

#[test]
fn aad_missing_aad_fails_decryption() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"sensitive data";
    let aad = b"required context";
    
    let encrypted = encrypt_with_aad(data, &bundle, Some(aad)).expect("encryption");
    
    // Decrypt without AAD should fail
    let result = decrypt_with_aad(&encrypted, &keypair, None);
    assert!(result.is_err(), "decryption without AAD should fail when AAD was used");
}

#[test]
fn aad_none_encryption_works_without_aad() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"data without context";
    
    // Encrypt without AAD
    let encrypted = encrypt_with_aad(data, &bundle, None).expect("encryption");
    assert!(encrypted.aad_hash.is_none(), "AAD hash should not be present");
    
    // Decrypt without AAD
    let decrypted = decrypt_with_aad(&encrypted, &keypair, None).expect("decryption");
    assert_eq!(decrypted, data);
}

// ============================================================================
// Key Rotation + Encryption Tests
// ============================================================================

#[test]
fn encryption_with_rotated_key_decrypts_with_old_key() {
    let mut store = KeypairStore::new();
    
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let old_bundle = keypair.public_bundle();
    let handle = store.insert(keypair);
    
    // Encrypt with old key
    let data = b"data encrypted before rotation";
    let encrypted = encrypt(data, &old_bundle).expect("encryption");
    
    // Rotate the key
    let _new_bundle = store.rotate(handle).expect("rotation");
    
    // Get all keypairs for decryption
    let all_keypairs = store.get_all_for_decryption(handle);
    assert!(all_keypairs.len() >= 2, "should have current + rotated keypairs");
    
    // Should be able to decrypt with one of the keys
    let mut decrypted = None;
    for kp_arc in all_keypairs {
        let kp = kp_arc.lock().unwrap();
        if let Ok(plain) = decrypt(&encrypted, &kp) {
            decrypted = Some(plain);
            break;
        }
    }
    
    assert!(decrypted.is_some(), "should decrypt with one of the keys");
    assert_eq!(decrypted.unwrap(), data);
}

#[test]
fn encryption_with_new_key_after_rotation() {
    let mut store = KeypairStore::new();
    
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let handle = store.insert(keypair);
    
    // Rotate the key
    let new_bundle = store.rotate(handle).expect("rotation");
    
    // Encrypt with new key
    let data = b"data encrypted after rotation";
    let encrypted = encrypt(data, &new_bundle).expect("encryption");
    
    // Current keypair should decrypt
    let current = store.get(handle).unwrap();
    let kp = current.lock().unwrap();
    let decrypted = decrypt(&encrypted, &kp).expect("decryption");
    
    assert_eq!(decrypted, data);
}

// ============================================================================
// Password-Based Encryption Tests
// ============================================================================

#[test]
fn password_encryption_roundtrip() {
    let data = b"secret data protected by password";
    let password = b"correct-horse-battery-staple";
    
    let encrypted = encrypt_with_password(data, password).expect("encryption");
    let decrypted = decrypt_with_password(&encrypted, password).expect("decryption");
    
    assert_eq!(decrypted, data);
}

#[test]
fn password_encryption_wrong_password_fails() {
    let data = b"secret data";
    let password = b"correct-password";
    let wrong_password = b"wrong-password";
    
    let encrypted = encrypt_with_password(data, password).expect("encryption");
    
    let result = decrypt_with_password(&encrypted, wrong_password);
    assert!(result.is_err(), "decryption with wrong password should fail");
}

#[test]
fn password_encryption_empty_password() {
    let data = b"data with empty password";
    let password = b"";
    
    let encrypted = encrypt_with_password(data, password).expect("encryption");
    let decrypted = decrypt_with_password(&encrypted, password).expect("decryption");
    
    assert_eq!(decrypted, data);
}

#[test]
fn password_encryption_produces_different_ciphertext() {
    let data = b"same plaintext";
    let password = b"same-password";
    
    let encrypted1 = encrypt_with_password(data, password).expect("encryption 1");
    let encrypted2 = encrypt_with_password(data, password).expect("encryption 2");
    
    // Different salt and nonce should produce different ciphertext
    assert_ne!(encrypted1, encrypted2);
}

#[test]
fn password_encryption_rejects_truncated_data() {
    let data = b"secret data";
    let password = b"password";
    
    let encrypted = encrypt_with_password(data, password).expect("encryption");
    
    // Truncate the encrypted data
    let truncated = &encrypted[..encrypted.len() / 2];
    
    let result = decrypt_with_password(truncated, password);
    assert!(result.is_err(), "truncated data should fail decryption");
}

#[test]
fn password_encryption_rejects_too_short_data() {
    let password = b"password";
    
    // Data shorter than minimum (salt + nonce + tag = 28 bytes)
    let short_data = vec![0u8; 10];
    
    let result = decrypt_with_password(&short_data, password);
    assert!(result.is_err(), "too short data should fail");
}
