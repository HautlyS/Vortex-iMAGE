//! Multi-algorithm compression module
//! Supports: Zstd, LZ4, Snap, Brotli, Gzip
//! Based on exemple-alghos/compress implementation

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressError {
    #[error("compression failed: {0}")]
    Compress(String),
    #[error("decompression failed: {0}")]
    Decompress(String),
    #[error("invalid data")]
    InvalidData,
    #[error("unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
}

impl Serialize for CompressError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Compression algorithm selection
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Algorithm {
    #[default]
    Zstd,
    Lz4,
    Snap,
    Brotli,
    Gzip,
    None,
}

impl From<&str> for Algorithm {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "zstd" => Self::Zstd,
            "lz4" => Self::Lz4,
            "snap" | "snappy" => Self::Snap,
            "brotli" | "br" => Self::Brotli,
            "gzip" | "gz" => Self::Gzip,
            "none" => Self::None,
            _ => Self::Zstd,
        }
    }
}

/// Compression settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressionSettings {
    pub algorithm: Algorithm,
    pub level: i32,
    pub prefer_speed: bool,
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::Zstd,
            level: 3,
            prefer_speed: false,
        }
    }
}

/// Compression result with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressionResult {
    pub data: Vec<u8>,
    pub algorithm: Algorithm,
    pub original_size: usize,
    pub compressed_size: usize,
    pub ratio: f64,
}

// ============================================================================
// Zstd Compression
// ============================================================================

pub fn zstd_compress(data: &[u8], level: i32) -> Result<Vec<u8>, CompressError> {
    if data.len() < 64 {
        return Ok(data.to_vec());
    }
    let level = level.clamp(1, 22);
    zstd::encode_all(data, level)
        .map_err(|e| CompressError::Compress(e.to_string()))
}

pub fn zstd_decompress(data: &[u8]) -> Result<Vec<u8>, CompressError> {
    zstd::decode_all(data)
        .map_err(|e| CompressError::Decompress(e.to_string()))
}

// ============================================================================
// LZ4 Compression (fast)
// ============================================================================

pub fn lz4_compress(data: &[u8]) -> Vec<u8> {
    lz4_flex::compress_prepend_size(data)
}

pub fn lz4_decompress(data: &[u8]) -> Result<Vec<u8>, CompressError> {
    lz4_flex::decompress_size_prepended(data)
        .map_err(|_| CompressError::InvalidData)
}

// ============================================================================
// Snap Compression (very fast)
// ============================================================================

pub fn snap_compress(data: &[u8]) -> Result<Vec<u8>, CompressError> {
    let mut encoder = snap::raw::Encoder::new();
    let compressed = encoder.compress_vec(data)
        .map_err(|e| CompressError::Compress(e.to_string()))?;
    
    // Prepend original size for decompression
    let mut output = Vec::with_capacity(4 + compressed.len());
    output.extend_from_slice(&(data.len() as u32).to_le_bytes());
    output.extend_from_slice(&compressed);
    Ok(output)
}

pub fn snap_decompress(data: &[u8]) -> Result<Vec<u8>, CompressError> {
    if data.len() < 4 {
        return Err(CompressError::InvalidData);
    }
    let original_size = u32::from_le_bytes(data[..4].try_into().unwrap()) as usize;
    let mut decoder = snap::raw::Decoder::new();
    let mut output = vec![0u8; original_size];
    decoder.decompress(&data[4..], &mut output)
        .map_err(|e| CompressError::Decompress(e.to_string()))?;
    Ok(output)
}

// ============================================================================
// Brotli Compression (high ratio)
// ============================================================================

pub fn brotli_compress(data: &[u8], level: i32) -> Result<Vec<u8>, CompressError> {
    let quality = level.clamp(0, 11);
    let mut output = Vec::new();
    let mut params = brotli::enc::BrotliEncoderParams::default();
    params.quality = quality;
    
    brotli::BrotliCompress(&mut &data[..], &mut output, &params)
        .map_err(|e| CompressError::Compress(e.to_string()))?;
    Ok(output)
}

