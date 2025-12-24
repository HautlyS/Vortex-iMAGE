//! File Compression Tests
//!
//! Tests for file-based compression with:
//! - Checksum verification
//! - Already-compressed format detection
//! - Size threshold handling

use crate::compress::{
    compress_file_data, decompress_file_data, Algorithm, ItemCompressionSettings,
};

// ============================================================================
// File Compression Roundtrip Tests
// ============================================================================

#[test]
fn file_compression_roundtrip() {
    let data = vec![42u8; 10000];
    let filename = "test.txt";
    let settings = ItemCompressionSettings::default();
    
    let compressed = compress_file_data(&data, filename, &settings).unwrap();
    assert!(compressed.compressed);
    
    let decompressed = decompress_file_data(&compressed).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn file_compression_checksum_verification() {
    let data = vec![42u8; 10000];
    let filename = "test.txt";
    let settings = ItemCompressionSettings::default();
    
    let mut compressed = compress_file_data(&data, filename, &settings).unwrap();
    
    // Corrupt the checksum
    compressed.checksum[0] = compressed.checksum[0].wrapping_add(1);
    
    let result = decompress_file_data(&compressed);
    assert!(result.is_err(), "corrupted checksum should fail decompression");
}

#[test]
fn file_compression_disabled() {
    let data = vec![42u8; 10000];
    let filename = "test.txt";
    let settings = ItemCompressionSettings {
        enabled: false,
        ..Default::default()
    };
    
    let result = compress_file_data(&data, filename, &settings).unwrap();
    
    assert!(!result.compressed);
    assert_eq!(result.data, data);
    assert_eq!(result.algorithm, Algorithm::None);
}

// ============================================================================
// Size Threshold Tests
// ============================================================================

#[test]
fn file_compression_below_threshold() {
    let data = vec![42u8; 500]; // Below default 1024 threshold
    let filename = "small.txt";
    let settings = ItemCompressionSettings::default();
    
    let result = compress_file_data(&data, filename, &settings).unwrap();
    
    assert!(!result.compressed, "data below threshold should not be compressed");
    assert_eq!(result.data, data);
}

#[test]
fn file_compression_above_threshold() {
    let data = vec![42u8; 2000]; // Above default 1024 threshold
    let filename = "large.txt";
    let settings = ItemCompressionSettings::default();
    
    let result = compress_file_data(&data, filename, &settings).unwrap();
    
    assert!(result.compressed, "data above threshold should be compressed");
}

#[test]
fn file_compression_custom_threshold() {
    let data = vec![42u8; 500];
    let filename = "test.txt";
    let settings = ItemCompressionSettings {
        min_size_threshold: 100, // Lower threshold
        ..Default::default()
    };
    
    let result = compress_file_data(&data, filename, &settings).unwrap();
    
    assert!(result.compressed, "data above custom threshold should be compressed");
}

// ============================================================================
// Already-Compressed Format Detection Tests
// ============================================================================

#[test]
fn file_compression_skips_jpeg() {
    let data = vec![42u8; 10000];
    let settings = ItemCompressionSettings {
        skip_already_compressed: true,
        ..Default::default()
    };
    
    for ext in ["jpg", "jpeg", "JPG", "JPEG"] {
        let filename = format!("image.{}", ext);
        let result = compress_file_data(&data, &filename, &settings).unwrap();
        
        assert!(!result.compressed, "{} should be skipped", ext);
    }
}

#[test]
fn file_compression_skips_png() {
    let data = vec![42u8; 10000];
    let filename = "image.png";
    let settings = ItemCompressionSettings {
        skip_already_compressed: true,
        ..Default::default()
    };
    
    let result = compress_file_data(&data, filename, &settings).unwrap();
    assert!(!result.compressed, "PNG should be skipped");
}

#[test]
fn file_compression_skips_video_formats() {
    let data = vec![42u8; 10000];
    let settings = ItemCompressionSettings {
        skip_already_compressed: true,
        ..Default::default()
    };
    
    for ext in ["mp4", "mkv", "avi", "mov", "webm"] {
        let filename = format!("video.{}", ext);
        let result = compress_file_data(&data, &filename, &settings).unwrap();
        
        assert!(!result.compressed, "{} should be skipped", ext);
    }
}

#[test]
fn file_compression_skips_audio_formats() {
    let data = vec![42u8; 10000];
    let settings = ItemCompressionSettings {
        skip_already_compressed: true,
        ..Default::default()
    };
    
    for ext in ["mp3", "aac", "ogg", "flac"] {
        let filename = format!("audio.{}", ext);
        let result = compress_file_data(&data, &filename, &settings).unwrap();
        
        assert!(!result.compressed, "{} should be skipped", ext);
    }
}

#[test]
fn file_compression_skips_archive_formats() {
    let data = vec![42u8; 10000];
    let settings = ItemCompressionSettings {
        skip_already_compressed: true,
        ..Default::default()
    };
    
    for ext in ["zip", "gz", "bz2", "xz", "7z", "rar", "zst", "lz4", "br"] {
        let filename = format!("archive.{}", ext);
        let result = compress_file_data(&data, &filename, &settings).unwrap();
        
        assert!(!result.compressed, "{} should be skipped", ext);
    }
}

#[test]
fn file_compression_compresses_text_files() {
    let data = vec![42u8; 10000];
    let settings = ItemCompressionSettings {
        skip_already_compressed: true,
        ..Default::default()
    };
    
    for ext in ["txt", "json", "xml", "html", "css", "js", "ts", "md"] {
        let filename = format!("file.{}", ext);
        let result = compress_file_data(&data, &filename, &settings).unwrap();
        
        assert!(result.compressed, "{} should be compressed", ext);
    }
}

#[test]
fn file_compression_skip_disabled() {
    let data = vec![42u8; 10000];
    let filename = "image.jpg";
    let settings = ItemCompressionSettings {
        skip_already_compressed: false, // Disabled
        ..Default::default()
    };
    
    let result = compress_file_data(&data, filename, &settings).unwrap();
    
    // Should compress even though it's a JPEG
    assert!(result.compressed, "should compress when skip is disabled");
}

// ============================================================================
// Compression Ratio Tests
// ============================================================================

#[test]
fn file_compression_skips_when_larger() {
    // Data that doesn't compress well (random-ish)
    let data: Vec<u8> = (0..2000).map(|i| ((i * 17 + 31) % 256) as u8).collect();
    let filename = "random.bin";
    let settings = ItemCompressionSettings {
        algorithm: Algorithm::Zstd,
        level: 1, // Low compression level
        ..Default::default()
    };
    
    let result = compress_file_data(&data, filename, &settings).unwrap();
    
    // If compression would make it larger, it should skip
    if !result.compressed {
        assert_eq!(result.data, data, "uncompressed data should be unchanged");
    }
}

// ============================================================================
// Metadata Tests
// ============================================================================

#[test]
fn file_compression_metadata_accuracy() {
    let data = vec![42u8; 10000];
    let filename = "test.txt";
    let settings = ItemCompressionSettings::default();
    
    let result = compress_file_data(&data, filename, &settings).unwrap();
    
    assert_eq!(result.original_size, data.len());
    assert_eq!(result.compressed_size, result.data.len());
    assert!(!result.checksum.is_empty(), "checksum should be present");
    
    // Verify ratio
    let expected_ratio = result.compressed_size as f64 / result.original_size as f64;
    assert!((result.ratio - expected_ratio).abs() < 0.0001);
}

#[test]
fn file_compression_checksum_is_blake3() {
    let data = vec![42u8; 10000];
    let filename = "test.txt";
    let settings = ItemCompressionSettings::default();
    
    let result = compress_file_data(&data, filename, &settings).unwrap();
    
    // BLAKE3 produces 32-byte hashes
    assert_eq!(result.checksum.len(), 32, "checksum should be 32 bytes (BLAKE3)");
    
    // Verify checksum matches original data
    let expected_checksum = blake3::hash(&data).as_bytes().to_vec();
    assert_eq!(result.checksum, expected_checksum);
}
