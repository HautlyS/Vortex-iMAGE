//! Async segmented compression with Zstd + Tokio + VortexCodec
//! 
//! # Vortex Compression System
//! 
//! Revolutionary compression using nonary (base-9) encoding:
//! - `VortexCodec` - Core nonary arithmetic coding
//! - `entropy` - Adaptive compression based on data analysis
//! - Standard Zstd/LZ4 for compatibility

use async_compression::tokio::bufread::{ZstdDecoder, ZstdEncoder};
use bytes::{Bytes, BytesMut};
use std::io;
use thiserror::Error;
use tokio::io::{AsyncReadExt, BufReader};

pub mod vortex_codec;
pub mod entropy;

pub use vortex_codec::{
    VortexCodec, Nonary, SymbolTable,
    vortex_compress, vortex_decompress,
    vortex_compress_segmented, vortex_decompress_segmented,
};

pub use entropy::{
    calculate_entropy, classify_entropy, EntropyClass,
    adaptive_compress, adaptive_decompress,
    rle_encode, rle_decode,
};

pub const SEGMENT_SIZE: usize = 64 * 1024; // 64KB segments

#[derive(Error, Debug)]
pub enum CompressError {
    #[error("compression failed: {0}")]
    Compress(#[from] io::Error),
    #[error("invalid data")]
    InvalidData,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Algorithm {
    #[default]
    Zstd = 0,
    Lz4 = 1,
    None = 255,
}

impl From<u8> for Algorithm {
    fn from(v: u8) -> Self {
        match v { 0 => Self::Zstd, 1 => Self::Lz4, _ => Self::None }
    }
}

/// Async compress with Zstd streaming
pub async fn compress_async(data: &[u8], level: i32) -> Result<Bytes, CompressError> {
    if data.len() < 64 { return Ok(Bytes::copy_from_slice(data)); }
    let reader = BufReader::new(data);
    let mut encoder = ZstdEncoder::with_quality(reader, async_compression::Level::Precise(level));
    let mut output = Vec::with_capacity(data.len() / 2);
    encoder.read_to_end(&mut output).await?;
    Ok(Bytes::from(output))
}

/// Async decompress with Zstd streaming
pub async fn decompress_async(data: &[u8]) -> Result<Bytes, CompressError> {
    let reader = BufReader::new(data);
    let mut decoder = ZstdDecoder::new(reader);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output).await?;
    Ok(Bytes::from(output))
}

/// Segmented compression: compress in parallel chunks
pub async fn compress_segmented(data: &[u8], level: i32) -> Result<Bytes, CompressError> {
    if data.len() <= SEGMENT_SIZE { return compress_async(data, level).await; }
    
    let chunks: Vec<_> = data.chunks(SEGMENT_SIZE).map(|c| c.to_vec()).collect();
    let handles: Vec<_> = chunks.into_iter().map(|chunk| {
        tokio::spawn(async move { compress_async(&chunk, level).await })
    }).collect();
    
    let mut output = BytesMut::new();
    output.extend_from_slice(&(handles.len() as u32).to_le_bytes());
    
    for handle in handles {
        let compressed = handle.await.map_err(|_| CompressError::InvalidData)??;
        output.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        output.extend_from_slice(&compressed);
    }
    Ok(output.freeze())
}

/// Decompress segmented data
pub async fn decompress_segmented(data: &[u8]) -> Result<Bytes, CompressError> {
    if data.len() < 4 { return Err(CompressError::InvalidData); }
    
    let segment_count = u32::from_le_bytes(data[..4].try_into().unwrap()) as usize;
    let mut offset = 4;
    let mut segments = Vec::with_capacity(segment_count);
    
    for _ in 0..segment_count {
        if offset + 4 > data.len() { return Err(CompressError::InvalidData); }
        let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if offset + len > data.len() { return Err(CompressError::InvalidData); }
        segments.push(data[offset..offset + len].to_vec());
        offset += len;
    }
    
    let handles: Vec<_> = segments.into_iter().map(|chunk| {
        tokio::spawn(async move { decompress_async(&chunk).await })
    }).collect();
    
    let mut output = BytesMut::new();
    for handle in handles {
        output.extend_from_slice(&handle.await.map_err(|_| CompressError::InvalidData)??);
    }
    Ok(output.freeze())
}

/// Sync compress (for small data)
pub fn compress_sync(data: &[u8], level: i32) -> Result<Bytes, CompressError> {
    if data.len() < 64 { return Ok(Bytes::copy_from_slice(data)); }
    zstd::encode_all(data, level).map(Bytes::from).map_err(|e| CompressError::Compress(io::Error::other(e)))
}

pub fn decompress_sync(data: &[u8]) -> Result<Bytes, CompressError> {
    zstd::decode_all(data).map(Bytes::from).map_err(|e| CompressError::Compress(io::Error::other(e)))
}

/// Sync decompress segmented data (Send-safe, no async BufReader)
pub fn decompress_segmented_sync(data: &[u8]) -> Result<Bytes, CompressError> {
    if data.len() < 4 { return Err(CompressError::InvalidData); }
    
    let segment_count = u32::from_le_bytes(data[..4].try_into().unwrap()) as usize;
    
    // If segment_count is 0 or unreasonably large, try direct decompression
    // This handles data that was compressed with compress_async (non-segmented)
    if segment_count == 0 || segment_count > 100_000 {
        return decompress_sync(data);
    }
    
    let mut offset = 4;
    let mut output = BytesMut::new();
    
    for _ in 0..segment_count {
        if offset + 4 > data.len() { return Err(CompressError::InvalidData); }
        let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if offset + len > data.len() { return Err(CompressError::InvalidData); }
        let decompressed = decompress_sync(&data[offset..offset + len])?;
        output.extend_from_slice(&decompressed);
        offset += len;
    }
    
    Ok(output.freeze())
}

/// LZ4 for speed-critical paths
pub fn lz4_compress(data: &[u8]) -> Bytes { Bytes::from(lz4_flex::compress_prepend_size(data)) }

pub fn lz4_decompress(data: &[u8]) -> Result<Bytes, CompressError> {
    lz4_flex::decompress_size_prepended(data).map(Bytes::from).map_err(|_| CompressError::InvalidData)
}

/// Snap compression (learned from Kytan) - very fast
pub fn snap_compress(data: &[u8]) -> Bytes {
    let mut encoder = snap::raw::Encoder::new();
    let compressed = encoder.compress_vec(data).unwrap_or_else(|_| data.to_vec());
    // Prepend original size for decompression
    let mut output = Vec::with_capacity(4 + compressed.len());
    output.extend_from_slice(&(data.len() as u32).to_le_bytes());
    output.extend_from_slice(&compressed);
    Bytes::from(output)
}

pub fn snap_decompress(data: &[u8]) -> Result<Bytes, CompressError> {
    if data.len() < 4 {
        return Err(CompressError::InvalidData);
    }
    let original_size = u32::from_le_bytes(data[..4].try_into().unwrap()) as usize;
    let mut decoder = snap::raw::Decoder::new();
    let mut output = vec![0u8; original_size];
    decoder.decompress(&data[4..], &mut output).map_err(|_| CompressError::InvalidData)?;
    Ok(Bytes::from(output))
}

/// Select optimal compression algorithm based on data characteristics
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
pub fn compress_auto(data: &[u8], prefer_speed: bool) -> (Bytes, Algorithm) {
    let algo = select_algorithm(data, prefer_speed);
    let compressed = match algo {
        Algorithm::Zstd => compress_sync(data, 3).unwrap_or_else(|_| Bytes::copy_from_slice(data)),
        Algorithm::Lz4 => lz4_compress(data),
        Algorithm::None => Bytes::copy_from_slice(data),
    };
    (compressed, algo)
}

/// Decompress with algorithm hint
pub fn decompress_auto(data: &[u8], algo: Algorithm) -> Result<Bytes, CompressError> {
    match algo {
        Algorithm::Zstd => decompress_sync(data),
        Algorithm::Lz4 => lz4_decompress(data),
        Algorithm::None => Ok(Bytes::copy_from_slice(data)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_compress_decompress() {
        let data = vec![0u8; 10000];
        let compressed = compress_async(&data, 3).await.unwrap();
        let decompressed = decompress_async(&compressed).await.unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[tokio::test]
    async fn test_segmented_compress() {
        let data: Vec<u8> = (0..200_000).map(|i| (i % 256) as u8).collect();
        let compressed = compress_segmented(&data, 3).await.unwrap();
        let decompressed = decompress_segmented(&compressed).await.unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[tokio::test]
    async fn test_segmented_large() {
        let data = vec![42u8; 5_000_000];
        let compressed = compress_segmented(&data, 10).await.unwrap();
        assert!(compressed.len() < data.len() / 100);
        let decompressed = decompress_segmented(&compressed).await.unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[test]
    fn test_lz4() {
        let data = b"hello world repeated ".repeat(100);
        let compressed = lz4_compress(&data);
        let decompressed = lz4_decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_ref());
    }

    #[test]
    fn test_lz4_speed() {
        let data = vec![42u8; 1_000_000];
        let start = std::time::Instant::now();
        let compressed = lz4_compress(&data);
        let compress_time = start.elapsed();
        assert!(compress_time.as_millis() < 500, "LZ4 compress too slow: {:?}", compress_time);
        
        let start = std::time::Instant::now();
        let decompressed = lz4_decompress(&compressed).unwrap();
        let decompress_time = start.elapsed();
        assert!(decompress_time.as_millis() < 500, "LZ4 decompress too slow: {:?}", decompress_time);
        assert_eq!(data, decompressed.as_ref());
    }

    #[tokio::test]
    async fn test_compression_ratio() {
        let data = vec![42u8; 100_000];
        let compressed = compress_async(&data, 19).await.unwrap();
        assert!(compressed.len() < data.len() / 10);
    }

    #[tokio::test]
    async fn test_compression_levels() {
        let data = vec![42u8; 50_000];
        let c1 = compress_async(&data, 1).await.unwrap();
        let c10 = compress_async(&data, 10).await.unwrap();
        let c22 = compress_async(&data, 22).await.unwrap();
        
        assert!(c22.len() <= c10.len());
        assert!(c10.len() <= c1.len());
        
        assert_eq!(data, decompress_async(&c1).await.unwrap().as_ref());
        assert_eq!(data, decompress_async(&c10).await.unwrap().as_ref());
        assert_eq!(data, decompress_async(&c22).await.unwrap().as_ref());
    }

    #[tokio::test]
    async fn test_empty_data() {
        let data = vec![];
        let compressed = compress_async(&data, 3).await.unwrap();
        assert_eq!(data, compressed.as_ref());
    }

    #[tokio::test]
    async fn test_small_data_bypass() {
        let data = vec![1, 2, 3];
        let compressed = compress_async(&data, 3).await.unwrap();
        assert_eq!(data, compressed.as_ref());
    }

    #[test]
    fn test_sync_roundtrip() {
        let data: Vec<u8> = (0..50_000).map(|i| (i % 256) as u8).collect();
        let compressed = compress_sync(&data, 3).unwrap();
        let decompressed = decompress_sync(&compressed).unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[tokio::test]
    async fn test_binary_data() {
        let data: Vec<u8> = (0..100_000).map(|i| ((i * 7919) % 256) as u8).collect();
        let compressed = compress_segmented(&data, 5).await.unwrap();
        let decompressed = decompress_segmented(&compressed).await.unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[tokio::test]
    async fn test_text_data() {
        let text = "Lorem ipsum dolor sit amet ".repeat(10_000);
        let data = text.as_bytes();
        let compressed = compress_async(data, 10).await.unwrap();
        assert!(compressed.len() < data.len() / 20);
        let decompressed = decompress_async(&compressed).await.unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[tokio::test]
    async fn test_segment_boundary() {
        let data = vec![0xFFu8; SEGMENT_SIZE + 1];
        let compressed = compress_async(&data, 3).await.unwrap();
        let decompressed = decompress_async(&compressed).await.unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[tokio::test]
    async fn test_exact_segment_size() {
        let data = vec![42u8; SEGMENT_SIZE];
        let compressed = compress_async(&data, 3).await.unwrap();
        let decompressed = decompress_async(&compressed).await.unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[tokio::test]
    async fn test_multiple_segments() {
        let data = vec![0xABu8; SEGMENT_SIZE * 2];
        let compressed = compress_segmented(&data, 3).await.unwrap();
        let decompressed = decompress_segmented(&compressed).await.unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[tokio::test]
    async fn test_invalid_segment_header() {
        let invalid = vec![0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00];
        assert!(decompress_segmented(&invalid).await.is_err());
    }

    #[test]
    fn test_snap_roundtrip() {
        let data = b"hello world repeated ".repeat(100);
        let compressed = snap_compress(&data);
        let decompressed = snap_decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_ref());
    }

    #[test]
    fn test_auto_compression() {
        let data = vec![42u8; 10_000];
        
        // Speed preference
        let (compressed, algo) = compress_auto(&data, true);
        assert_eq!(algo, Algorithm::Lz4);
        let decompressed = decompress_auto(&compressed, algo).unwrap();
        assert_eq!(data, decompressed.as_ref());
        
        // Ratio preference
        let (compressed, algo) = compress_auto(&data, false);
        assert_eq!(algo, Algorithm::Zstd);
        let decompressed = decompress_auto(&compressed, algo).unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[test]
    fn test_small_data_no_compression() {
        let data = vec![1, 2, 3];
        let (compressed, algo) = compress_auto(&data, false);
        assert_eq!(algo, Algorithm::None);
        assert_eq!(data, compressed.as_ref());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 36: Compression Round-Trip
        #[test]
        fn prop_compression_roundtrip(
            data in prop::collection::vec(any::<u8>(), 0..50000),
        ) {
            // Zstd sync
            if data.len() >= 64 {
                let compressed = compress_sync(&data, 3).unwrap();
                let decompressed = decompress_sync(&compressed).unwrap();
                prop_assert_eq!(&data, decompressed.as_ref());
            }
        }

        /// Property: LZ4 Round-Trip
        #[test]
        fn prop_lz4_roundtrip(
            data in prop::collection::vec(any::<u8>(), 1..50000),
        ) {
            let compressed = lz4_compress(&data);
            let decompressed = lz4_decompress(&compressed).unwrap();
            prop_assert_eq!(&data, decompressed.as_ref());
        }

        /// Property: Snap Round-Trip
        #[test]
        fn prop_snap_roundtrip(
            data in prop::collection::vec(any::<u8>(), 1..50000),
        ) {
            let compressed = snap_compress(&data);
            let decompressed = snap_decompress(&compressed).unwrap();
            prop_assert_eq!(&data, decompressed.as_ref());
        }

        /// Property: Auto compression Round-Trip
        #[test]
        fn prop_auto_compression_roundtrip(
            data in prop::collection::vec(any::<u8>(), 0..10000),
            prefer_speed in any::<bool>(),
        ) {
            let (compressed, algo) = compress_auto(&data, prefer_speed);
            let decompressed = decompress_auto(&compressed, algo).unwrap();
            prop_assert_eq!(&data, decompressed.as_ref());
        }

        /// Property 35: Compression Reduces Size (for compressible data)
        #[test]
        fn prop_compression_reduces_size(
            repeat_byte in any::<u8>(),
            size in 1000usize..50000,
        ) {
            let data = vec![repeat_byte; size];
            let compressed = compress_sync(&data, 10).unwrap();
            // Highly repetitive data should compress well
            prop_assert!(compressed.len() <= data.len());
        }
    }
}
