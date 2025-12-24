//! Property-Based Tests for Crypto Module
//!
//! Uses proptest to verify cryptographic properties hold for arbitrary inputs.
//! These tests are more thorough than unit tests but slower to run.
//!
//! Run with: `cargo test --release prop_` (release mode recommended for speed)

use crate::crypto::{
    decrypt, decrypt_with_aad, decrypt_with_password, encrypt, encrypt_with_aad,
    encrypt_with_password, encrypt_token_v4, HybridKeypair, KeypairStore, SecretBytes,
    TokenContext, TOKEN_VERSION_V4,
};
use proptest::prelude::*;

// ============================================================================
// Signing Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// Property: Sign-verify roundtrip succeeds for any data
    #[test]
    fn prop_signing_roundtrip(data in prop::collection::vec(any::<u8>(), 0..256)) {
        let keypair = HybridKeypair::generate().expect("keypair generation");
        let signature = keypair.sign(&data).expect("signing");
        let bundle = keypair.public_bundle();
        bundle.verify(&data, &signature).expect("verification should succeed");
    }

    /// Property: Tampered data fails verification
    #[test]
    fn prop_signature_tamper_detection(data in prop::collection::vec(any::<u8>(), 1..256)) {
        let keypair = HybridKeypair::generate().expect("keypair generation");
        let signature = keypair.sign(&data).expect("signing");
        let bundle = keypair.public_bundle();

        // Original data verifies
        bundle.verify(&data, &signature).expect("verification");

        // Tampered data should fail
        let mut tampered = data.clone();
        tampered[0] = tampered[0].wrapping_add(1);
        let result = bundle.verify(&tampered, &signature);
        prop_assert!(result.is_err(), "tampered data should fail verification");
    }
}

// ============================================================================
// Encryption Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// Property: Hybrid encryption roundtrip succeeds for any data
    #[test]
    fn prop_hybrid_encryption_roundtrip(data in prop::collection::vec(any::<u8>(), 0..1024)) {
        let keypair = HybridKeypair::generate().expect("keypair generation");
        let bundle = keypair.public_bundle();

        let encrypted = encrypt(&data, &bundle).expect("encryption");
        let decrypted = decrypt(&encrypted, &keypair).expect("decryption");

        prop_assert_eq!(decrypted, data);
    }

    /// Property: AAD binding is enforced correctly
    #[test]
    fn prop_aad_binding_correctness(
        data in prop::collection::vec(any::<u8>(), 1..128),
        aad in prop::collection::vec(any::<u8>(), 1..32)
    ) {
        let keypair = HybridKeypair::generate().expect("keypair generation");
        let bundle = keypair.public_bundle();

        // Encrypt with AAD
        let encrypted = encrypt_with_aad(&data, &bundle, Some(&aad)).expect("encryption");

        // Correct AAD should decrypt
        let decrypted = decrypt_with_aad(&encrypted, &keypair, Some(&aad)).expect("decryption");
        prop_assert_eq!(decrypted, data);

        // Wrong AAD should fail
        let wrong_aad = vec![0u8; aad.len()];
        if wrong_aad != aad {
            let result = decrypt_with_aad(&encrypted, &keypair, Some(&wrong_aad));
            prop_assert!(result.is_err(), "wrong AAD should fail decryption");
        }

        // Missing AAD should fail (when AAD was used)
        let result = decrypt_with_aad(&encrypted, &keypair, None);
        prop_assert!(result.is_err(), "missing AAD should fail decryption");
    }
}

// ============================================================================
// Password Encryption Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))] // Slower due to Argon2

    /// Property: Password encryption roundtrip succeeds
    #[test]
    fn prop_password_encryption_roundtrip(
        data in prop::collection::vec(any::<u8>(), 1..256),
        password in "[a-zA-Z0-9]{8,16}"
    ) {
        let encrypted = encrypt_with_password(&data, password.as_bytes()).expect("encryption");
        let decrypted = decrypt_with_password(&encrypted, password.as_bytes()).expect("decryption");

        prop_assert_eq!(decrypted, data);

        // Wrong password should fail
        let wrong_password = format!("{}X", password);
        let result = decrypt_with_password(&encrypted, wrong_password.as_bytes());
        prop_assert!(result.is_err(), "wrong password should fail");
    }
}

// ============================================================================
// Key Rotation Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    /// Property: Data encrypted before rotation can still be decrypted
    #[test]
    fn prop_key_rotation_decryption_compatibility(data in prop::collection::vec(any::<u8>(), 1..256)) {
        let mut store = KeypairStore::new();

        // Generate initial keypair
        let keypair = HybridKeypair::generate().expect("keypair generation");
        let old_bundle = keypair.public_bundle();
        let handle = store.insert(keypair);

        // Encrypt with old key
        let encrypted = encrypt(&data, &old_bundle).expect("encryption");

        // Rotate the key
        let _new_bundle = store.rotate(handle).expect("rotation");

        // Get all keypairs for decryption (should include old)
        let all_keypairs = store.get_all_for_decryption(handle);
        prop_assert!(all_keypairs.len() >= 2, "should have current + rotated keypairs");

        // Try decryption with all available keys
        let mut decrypted = None;
        for kp_arc in all_keypairs {
            let kp = kp_arc.lock().unwrap();
            if let Ok(plain) = decrypt(&encrypted, &kp) {
                decrypted = Some(plain);
                break;
            }
        }

        prop_assert!(decrypted.is_some(), "should decrypt with one of the keys");
        prop_assert_eq!(decrypted.unwrap(), data);
    }
}

// ============================================================================
// Token Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Property: Token version byte is always correct
    #[test]
    fn prop_token_version_byte(plaintext in "[a-zA-Z0-9]{1,50}") {
        let context = TokenContext::new();
        let token = encrypt_token_v4(&plaintext, &context).expect("encryption");
        prop_assert_eq!(token[0], TOKEN_VERSION_V4, "first byte should be version 0x04");
    }
}

// ============================================================================
// Secret Types Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Property: SecretBytes clone_secret produces equal bytes
    #[test]
    fn prop_secret_bytes_clone_produces_equal_bytes(data in prop::collection::vec(any::<u8>(), 1..128)) {
        let secret = SecretBytes::new(data.clone());
        let cloned = secret.clone_secret();
        prop_assert_eq!(secret.as_slice(), cloned.as_slice());
        prop_assert_eq!(secret.len(), cloned.len());
    }
}

// ============================================================================
// Stress Tests (Higher Case Counts)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Stress test: Many small encryptions
    #[test]
    fn prop_stress_small_encryptions(data in prop::collection::vec(any::<u8>(), 1..64)) {
        let keypair = HybridKeypair::generate().expect("keypair generation");
        let bundle = keypair.public_bundle();

        let encrypted = encrypt(&data, &bundle).expect("encryption");
        let decrypted = decrypt(&encrypted, &keypair).expect("decryption");

        prop_assert_eq!(decrypted, data);
    }
}