pub fn brotli_decompress(data: &[u8]) -> Result<Vec<u8>, CompressError> {
    let mut output = Vec::new();
    brotli::BrotliDecompress(&mut &data[..], &mut output)
        .map_err(|e| CompressError::Decompress(e.to_string()))?;
    Ok(output)
}

// ============================================================================
// Gzip Compression (compatible)
// ============================================================================

pub fn gzip_compress(data: &[u8], level: i32) -> Result<Vec<u8>, CompressError> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    
    let level = level.clamp(0, 9) as u32;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
    encoder.write_all(data)
        .map_err(|e| CompressError::Compress(e.to_string()))?;
    encoder.finish()
        .map_err(|e| CompressError::Compress(e.to_string()))
}

pub fn gzip_decompress(data: &[u8]) -> Result<Vec<u8>, CompressError> {
    use flate2::read::GzDecoder;
    
    let mut decoder = GzDecoder::new(data);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output)
        .map_err(|e| CompressError::Decompress(e.to_string()))?;
    Ok(output)
}

// ============================================================================
// Unified Compression API
// ============================================================================

/// Compress data with specified algorithm
pub fn compress(data: &[u8], settings: &CompressionSettings) -> Result<CompressionResult, CompressError> {
    let original_size = data.len();
    
    let compressed = match settings.algorithm {
        Algorithm::Zstd => zstd_compress(data, settings.level)?,
        Algorithm::Lz4 => lz4_compress(data),
        Algorithm::Snap => snap_compress(data)?,
        Algorithm::Brotli => brotli_compress(data, settings.level)?,
        Algorithm::Gzip => gzip_compress(data, settings.level)?,
        Algorithm::None => data.to_vec(),
    };
    
    let compressed_size = compressed.len();
    let ratio = if original_size > 0 {
        compressed_size as f64 / original_size as f64
    } else {
        1.0
    };
    
    Ok(CompressionResult {
        data: compressed,
        algorithm: settings.algorithm,
        original_size,
        compressed_size,
        ratio,
    })
}

/// Decompress data with specified algorithm
pub fn decompress(data: &[u8], algorithm: Algorithm) -> Result<Vec<u8>, CompressError> {
    match algorithm {
        Algorithm::Zstd => zstd_decompress(data),
        Algorithm::Lz4 => lz4_decompress(data),
        Algorithm::Snap => snap_decompress(data),
        Algorithm::Brotli => brotli_decompress(data),
        Algorithm::Gzip => gzip_decompress(data),
        Algorithm::None => Ok(data.to_vec()),
    }
}

/// Auto-select best algorithm based on data characteristics
pub fn select_algorithm(data: &[u8], prefer_speed: bool) -> Algorithm {
    if data.len() < 64 {
        return Algorithm::None;
    }
    
    if prefer_speed {
        Algorithm::Lz4
    } else {
        Algorithm::Zstd
    }
}

/// Compress with automatic algorithm selection
pub fn compress_auto(data: &[u8], prefer_speed: bool) -> Result<CompressionResult, CompressError> {
    let algorithm = select_algorithm(data, prefer_speed);
    let settings = CompressionSettings {
        algorithm,
        level: if prefer_speed { 1 } else { 3 },
        prefer_speed,
    };
    compress(data, &settings)
}

// ============================================================================
// Per-Item Compression Settings
// ============================================================================

/// Compression settings for photos/albums with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemCompressionSettings {
    pub enabled: bool,
    pub algorithm: Algorithm,
    pub level: i32,
    pub prefer_speed: bool,
    pub min_size_threshold: usize, // Don't compress files smaller than this
    pub skip_already_compressed: bool, // Skip JPEG, PNG, etc.
}

impl Default for ItemCompressionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: Algorithm::Zstd,
            level: 3,
            prefer_speed: false,
            min_size_threshold: 1024, // 1KB minimum
            skip_already_compressed: true,
        }
    }
}

