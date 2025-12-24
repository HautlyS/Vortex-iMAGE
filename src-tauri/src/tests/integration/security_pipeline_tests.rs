//! Security Pipeline Integration Tests
//!
//! End-to-end tests verifying the complete security pipeline:
//! - Compress → Encrypt → Decrypt → Decompress roundtrip
//! - Data integrity across the full pipeline
//! - Various data sizes and types

use crate::compress::{
    compress_file_data, decompress_file_data, Algorithm, ItemCompressionSettings,
};
use crate::crypto::{
    decrypt, decrypt_with_aad, decrypt_with_keypair_bytes, encrypt, encrypt_with_aad,
    EncryptedFileData, EncryptedPayload, EncryptionMethod, HybridKeypair, KeypairStore,
};

// ============================================================================
// Full Pipeline Roundtrip Tests
// ============================================================================

#[test]
fn security_pipeline_full_roundtrip() {
    // Generate keypair for encryption
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let public_bundle = keypair.public_bundle();

    // Test data: 1MB of zeros (compresses well)
    let original_data = vec![0u8; 1024 * 1024];
    let filename = "test_image.raw";

    // Step 1: Compress
    let compression_settings = ItemCompressionSettings {
        enabled: true,
        algorithm: Algorithm::Zstd,
        level: 3,
        prefer_speed: false,
        min_size_threshold: 0,
        skip_already_compressed: false,
    };

    let compressed = compress_file_data(&original_data, filename, &compression_settings)
        .expect("compression");

    assert!(compressed.compressed, "data should be compressed");
    assert!(
        compressed.data.len() < original_data.len(),
        "compressed should be smaller"
    );

    // Step 2: Serialize and encrypt
    let compressed_bytes = serde_json::to_vec(&compressed).expect("serialization");
    let encrypted_payload = encrypt(&compressed_bytes, &public_bundle).expect("encryption");

    // Wrap in EncryptedFileData
    let encrypted_file = EncryptedFileData {
        data: serde_json::to_vec(&encrypted_payload).expect("payload serialization"),
        encrypted: true,
        method: EncryptionMethod::HybridPQ,
        metadata: None,
    };

    // Step 3: Simulate storage/transmission
    let stored_bytes = serde_json::to_vec(&encrypted_file).expect("final serialization");

    // Verify encryption changed the data
    assert_ne!(stored_bytes, original_data);

    // Step 4: Decrypt
    let received_file: EncryptedFileData =
        serde_json::from_slice(&stored_bytes).expect("deserialize encrypted file");
    let received_payload: EncryptedPayload =
        serde_json::from_slice(&received_file.data).expect("deserialize payload");

    let decrypted_bytes =
        decrypt_with_keypair_bytes(&received_payload, &keypair.to_bytes()).expect("decryption");

    // Step 5: Decompress
    let received_compressed: crate::compress::CompressedFileData =
        serde_json::from_slice(&decrypted_bytes).expect("deserialize compressed");
    let final_data = decompress_file_data(&received_compressed).expect("decompression");

    // Verify roundtrip integrity
    assert_eq!(final_data, original_data, "roundtrip failed: data mismatch");
}

#[test]
fn security_pipeline_with_aad() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let public_bundle = keypair.public_bundle();

    let original_data = vec![42u8; 10000];
    let filename = "secret.dat";
    let aad = b"recipient:user@example.com;timestamp:1234567890";

    // Compress
    let settings = ItemCompressionSettings::default();
    let compressed = compress_file_data(&original_data, filename, &settings).expect("compression");
    let compressed_bytes = serde_json::to_vec(&compressed).expect("serialization");

    // Encrypt with AAD
    let encrypted = encrypt_with_aad(&compressed_bytes, &public_bundle, Some(aad))
        .expect("encryption with AAD");

    assert!(encrypted.aad_hash.is_some(), "AAD hash should be present");

    // Decrypt with correct AAD
    let decrypted_bytes =
        decrypt_with_aad(&encrypted, &keypair, Some(aad)).expect("decryption with AAD");

    // Decompress
    let received_compressed: crate::compress::CompressedFileData =
        serde_json::from_slice(&decrypted_bytes).expect("deserialize");
    let final_data = decompress_file_data(&received_compressed).expect("decompression");

    assert_eq!(final_data, original_data);

    // Verify wrong AAD fails
    let wrong_aad = b"wrong context";
    let result = decrypt_with_aad(&encrypted, &keypair, Some(wrong_aad));
    assert!(result.is_err(), "wrong AAD should fail");
}

