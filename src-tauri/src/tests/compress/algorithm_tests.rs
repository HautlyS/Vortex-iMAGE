//! Algorithm-Specific Tests
//!
//! Tests for individual compression algorithms:
//! - Zstd, LZ4, Snappy, Brotli, Gzip
//! - Algorithm parsing and validation

use crate::compress::{
    brotli_compress, brotli_decompress, gzip_compress, gzip_decompress, lz4_compress,
    lz4_decompress, snap_compress, snap_decompress, zstd_compress, zstd_decompress,
    Algorithm, CompressError,
};

// ============================================================================
// Zstd Tests
// ============================================================================

#[test]
fn zstd_roundtrip_large_data() {
    let data = vec![42u8; 10000];
    let (compressed, was_compressed) = zstd_compress(&data, 3).unwrap();
    
    assert!(was_compressed, "large data should be compressed");
    assert!(compressed.len() < data.len(), "compressed should be smaller");
    
    let decompressed = zstd_decompress(&compressed).unwrap();
    assert_eq!(data, decompressed);
}

#[test]
fn zstd_small_data_bypass() {
    // Data smaller than 64 bytes should not be compressed
    let data = vec![42u8; 32];
    let (result, was_compressed) = zstd_compress(&data, 3).unwrap();
    
    assert!(!was_compressed, "small data should not be compressed");
    assert_eq!(result, data, "uncompressed data should be unchanged");
}

#[test]
fn zstd_level_clamping() {
    let data = vec![42u8; 1000];
    
    // Level below minimum (1) should be clamped
    let (compressed_low, _) = zstd_compress(&data, -5).unwrap();
    
    // Level above maximum (22) should be clamped
    let (compressed_high, _) = zstd_compress(&data, 100).unwrap();
    
    // Both should decompress correctly
    let decompressed_low = zstd_decompress(&compressed_low).unwrap();
    let decompressed_high = zstd_decompress(&compressed_high).unwrap();
    
    assert_eq!(decompressed_low, data);
    assert_eq!(decompressed_high, data);
}

#[test]
fn zstd_empty_data() {
    let data = vec![];
    let (result, was_compressed) = zstd_compress(&data, 3).unwrap();
    
    assert!(!was_compressed, "empty data should not be compressed");
    assert!(result.is_empty());
}

// ============================================================================
// LZ4 Tests
// ============================================================================

#[test]
fn lz4_roundtrip() {
    let data = b"hello world repeated ".repeat(100);
    let compressed = lz4_compress(&data);
    let decompressed = lz4_decompress(&compressed).unwrap();
    
    assert_eq!(data.as_slice(), decompressed.as_slice());
}

#[test]
fn lz4_empty_data() {
    let data = vec![];
    let compressed = lz4_compress(&data);
    let decompressed = lz4_decompress(&compressed).unwrap();
    
    assert!(decompressed.is_empty());
}

#[test]
fn lz4_binary_data() {
    // All byte values
    let data: Vec<u8> = (0..=255).collect();
    let compressed = lz4_compress(&data);
    let decompressed = lz4_decompress(&compressed).unwrap();
    
    assert_eq!(data, decompressed);
}

#[test]
fn lz4_invalid_data_rejected() {
    // Random garbage that's not valid LZ4
    let garbage = vec![0xFF, 0xFE, 0xFD, 0xFC];
    let result = lz4_decompress(&garbage);
    
    assert!(result.is_err());
}

// ============================================================================
// Snappy Tests
// ============================================================================

#[test]
fn snap_roundtrip() {
    let data = b"test data for snap compression".repeat(50);
    let compressed = snap_compress(&data).unwrap();
    let decompressed = snap_decompress(&compressed).unwrap();
    
    assert_eq!(data.as_slice(), decompressed.as_slice());
}

#[test]
fn snap_includes_original_size() {
    let data = b"test data".repeat(10);
    let compressed = snap_compress(&data).unwrap();
    
    // First 4 bytes should be original size (little-endian)
    let stored_size = u32::from_le_bytes(compressed[..4].try_into().unwrap()) as usize;
    assert_eq!(stored_size, data.len());
}

#[test]
fn snap_too_short_data_rejected() {
    // Less than 4 bytes (no size header)
    let short = vec![0, 1, 2];
    let result = snap_decompress(&short);
    
    assert!(result.is_err());
}

#[test]
fn snap_empty_data() {
    let data = vec![];
    let compressed = snap_compress(&data).unwrap();
    let decompressed = snap_decompress(&compressed).unwrap();
    
    assert!(decompressed.is_empty());
}

// ============================================================================
// Brotli Tests
// ============================================================================

