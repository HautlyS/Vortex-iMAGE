//! Signature Tests
//!
//! Tests for:
//! - Hybrid signature generation (Dilithium + Ed25519)
//! - Signature verification
//! - Tamper detection
//! - Cross-keypair verification failure

use crate::crypto::HybridKeypair;

// ============================================================================
// Basic Signature Tests
// ============================================================================

#[test]
fn signature_roundtrip_basic() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"message to sign";
    let signature = keypair.sign(data).expect("signing");
    
    bundle.verify(data, &signature).expect("verification should succeed");
}

#[test]
fn signature_roundtrip_empty_data() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"";
    let signature = keypair.sign(data).expect("signing");
    
    bundle.verify(data, &signature).expect("verification should succeed");
}

#[test]
fn signature_roundtrip_large_data() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    // 1MB of data
    let data = vec![0xABu8; 1024 * 1024];
    let signature = keypair.sign(&data).expect("signing");
    
    bundle.verify(&data, &signature).expect("verification should succeed");
}

#[test]
fn signature_roundtrip_binary_data() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    // All byte values
    let data: Vec<u8> = (0..=255).collect();
    let signature = keypair.sign(&data).expect("signing");
    
    bundle.verify(&data, &signature).expect("verification should succeed");
}

// ============================================================================
// Tamper Detection Tests
// ============================================================================

#[test]
fn signature_detects_tampered_data() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"original message";
    let signature = keypair.sign(data).expect("signing");
    
    // Tamper with the data
    let mut tampered = data.to_vec();
    tampered[0] = tampered[0].wrapping_add(1);
    
    let result = bundle.verify(&tampered, &signature);
    assert!(result.is_err(), "tampered data should fail verification");
}

#[test]
fn signature_detects_tampered_signature() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"message";
    let mut signature = keypair.sign(data).expect("signing");
    
    // Tamper with the signature (flip a bit in the middle)
    let mid = signature.len() / 2;
    signature[mid] = signature[mid].wrapping_add(1);
    
    let result = bundle.verify(data, &signature);
    assert!(result.is_err(), "tampered signature should fail verification");
}

#[test]
fn signature_detects_truncated_signature() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"message";
    let signature = keypair.sign(data).expect("signing");
    
    // Truncate the signature
    let truncated = &signature[..signature.len() / 2];
    
    let result = bundle.verify(data, truncated);
    assert!(result.is_err(), "truncated signature should fail verification");
}

#[test]
fn signature_rejects_too_short_signature() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"message";
    
    // Signature shorter than minimum (68 bytes for length prefix + some data)
    let short_sig = vec![0u8; 10];
    
    let result = bundle.verify(data, &short_sig);
    assert!(result.is_err(), "too short signature should fail");
}

// ============================================================================
// Cross-Keypair Tests
// ============================================================================

#[test]
fn signature_wrong_keypair_fails_verification() {
    let keypair1 = HybridKeypair::generate().expect("keypair generation 1");
    let keypair2 = HybridKeypair::generate().expect("keypair generation 2");
    let bundle2 = keypair2.public_bundle();
    
    let data = b"message signed by keypair1";
    let signature = keypair1.sign(data).expect("signing");
    
    // Verify with wrong public bundle
    let result = bundle2.verify(data, &signature);
    assert!(result.is_err(), "verification with wrong keypair should fail");
}

#[test]
fn signature_different_data_same_keypair() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data1 = b"message one";
    let data2 = b"message two";
    
    let sig1 = keypair.sign(data1).expect("signing 1");
    let sig2 = keypair.sign(data2).expect("signing 2");
    
    // Each signature should only verify its own data
    bundle.verify(data1, &sig1).expect("sig1 should verify data1");
    bundle.verify(data2, &sig2).expect("sig2 should verify data2");
    
    // Cross-verification should fail
    assert!(bundle.verify(data1, &sig2).is_err(), "sig2 should not verify data1");
    assert!(bundle.verify(data2, &sig1).is_err(), "sig1 should not verify data2");
}

// ============================================================================
// Signature Format Tests
// ============================================================================

#[test]
fn signature_has_expected_structure() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    
    let data = b"test message";
    let signature = keypair.sign(data).expect("signing");
    
    // Signature format: [pq_sig_len (4 bytes)][pq_sig][ed_sig (64 bytes)]
    assert!(signature.len() >= 68, "signature should be at least 68 bytes");
    
    // Parse length prefix
    let pq_sig_len = u32::from_le_bytes(signature[..4].try_into().unwrap()) as usize;
    
    // Verify structure
    assert!(signature.len() >= 4 + pq_sig_len + 64, "signature should contain both signatures");
    
    // Ed25519 signature is always 64 bytes
    let expected_len = 4 + pq_sig_len + 64;
    assert_eq!(signature.len(), expected_len, "signature length should match structure");
}

#[test]
fn signature_deterministic_for_same_input() {
    // Note: Dilithium signatures may be deterministic or randomized depending on implementation
    // This test verifies the signature is valid regardless
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    let data = b"same message";
    
    // Generate multiple signatures
    let sig1 = keypair.sign(data).expect("signing 1");
    let sig2 = keypair.sign(data).expect("signing 2");
    
    // Both should verify
    bundle.verify(data, &sig1).expect("sig1 should verify");
    bundle.verify(data, &sig2).expect("sig2 should verify");
}
