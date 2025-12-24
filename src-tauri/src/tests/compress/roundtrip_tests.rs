//! Compression Roundtrip Tests
//!
//! Tests for the high-level compress/decompress API:
//! - Full roundtrip with settings
//! - Auto-selection algorithm
//! - Compression result transparency

use crate::compress::{
    compress, compress_auto, decompress, select_algorithm, Algorithm, CompressionSettings,
};

// ============================================================================
// Basic Roundtrip Tests
// ============================================================================

#[test]
fn compress_decompress_roundtrip_zstd() {
    let data = vec![42u8; 10000];
    let settings = CompressionSettings {
        algorithm: Algorithm::Zstd,
        level: 3,
        prefer_speed: false,
    };
    
    let result = compress(&data, &settings).unwrap();
    assert!(result.was_compressed);
    
    let decompressed = decompress(&result.data, result.algorithm).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn compress_decompress_roundtrip_lz4() {
    let data = vec![42u8; 10000];
    let settings = CompressionSettings {
        algorithm: Algorithm::Lz4,
        level: 1,
        prefer_speed: true,
    };
    
    let result = compress(&data, &settings).unwrap();
    assert!(result.was_compressed);
    
    let decompressed = decompress(&result.data, result.algorithm).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn compress_decompress_roundtrip_all_algorithms() {
    let data = vec![42u8; 10000];
    let algorithms = [
        Algorithm::Zstd,
        Algorithm::Lz4,
        Algorithm::Snap,
        Algorithm::Brotli,
        Algorithm::Gzip,
    ];
    
    for algo in algorithms {
        let settings = CompressionSettings {
            algorithm: algo,
            level: 3,
            prefer_speed: false,
        };
        
        let result = compress(&data, &settings).unwrap();
        let decompressed = decompress(&result.data, result.algorithm).unwrap();
        
        assert_eq!(decompressed, data, "roundtrip failed for {:?}", algo);
    }
}

// ============================================================================
// Auto-Selection Tests
// ============================================================================

#[test]
fn compress_auto_prefer_speed() {
    let data = vec![42u8; 10000];
    
    let result = compress_auto(&data, true).unwrap();
    assert_eq!(result.algorithm, Algorithm::Lz4, "prefer_speed should use LZ4");
    assert!(result.was_compressed);
}

#[test]
fn compress_auto_prefer_ratio() {
    let data = vec![42u8; 10000];
    
    let result = compress_auto(&data, false).unwrap();
    assert_eq!(result.algorithm, Algorithm::Zstd, "prefer_ratio should use Zstd");
    assert!(result.was_compressed);
}

#[test]
fn select_algorithm_small_data() {
    let data = vec![1, 2, 3]; // Less than 64 bytes
    
    let algo = select_algorithm(&data, false);
    assert_eq!(algo, Algorithm::None, "small data should not be compressed");
}

#[test]
fn select_algorithm_large_data_speed() {
    let data = vec![42u8; 1000];
    
    let algo = select_algorithm(&data, true);
    assert_eq!(algo, Algorithm::Lz4);
}

#[test]
fn select_algorithm_large_data_ratio() {
    let data = vec![42u8; 1000];
    
    let algo = select_algorithm(&data, false);
    assert_eq!(algo, Algorithm::Zstd);
}

// ============================================================================
// Compression Result Transparency Tests
// ============================================================================

#[test]
fn compression_result_small_data_not_compressed() {
    let small_data = vec![1u8; 32];
    let settings = CompressionSettings {
        algorithm: Algorithm::Zstd,
        level: 3,
        prefer_speed: false,
    };
    
    let result = compress(&small_data, &settings).unwrap();
    
    assert!(!result.was_compressed, "small data should not be compressed");
    assert_eq!(result.data, small_data, "uncompressed data should be unchanged");
    assert_eq!(result.original_size, result.compressed_size);
    assert_eq!(result.ratio, 1.0);
}

#[test]
fn compression_result_large_data_compressed() {
    let large_data = vec![42u8; 10000];
    let settings = CompressionSettings {
        algorithm: Algorithm::Zstd,
        level: 3,
        prefer_speed: false,
    };
    
    let result = compress(&large_data, &settings).unwrap();
    
    assert!(result.was_compressed, "large data should be compressed");
    assert!(result.compressed_size < result.original_size, "compressed should be smaller");
    assert!(result.ratio < 1.0, "ratio should be less than 1");
}

#[test]
fn compression_result_algorithm_none_never_compresses() {
    let large_data = vec![42u8; 10000];
    let settings = CompressionSettings {
        algorithm: Algorithm::None,
        level: 0,
        prefer_speed: false,
    };
    
    let result = compress(&large_data, &settings).unwrap();
    
    assert!(!result.was_compressed, "Algorithm::None should not compress");
    assert_eq!(result.data, large_data, "data should be unchanged");
    assert_eq!(result.algorithm, Algorithm::None);
}

#[test]
fn compression_result_metadata_accuracy() {
    let data = vec![42u8; 10000];
    let settings = CompressionSettings {
        algorithm: Algorithm::Zstd,
        level: 3,
        prefer_speed: false,
    };
    
    let result = compress(&data, &settings).unwrap();
    
    assert_eq!(result.original_size, data.len());
    assert_eq!(result.compressed_size, result.data.len());
    assert_eq!(result.algorithm, Algorithm::Zstd);
    
    // Verify ratio calculation
    let expected_ratio = result.compressed_size as f64 / result.original_size as f64;
    assert!((result.ratio - expected_ratio).abs() < 0.0001);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn compress_empty_data() {
    let data = vec![];
    let settings = CompressionSettings::default();
    
    let result = compress(&data, &settings).unwrap();
    
    // Empty data behavior depends on algorithm
    assert_eq!(result.original_size, 0);
}

#[test]
fn compress_incompressible_data() {
    // Random-looking data that doesn't compress well
    let data: Vec<u8> = (0..1000).map(|i| ((i * 17 + 31) % 256) as u8).collect();
    let settings = CompressionSettings {
        algorithm: Algorithm::Zstd,
        level: 1,
        prefer_speed: true,
    };
    
    let result = compress(&data, &settings).unwrap();
    
    // Should still work, even if compression ratio is poor
    let decompressed = decompress(&result.data, result.algorithm).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn decompress_algorithm_none() {
    let data = vec![1, 2, 3, 4, 5];
    
    let result = decompress(&data, Algorithm::None).unwrap();
    assert_eq!(result, data, "Algorithm::None should return data unchanged");
}
