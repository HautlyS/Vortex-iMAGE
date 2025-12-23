//! Rust Module - 1 functions, 0 structs
//! Core functionality: Backend operations and data processing
//! External crates: 2 dependencies

#[cfg(test)]
mod tests {
    use crate::crypto::{HybridKeypair, EncryptionSettings, encrypt_file_data, decrypt_file_data, EncryptionMethod};
    use crate::compress::{compress_file_data, decompress_file_data, ItemCompressionSettings, Algorithm};
    #[test]
    fn verify_security_pipeline_roundtrip() {
        
        let keypair = HybridKeypair::generate().expect("Failed to generate keypair");
        let public_bundle = keypair.public_bundle();

        let original_data = vec![0u8; 1024 * 1024]; 
        
        let filename = "test_image.jpg";

        let compression_settings = ItemCompressionSettings {
            enabled: true,
            algorithm: Algorithm::Zstd,
            level: 3,
            prefer_speed: false,
            min_size_threshold: 0, 
            skip_already_compressed: false,
        };

        let compressed_data = compress_file_data(&original_data, filename, &compression_settings)
            .expect("Compression failed");
        
        println!("Original size: {}", original_data.len());
        println!("Compressed size: {}", compressed_data.data.len());
        
        assert!(compressed_data.data.len() < original_data.len(), "Compression should reduce size for this data");
        assert!(compressed_data.compressed, "Data should be marked as compressed");

        let compressed_bytes = serde_json::to_vec(&compressed_data).expect("Serialization failed");

        let encryption_settings = EncryptionSettings {
            enabled: true,
            use_password: false,
            use_keypair: true,
            recipient_bundle: Some(public_bundle),
        };

        let encrypted_file = encrypt_file_data(&compressed_bytes, &encryption_settings, None, None)
            .expect("Encryption failed");

        let encrypted_payload = serde_json::to_vec(&encrypted_file).expect("Final serialization failed");

        println!("Encrypted payload size: {}", encrypted_payload.len());

        assert_ne!(encrypted_payload, original_data, "Encrypted data should not match original");
        assert!(encrypted_file.encrypted, "File should be marked as encrypted");
        match encrypted_file.method {
            EncryptionMethod::HybridPQ => (),
            _ => panic!("Wrong encryption method used"),
        }

        let downloaded_bytes = encrypted_payload;

        let received_encrypted_file: crate::crypto::EncryptedFileData = serde_json::from_slice(&downloaded_bytes)
            .expect("Failed to deserialize encrypted file");
        
        let decrypted_bytes = decrypt_file_data(
            &received_encrypted_file, 
            None, 
            Some(&keypair.to_bytes())
        ).expect("Decryption failed");

        let received_compressed_file: crate::compress::CompressedFileData = serde_json::from_slice(&decrypted_bytes)
            .expect("Failed to deserialize compressed file");
            
        let final_data = decompress_file_data(&received_compressed_file)
            .expect("Decompression failed");

        assert_eq!(final_data, original_data, "Roundtrip failed: Data mismatch");
        println!("Verification successful!");
    }
}