#[test]
fn brotli_roundtrip() {
    let data = vec![0xABu8; 5000];
    let compressed = brotli_compress(&data, 6).unwrap();
    let decompressed = brotli_decompress(&compressed).unwrap();
    
    assert_eq!(data, decompressed);
}

#[test]
fn brotli_level_clamping() {
    let data = vec![42u8; 1000];
    
    // Level below minimum (0) should be clamped
    let compressed_low = brotli_compress(&data, -5).unwrap();
    
    // Level above maximum (11) should be clamped
    let compressed_high = brotli_compress(&data, 100).unwrap();
    
    // Both should decompress correctly
    let decompressed_low = brotli_decompress(&compressed_low).unwrap();
    let decompressed_high = brotli_decompress(&compressed_high).unwrap();
    
    assert_eq!(decompressed_low, data);
    assert_eq!(decompressed_high, data);
}

#[test]
fn brotli_empty_data() {
    let data = vec![];
    let compressed = brotli_compress(&data, 6).unwrap();
    let decompressed = brotli_decompress(&compressed).unwrap();
    
    assert!(decompressed.is_empty());
}

// ============================================================================
// Gzip Tests
// ============================================================================

#[test]
fn gzip_roundtrip() {
    let data = b"gzip test data ".repeat(200);
    let compressed = gzip_compress(&data, 6).unwrap();
    let decompressed = gzip_decompress(&compressed).unwrap();
    
    assert_eq!(data.as_slice(), decompressed.as_slice());
}

#[test]
fn gzip_level_clamping() {
    let data = vec![42u8; 1000];
    
    // Level below minimum (0) should be clamped
    let compressed_low = gzip_compress(&data, -5).unwrap();
    
    // Level above maximum (9) should be clamped
    let compressed_high = gzip_compress(&data, 100).unwrap();
    
    // Both should decompress correctly
    let decompressed_low = gzip_decompress(&compressed_low).unwrap();
    let decompressed_high = gzip_decompress(&compressed_high).unwrap();
    
    assert_eq!(decompressed_low, data);
    assert_eq!(decompressed_high, data);
}

#[test]
fn gzip_empty_data() {
    let data = vec![];
    let compressed = gzip_compress(&data, 6).unwrap();
    let decompressed = gzip_decompress(&compressed).unwrap();
    
    assert!(decompressed.is_empty());
}

// ============================================================================
// Algorithm Parsing Tests
// ============================================================================

#[test]
fn algorithm_from_str_valid() {
    assert_eq!(Algorithm::from("zstd"), Algorithm::Zstd);
    assert_eq!(Algorithm::from("lz4"), Algorithm::Lz4);
    assert_eq!(Algorithm::from("snap"), Algorithm::Snap);
    assert_eq!(Algorithm::from("snappy"), Algorithm::Snap);
    assert_eq!(Algorithm::from("brotli"), Algorithm::Brotli);
    assert_eq!(Algorithm::from("br"), Algorithm::Brotli);
    assert_eq!(Algorithm::from("gzip"), Algorithm::Gzip);
    assert_eq!(Algorithm::from("gz"), Algorithm::Gzip);
    assert_eq!(Algorithm::from("none"), Algorithm::None);
}

#[test]
fn algorithm_from_str_case_insensitive() {
    assert_eq!(Algorithm::from("ZSTD"), Algorithm::Zstd);
    assert_eq!(Algorithm::from("Zstd"), Algorithm::Zstd);
    assert_eq!(Algorithm::from("LZ4"), Algorithm::Lz4);
    assert_eq!(Algorithm::from("BROTLI"), Algorithm::Brotli);
}

#[test]
fn algorithm_from_str_unknown_defaults_to_zstd() {
    assert_eq!(Algorithm::from("unknown"), Algorithm::Zstd);
    assert_eq!(Algorithm::from("invalid"), Algorithm::Zstd);
    assert_eq!(Algorithm::from(""), Algorithm::Zstd);
}

#[test]
fn algorithm_try_from_str_valid() {
    assert!(Algorithm::try_from_str("zstd").is_ok());
    assert!(Algorithm::try_from_str("lz4").is_ok());
    assert!(Algorithm::try_from_str("ZSTD").is_ok()); // Case insensitive
}

#[test]
fn algorithm_try_from_str_invalid() {
    assert!(Algorithm::try_from_str("invalid").is_err());
    assert!(Algorithm::try_from_str("xyz").is_err());
    assert!(Algorithm::try_from_str("").is_err());
}

#[test]
fn algorithm_try_from_str_error_contains_name() {
    if let Err(CompressError::UnsupportedAlgorithm(name)) = Algorithm::try_from_str("invalid") {
        assert_eq!(name, "invalid");
    } else {
        panic!("expected UnsupportedAlgorithm error");
    }
}