/// Result of compression with full metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressedFileData {
    pub data: Vec<u8>,
    pub compressed: bool,
    pub algorithm: Algorithm,
    pub original_size: usize,
    pub compressed_size: usize,
    pub ratio: f64,
    pub checksum: Vec<u8>, // BLAKE3 hash of original
}

/// Check if file extension indicates already compressed format
fn is_compressed_format(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(ext.as_str(), 
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "avif" | "heic" | "heif" |
        "mp4" | "mkv" | "avi" | "mov" | "webm" |
        "mp3" | "aac" | "ogg" | "flac" |
        "zip" | "gz" | "bz2" | "xz" | "7z" | "rar" |
        "zst" | "lz4" | "br"
    )
}

/// Compress file data with settings
pub fn compress_file_data(
    data: &[u8],
    filename: &str,
    settings: &ItemCompressionSettings,
) -> Result<CompressedFileData, CompressError> {
    // Calculate checksum first
    let checksum = blake3::hash(data).as_bytes().to_vec();
    
    // Check if compression should be skipped
    if !settings.enabled {
        return Ok(CompressedFileData {
            data: data.to_vec(),
            compressed: false,
            algorithm: Algorithm::None,
            original_size: data.len(),
            compressed_size: data.len(),
            ratio: 1.0,
            checksum,
        });
    }
    
    if data.len() < settings.min_size_threshold {
        return Ok(CompressedFileData {
            data: data.to_vec(),
            compressed: false,
            algorithm: Algorithm::None,
            original_size: data.len(),
            compressed_size: data.len(),
            ratio: 1.0,
            checksum,
        });
    }
    
    if settings.skip_already_compressed && is_compressed_format(filename) {
        return Ok(CompressedFileData {
            data: data.to_vec(),
            compressed: false,
            algorithm: Algorithm::None,
            original_size: data.len(),
            compressed_size: data.len(),
            ratio: 1.0,
            checksum,
        });
    }
    
    // Perform compression
    let comp_settings = CompressionSettings {
        algorithm: settings.algorithm,
        level: settings.level,
        prefer_speed: settings.prefer_speed,
    };
    
    let result = compress(data, &comp_settings)?;
    
    // Only use compressed if it's actually smaller
    if result.compressed_size >= data.len() {
        return Ok(CompressedFileData {
            data: data.to_vec(),
            compressed: false,
            algorithm: Algorithm::None,
            original_size: data.len(),
            compressed_size: data.len(),
            ratio: 1.0,
            checksum,
        });
    }
    
    Ok(CompressedFileData {
        data: result.data,
        compressed: true,
        algorithm: result.algorithm,
        original_size: result.original_size,
        compressed_size: result.compressed_size,
        ratio: result.ratio,
        checksum,
    })
}

/// Decompress file data and verify checksum
pub fn decompress_file_data(
    compressed: &CompressedFileData,
) -> Result<Vec<u8>, CompressError> {
    if !compressed.compressed {
        return Ok(compressed.data.clone());
    }
    
    let decompressed = decompress(&compressed.data, compressed.algorithm)?;
    
    // Verify checksum
    let checksum = blake3::hash(&decompressed).as_bytes().to_vec();
    if checksum != compressed.checksum {
        return Err(CompressError::Decompress("checksum mismatch - data corrupted".into()));
    }
    
    Ok(decompressed)
}

// ============================================================================
// Tauri Commands
// ============================================================================

use crate::github::AppError;