#[test]
fn security_pipeline_with_key_rotation() {
    let mut store = KeypairStore::new();

    let keypair = HybridKeypair::generate().expect("keypair generation");
    let old_bundle = keypair.public_bundle();
    let handle = store.insert(keypair);

    // Encrypt data with old key
    let original_data = vec![42u8; 5000];
    let filename = "rotated.dat";

    let settings = ItemCompressionSettings::default();
    let compressed = compress_file_data(&original_data, filename, &settings).expect("compression");
    let compressed_bytes = serde_json::to_vec(&compressed).expect("serialization");

    let encrypted = encrypt(&compressed_bytes, &old_bundle).expect("encryption");

    // Rotate the key
    let new_bundle = store.rotate(handle).expect("rotation");
    assert_ne!(old_bundle.key_id, new_bundle.key_id);

    // Should still be able to decrypt with rotated keys
    let all_keypairs = store.get_all_for_decryption(handle);
    assert!(all_keypairs.len() >= 2);

    let mut decrypted_bytes = None;
    for kp_arc in all_keypairs {
        let kp = kp_arc.lock().unwrap();
        if let Ok(plain) = decrypt(&encrypted, &kp) {
            decrypted_bytes = Some(plain);
            break;
        }
    }

    let decrypted_bytes = decrypted_bytes.expect("should decrypt with one of the keys");

    // Decompress
    let received_compressed: crate::compress::CompressedFileData =
        serde_json::from_slice(&decrypted_bytes).expect("deserialize");
    let final_data = decompress_file_data(&received_compressed).expect("decompression");

    assert_eq!(final_data, original_data);
}

// ============================================================================
// Various Data Types Tests
// ============================================================================

#[test]
fn security_pipeline_binary_data() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();

    // All byte values
    let original_data: Vec<u8> = (0..=255).cycle().take(10000).collect();
    let filename = "binary.bin";

    let settings = ItemCompressionSettings::default();
    let compressed = compress_file_data(&original_data, filename, &settings).expect("compression");
    let compressed_bytes = serde_json::to_vec(&compressed).expect("serialization");

    let encrypted = encrypt(&compressed_bytes, &bundle).expect("encryption");
    let decrypted_bytes = decrypt(&encrypted, &keypair).expect("decryption");

    let received_compressed: crate::compress::CompressedFileData =
        serde_json::from_slice(&decrypted_bytes).expect("deserialize");
    let final_data = decompress_file_data(&received_compressed).expect("decompression");

    assert_eq!(final_data, original_data);
}

#[test]
fn security_pipeline_small_data() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();

    // Small data (below compression threshold)
    let original_data = vec![42u8; 100];
    let filename = "small.txt";

    let settings = ItemCompressionSettings::default();
    let compressed = compress_file_data(&original_data, filename, &settings).expect("compression");

    // Small data should not be compressed
    assert!(!compressed.compressed);

    let compressed_bytes = serde_json::to_vec(&compressed).expect("serialization");
    let encrypted = encrypt(&compressed_bytes, &bundle).expect("encryption");
    let decrypted_bytes = decrypt(&encrypted, &keypair).expect("decryption");

    let received_compressed: crate::compress::CompressedFileData =
        serde_json::from_slice(&decrypted_bytes).expect("deserialize");
    let final_data = decompress_file_data(&received_compressed).expect("decompression");

    assert_eq!(final_data, original_data);
}

