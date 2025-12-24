//! Keypair Management Tests
//!
//! Tests for:
//! - Keypair generation and metadata
//! - Opaque handle system (uniqueness, lookup, release)
//! - Key rotation with backward compatibility
//! - Keypair serialization/deserialization

use crate::crypto::{HybridKeypair, KeypairStore, SecretBytes, SecretKey32};
use std::collections::HashSet;

// ============================================================================
// Keypair Generation Tests
// ============================================================================

#[test]
fn keypair_generation_produces_valid_metadata() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    
    // created_at should be recent (within last minute)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!(keypair.created_at <= now, "created_at should not be in the future");
    assert!(keypair.created_at >= now - 60, "created_at should be recent");
    
    // Initial rotation count should be 0
    assert_eq!(keypair.rotation_count, 0, "new keypair should have rotation_count = 0");
    
    // Key ID should be non-empty hex string
    let bundle = keypair.public_bundle();
    assert!(!bundle.key_id.is_empty(), "key_id should not be empty");
    assert!(
        bundle.key_id.chars().all(|c| c.is_ascii_hexdigit()),
        "key_id should be hex string"
    );
}

#[test]
fn keypair_public_bundle_contains_all_public_keys() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bundle = keypair.public_bundle();
    
    // Verify all public key components are present
    assert!(!bundle.pq_encap.is_empty(), "PQ encapsulation key should be present");
    assert!(!bundle.pq_verify.is_empty(), "PQ verification key should be present");
    assert_ne!(bundle.x25519, [0u8; 32], "X25519 public key should not be zero");
    assert_ne!(bundle.ed_verify, [0u8; 32], "Ed25519 verification key should not be zero");
}

#[test]
fn keypair_serialization_roundtrip() {
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let bytes = keypair.to_bytes();
    let restored = HybridKeypair::from_bytes(&bytes).expect("deserialization");
    
    // Verify all fields match
    assert_eq!(keypair.pq_encap_key, restored.pq_encap_key);
    assert_eq!(keypair.x25519_public, restored.x25519_public);
    assert_eq!(keypair.pq_verifying_key, restored.pq_verifying_key);
    assert_eq!(keypair.ed_verifying_key, restored.ed_verifying_key);
    assert_eq!(keypair.created_at, restored.created_at);
    assert_eq!(keypair.rotation_count, restored.rotation_count);
}

#[test]
fn keypair_deserialization_rejects_invalid_data() {
    // Empty data
    assert!(HybridKeypair::from_bytes(&[]).is_err(), "empty data should fail");
    
    // Too short
    assert!(HybridKeypair::from_bytes(&[0u8; 10]).is_err(), "short data should fail");
    assert!(HybridKeypair::from_bytes(&[0u8; 100]).is_err(), "insufficient data should fail");
    
    // Random garbage (unlikely to have valid length prefixes)
    let garbage: Vec<u8> = (0..500).map(|i| (i % 256) as u8).collect();
    assert!(HybridKeypair::from_bytes(&garbage).is_err(), "garbage data should fail");
}

// ============================================================================
// Opaque Handle System Tests
// ============================================================================

#[test]
fn handle_generation_produces_unique_handles() {
    let mut store = KeypairStore::new();
    let mut handles = HashSet::new();
    
    for i in 0..50 {
        let keypair = HybridKeypair::generate().expect("keypair generation");
        let handle = store.insert(keypair);
        assert!(handles.insert(handle), "Handle {} was not unique (iteration {})", handle, i);
    }
    
    assert_eq!(handles.len(), 50, "Should have 50 unique handles");
}

#[test]
fn handle_lookup_returns_correct_keypair() {
    let mut store = KeypairStore::new();
    
    // Insert keypairs and track handles with their key_ids
    let mut handle_to_key_id = Vec::new();
    for _ in 0..10 {
        let keypair = HybridKeypair::generate().expect("keypair generation");
        let key_id = keypair.public_bundle().key_id.clone();
        let handle = store.insert(keypair);
        handle_to_key_id.push((handle, key_id));
    }
    
    // Verify each handle returns the correct keypair
    for (handle, expected_key_id) in &handle_to_key_id {
        let kp_arc = store.get(*handle).expect("handle should exist");
        let kp = kp_arc.lock().unwrap();
        let actual_key_id = kp.public_bundle().key_id;
        assert_eq!(&actual_key_id, expected_key_id, "Handle {} returned wrong keypair", handle);
    }
}

#[test]
fn handle_lookup_returns_none_for_invalid_handles() {
    let mut store = KeypairStore::new();
    
    // Insert a few keypairs
    let mut valid_handles = Vec::new();
    for _ in 0..5 {
        let keypair = HybridKeypair::generate().expect("keypair generation");
        valid_handles.push(store.insert(keypair));
    }
    
    // Invalid handles should return None
    let invalid_handles = [0, 999, 1000, u64::MAX];
    for invalid in invalid_handles {
        if !valid_handles.contains(&invalid) {
            assert!(
                store.get(invalid).is_none(),
                "Invalid handle {} should return None",
                invalid
            );
        }
    }
}