/// Compress data with specified algorithm
#[tauri::command]
pub async fn compress_data(
    data: Vec<u8>,
    algorithm: String,
    level: Option<i32>,
) -> Result<CompressionResult, AppError> {
    let settings = CompressionSettings {
        algorithm: Algorithm::from(algorithm.as_str()),
        level: level.unwrap_or(3),
        prefer_speed: false,
    };
    
    compress(&data, &settings)
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// Decompress data with specified algorithm
#[tauri::command]
pub async fn decompress_data(
    data: Vec<u8>,
    algorithm: String,
) -> Result<Vec<u8>, AppError> {
    let algo = Algorithm::from(algorithm.as_str());
    decompress(&data, algo)
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// Get compression info for data without compressing
#[tauri::command]
pub async fn estimate_compression(
    data: Vec<u8>,
    algorithm: String,
) -> Result<CompressionResult, AppError> {
    let settings = CompressionSettings {
        algorithm: Algorithm::from(algorithm.as_str()),
        level: 3,
        prefer_speed: false,
    };
    
    compress(&data, &settings)
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// List available compression algorithms
#[tauri::command]
pub fn list_compression_algorithms() -> Vec<String> {
    vec![
        "zstd".to_string(),
        "lz4".to_string(),
        "snap".to_string(),
        "brotli".to_string(),
        "gzip".to_string(),
        "none".to_string(),
    ]
}

/// Compress data with automatic algorithm selection
#[tauri::command]
pub async fn compress_data_auto(
    data: Vec<u8>,
    prefer_speed: bool,
) -> Result<CompressionResult, AppError> {
    compress_auto(&data, prefer_speed)
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// Compress file with per-item settings
#[tauri::command]
pub async fn compress_file(
    data: Vec<u8>,
    filename: String,
    settings: ItemCompressionSettings,
) -> Result<CompressedFileData, AppError> {
    compress_file_data(&data, &filename, &settings)
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// Decompress file and verify integrity
#[tauri::command]
pub async fn decompress_file(
    compressed: CompressedFileData,
) -> Result<Vec<u8>, AppError> {
    decompress_file_data(&compressed)
        .map_err(|e| AppError::Validation(e.to_string()))
}

/// Get compression algorithm recommendations based on file type
#[tauri::command]
pub fn get_compression_recommendation(filename: String, file_size: usize) -> serde_json::Value {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    
    let (algorithm, level, reason) = if is_compressed_format(&filename) {
        ("none", 0, "File is already in a compressed format")
    } else if file_size < 1024 {
        ("none", 0, "File too small to benefit from compression")
    } else if file_size > 100 * 1024 * 1024 {
        ("lz4", 1, "Large file - using fast compression")
    } else if matches!(ext.as_str(), "txt" | "json" | "xml" | "html" | "css" | "js" | "ts") {
        ("zstd", 6, "Text file - high compression ratio recommended")
    } else if matches!(ext.as_str(), "bmp" | "tiff" | "tif" | "raw") {
        ("zstd", 3, "Uncompressed image - good compression potential")
    } else {
        ("zstd", 3, "Default balanced compression")
    };
    
    serde_json::json!({
        "algorithm": algorithm,
        "level": level,
        "reason": reason,
        "estimated_ratio": if algorithm == "none" { 1.0 } else { 0.6 }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zstd_roundtrip() {
        let data = vec![42u8; 10000];
        let compressed = zstd_compress(&data, 3).unwrap();
        let decompressed = zstd_decompress(&compressed).unwrap();
        assert_eq!(data, decompressed);
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_lz4_roundtrip() {
        let data = b"hello world repeated ".repeat(100);
        let compressed = lz4_compress(&data);
        let decompressed = lz4_decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_snap_roundtrip() {
        let data = b"test data for snap compression".repeat(50);
        let compressed = snap_compress(&data).unwrap();
        let decompressed = snap_decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_brotli_roundtrip() {
        let data = vec![0xABu8; 5000];
        let compressed = brotli_compress(&data, 6).unwrap();
        let decompressed = brotli_decompress(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_gzip_roundtrip() {
        let data = b"gzip test data ".repeat(200);
        let compressed = gzip_compress(&data, 6).unwrap();
        let decompressed = gzip_decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_compress_auto() {
        let data = vec![42u8; 10000];
        
        // Speed preference
        let result = compress_auto(&data, true).unwrap();
        assert_eq!(result.algorithm, Algorithm::Lz4);
        
        // Ratio preference
        let result = compress_auto(&data, false).unwrap();
        assert_eq!(result.algorithm, Algorithm::Zstd);
    }

    #[test]
    fn test_small_data_bypass() {
        let data = vec![1, 2, 3];
        let algo = select_algorithm(&data, false);
        assert_eq!(algo, Algorithm::None);
    }
}