#[test]
fn security_pipeline_empty_data() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();

    let original_data = vec![];
    let filename = "empty.txt";

    let settings = ItemCompressionSettings {
        min_size_threshold: 0,
        ..Default::default()
    };
    let compressed = compress_file_data(&original_data, filename, &settings).expect("compression");
    let compressed_bytes = serde_json::to_vec(&compressed).expect("serialization");

    let encrypted = encrypt(&compressed_bytes, &bundle).expect("encryption");
    let decrypted_bytes = decrypt(&encrypted, &keypair).expect("decryption");

    let received_compressed: crate::compress::CompressedFileData =
        serde_json::from_slice(&decrypted_bytes).expect("deserialize");
    let final_data = decompress_file_data(&received_compressed).expect("decompression");

    assert!(final_data.is_empty());
}

// ============================================================================
// Algorithm Variation Tests
// ============================================================================

#[test]
fn security_pipeline_all_compression_algorithms() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();

    let original_data = vec![42u8; 10000];
    let filename = "test.dat";

    let algorithms = [
        Algorithm::Zstd,
        Algorithm::Lz4,
        Algorithm::Snap,
        Algorithm::Brotli,
        Algorithm::Gzip,
        Algorithm::None,
    ];

    for algo in algorithms {
        let settings = ItemCompressionSettings {
            algorithm: algo,
            min_size_threshold: 0,
            skip_already_compressed: false,
            ..Default::default()
        };

        let compressed =
            compress_file_data(&original_data, filename, &settings).expect("compression");
        let compressed_bytes = serde_json::to_vec(&compressed).expect("serialization");

        let encrypted = encrypt(&compressed_bytes, &bundle).expect("encryption");
        let decrypted_bytes = decrypt(&encrypted, &keypair).expect("decryption");

        let received_compressed: crate::compress::CompressedFileData =
            serde_json::from_slice(&decrypted_bytes).expect("deserialize");
        let final_data = decompress_file_data(&received_compressed).expect("decompression");

        assert_eq!(
            final_data, original_data,
            "roundtrip failed for {:?}",
            algo
        );
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn security_pipeline_corrupted_encrypted_data_fails() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();

    let original_data = vec![42u8; 10000];
    let filename = "test.dat";

    let settings = ItemCompressionSettings::default();
    let compressed = compress_file_data(&original_data, filename, &settings).expect("compression");
    let compressed_bytes = serde_json::to_vec(&compressed).expect("serialization");

    let mut encrypted = encrypt(&compressed_bytes, &bundle).expect("encryption");

    // Corrupt the ciphertext
    let mid = encrypted.ciphertext.len() / 2;
    encrypted.ciphertext[mid] = encrypted.ciphertext[mid].wrapping_add(1);

    let result = decrypt(&encrypted, &keypair);
    assert!(result.is_err(), "corrupted ciphertext should fail decryption");
}

#[test]
fn security_pipeline_corrupted_checksum_fails() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();

    let original_data = vec![42u8; 10000];
    let filename = "test.dat";

    let settings = ItemCompressionSettings::default();
    let compressed = compress_file_data(&original_data, filename, &settings).expect("compression");
    let compressed_bytes = serde_json::to_vec(&compressed).expect("serialization");

    let encrypted = encrypt(&compressed_bytes, &bundle).expect("encryption");
    let decrypted_bytes = decrypt(&encrypted, &keypair).expect("decryption");

    let mut received_compressed: crate::compress::CompressedFileData =
        serde_json::from_slice(&decrypted_bytes).expect("deserialize");

    // Corrupt the checksum
    received_compressed.checksum[0] = received_compressed.checksum[0].wrapping_add(1);

    let result = decompress_file_data(&received_compressed);
    assert!(result.is_err(), "corrupted checksum should fail decompression");
}