#[test]
fn handle_release_removes_keypair() {
    let mut store = KeypairStore::new();
    
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let handle = store.insert(keypair);
    
    // Should exist before removal
    assert!(store.get(handle).is_some(), "handle should exist before removal");
    
    // Remove and verify cleanup
    let removed = store.remove(handle);
    assert!(removed.is_some(), "remove should return the keypair");
    
    // Should not exist after removal
    assert!(store.get(handle).is_none(), "handle should not exist after removal");
    
    // Double removal should return None
    assert!(store.remove(handle).is_none(), "double removal should return None");
}

#[test]
fn handle_release_also_removes_rotated_keypairs() {
    let mut store = KeypairStore::new();
    
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let handle = store.insert(keypair);
    
    // Rotate a few times
    for _ in 0..3 {
        store.rotate(handle).expect("rotation");
    }
    
    // Should have multiple keypairs for decryption
    let all_before = store.get_all_for_decryption(handle);
    assert!(all_before.len() >= 4, "should have current + 3 rotated keypairs");
    
    // Remove the handle
    store.remove(handle);
    
    // All keypairs should be gone
    let all_after = store.get_all_for_decryption(handle);
    assert!(all_after.is_empty(), "all keypairs should be removed");
}

// ============================================================================
// Key Rotation Tests
// ============================================================================

#[test]
fn rotation_generates_new_keypair_with_different_key_id() {
    let mut store = KeypairStore::new();
    
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let old_bundle = keypair.public_bundle();
    let handle = store.insert(keypair);
    
    // Rotate
    let new_bundle = store.rotate(handle).expect("rotation");
    
    // Key ID should change
    assert_ne!(
        old_bundle.key_id, new_bundle.key_id,
        "key_id should change after rotation"
    );
    
    // Public keys should be different
    assert_ne!(old_bundle.pq_encap, new_bundle.pq_encap);
    assert_ne!(old_bundle.x25519, new_bundle.x25519);
}

#[test]
fn rotation_increments_rotation_count() {
    let mut store = KeypairStore::new();
    
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let handle = store.insert(keypair);
    
    // Initial rotation count should be 0
    {
        let kp_arc = store.get(handle).unwrap();
        let kp = kp_arc.lock().unwrap();
        assert_eq!(kp.rotation_count, 0, "initial rotation_count should be 0");
    }
    
    // Rotate multiple times and verify count
    for expected_count in 1..=5 {
        store.rotate(handle).expect("rotation");
        let kp_arc = store.get(handle).unwrap();
        let kp = kp_arc.lock().unwrap();
        assert_eq!(
            kp.rotation_count, expected_count,
            "after {} rotations, count should be {}",
            expected_count, expected_count
        );
    }
}

#[test]
fn rotation_preserves_old_keypairs_for_decryption() {
    let mut store = KeypairStore::new();
    
    let keypair = HybridKeypair::generate().expect("keypair generation");
    let handle = store.insert(keypair);
    
    // Rotate 3 times
    for _ in 0..3 {
        store.rotate(handle).expect("rotation");
    }
    
    // Should have current + 3 rotated = 4 keypairs
    let all_keypairs = store.get_all_for_decryption(handle);
    assert_eq!(all_keypairs.len(), 4, "should have 4 keypairs for decryption");
    
    // Current keypair should be first
    let current = store.get(handle).unwrap();
    let first_for_decryption = &all_keypairs[0];
    
    // Compare by key_id
    let current_id = current.lock().unwrap().public_bundle().key_id;
    let first_id = first_for_decryption.lock().unwrap().public_bundle().key_id;
    assert_eq!(current_id, first_id, "current keypair should be first in decryption list");
}

#[test]
fn rotation_fails_for_invalid_handle() {
    let mut store = KeypairStore::new();
    
    let result = store.rotate(999);
    assert!(result.is_err(), "rotation should fail for invalid handle");
}

// ============================================================================
// Secret Type Tests
// ============================================================================

#[test]
fn secret_bytes_clone_secret_produces_equal_bytes() {
    let data = vec![0xAB, 0xCD, 0xEF, 0x12, 0x34];
    let secret = SecretBytes::new(data.clone());
    let cloned = secret.clone_secret();
    
    assert_eq!(secret.as_slice(), cloned.as_slice());
    assert_eq!(secret.len(), cloned.len());
    assert_eq!(secret.len(), data.len());
}

#[test]
fn secret_key32_clone_secret_produces_equal_bytes() {
    let data = [0xABu8; 32];
    let secret = SecretKey32::new(data);
    let cloned = secret.clone_secret();
    
    assert_eq!(secret.as_bytes(), cloned.as_bytes());
    assert_eq!(secret.as_bytes(), &data);
}

#[test]
fn secret_bytes_is_empty_works_correctly() {
    let empty = SecretBytes::new(vec![]);
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
    
    let non_empty = SecretBytes::new(vec![1, 2, 3]);
    assert!(!non_empty.is_empty());
    assert_eq!(non_empty.len(), 3);
}
