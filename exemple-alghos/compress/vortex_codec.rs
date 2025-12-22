//! VortexCodec - Revolutionary Nonary (Base-9) Compression System
//! 
//! Uses symbols {1,2,3,4,5,6,7,8,9} instead of binary {0,1}
//! Combined with frequency-based encoding for compression
//! Optimized for P2P transfer over Iroh/Tor

use bytes::{Bytes, BytesMut};

/// Nonary symbol: values 1-9 (not 0!)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Nonary {
    One = 1, Two = 2, Three = 3, Four = 4, Five = 5,
    Six = 6, Seven = 7, Eight = 8, Nine = 9,
}

impl Nonary {
    #[inline]
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::One), 2 => Some(Self::Two), 3 => Some(Self::Three),
            4 => Some(Self::Four), 5 => Some(Self::Five), 6 => Some(Self::Six),
            7 => Some(Self::Seven), 8 => Some(Self::Eight), 9 => Some(Self::Nine),
            _ => None,
        }
    }
    
    #[inline]
    pub fn value(self) -> u8 { self as u8 }
}

/// Symbol frequency table for encoding
#[derive(Clone, Debug)]
pub struct SymbolTable {
    /// Byte -> frequency count
    frequencies: [u32; 256],
    /// Sorted symbols by frequency (most frequent first)
    sorted: Vec<u8>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self { 
            frequencies: [1; 256], 
            sorted: (0..=255u8).collect(),
        }
    }
}

impl SymbolTable {
    pub fn from_data(data: &[u8]) -> Self {
        let mut table = Self::default();
        for &byte in data {
            table.frequencies[byte as usize] = table.frequencies[byte as usize].saturating_add(1);
        }
        // Sort by frequency (descending)
        table.sorted.sort_by(|&a, &b| {
            table.frequencies[b as usize].cmp(&table.frequencies[a as usize])
        });
        table
    }
    
    /// Get rank of symbol (0 = most frequent)
    pub fn get_rank(&self, symbol: u8) -> usize {
        self.sorted.iter().position(|&s| s == symbol).unwrap_or(255)
    }
    
    /// Get symbol by rank
    pub fn get_symbol(&self, rank: usize) -> u8 {
        self.sorted.get(rank).copied().unwrap_or(0)
    }
    
    /// Serialize table (compact format)
    pub fn to_bytes(&self) -> Vec<u8> {
        // Store only the sorted order (256 bytes)
        self.sorted.clone()
    }
    
    /// Deserialize table
    pub fn from_bytes(data: &[u8]) -> Option<(Self, usize)> {
        if data.len() < 256 { return None; }
        let mut table = Self::default();
        table.sorted = data[..256].to_vec();
        Some((table, 256))
    }
}

/// Encode a value in nonary (base-9) with 1-indexed digits
/// Returns digits in little-endian order
fn encode_nonary(mut value: usize) -> Vec<Nonary> {
    if value == 0 {
        return vec![Nonary::One];
    }
    let mut digits = Vec::new();
    while value > 0 {
        let digit = (value % 9) as u8 + 1; // 1-indexed
        digits.push(Nonary::from_u8(digit).unwrap());
        value /= 9;
    }
    digits
}

/// Decode nonary digits to value
fn decode_nonary(digits: &[Nonary]) -> usize {
    let mut value = 0usize;
    let mut multiplier = 1usize;
    for &digit in digits {
        value += ((digit.value() - 1) as usize) * multiplier;
        multiplier *= 9;
    }
    value
}

/// Pack nonary symbols into bytes efficiently
/// Uses variable-length encoding: 2 nonary digits per byte (9^2 = 81 < 256)
pub fn pack_nonary(symbols: &[Nonary]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + symbols.len() / 2 + 1);
    out.extend_from_slice(&(symbols.len() as u32).to_le_bytes());
    
    for chunk in symbols.chunks(2) {
        let byte = match chunk {
            [a, b] => (a.value() - 1) * 9 + (b.value() - 1),
            [a] => (a.value() - 1) * 9 + 8, // Pad with 9
            _ => 0,
        };
        out.push(byte);
    }
    out
}

/// Unpack bytes to nonary symbols
pub fn unpack_nonary(data: &[u8]) -> Option<Vec<Nonary>> {
    if data.len() < 4 { return None; }
    let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let mut symbols = Vec::with_capacity(len);
    
    for &byte in &data[4..] {
        let a = (byte / 9) + 1;
        let b = (byte % 9) + 1;
        if let Some(s) = Nonary::from_u8(a) { 
            if symbols.len() < len { symbols.push(s); }
        }
        if let Some(s) = Nonary::from_u8(b) { 
            if symbols.len() < len { symbols.push(s); }
        }
    }
    symbols.truncate(len);
    Some(symbols)
}

/// VortexCodec: Main compression interface
/// 
/// Encoding strategy:
/// 1. Build frequency table from data
/// 2. Map each byte to its rank (0-255)
/// 3. Encode ranks in nonary (base-9)
/// 4. Pack nonary stream efficiently
pub struct VortexCodec;

impl VortexCodec {
    /// Compress data using nonary rank encoding
    pub fn compress(data: &[u8]) -> Bytes {
        if data.len() < 16 {
            // Too small, return with marker
            let mut out = vec![0u8]; // Marker: uncompressed
            out.extend_from_slice(data);
            return Bytes::from(out);
        }
        
        let table = SymbolTable::from_data(data);
        let mut nonary_stream = Vec::new();
        
        // Encode each byte as its rank in nonary
        for &byte in data {
            let rank = table.get_rank(byte);
            let digits = encode_nonary(rank);
            // Prefix with digit count (1-4 typically)
            nonary_stream.push(Nonary::from_u8((digits.len() as u8).min(9)).unwrap_or(Nonary::One));
            nonary_stream.extend(digits);
        }
        
        let packed = pack_nonary(&nonary_stream);
        let table_bytes = table.to_bytes();
        
        // Check if compression is beneficial
        let total_size = 1 + 4 + table_bytes.len() + packed.len();
        if total_size >= data.len() {
            // No benefit, store raw
            let mut out = vec![0u8];
            out.extend_from_slice(data);
            return Bytes::from(out);
        }
        
        // Build output: marker + original_size + table + packed data
        let mut out = Vec::with_capacity(total_size);
        out.push(1u8); // Marker: compressed
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&table_bytes);
        out.extend_from_slice(&packed);
        
        Bytes::from(out)
    }
    
    /// Decompress data
    pub fn decompress(data: &[u8]) -> Option<Bytes> {
        if data.is_empty() { return None; }
        
        if data[0] == 0 {
            // Uncompressed
            return Some(Bytes::copy_from_slice(&data[1..]));
        }
        
        if data.len() < 5 { return None; }
        
        let original_size = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
        let (table, table_size) = SymbolTable::from_bytes(&data[5..])?;
        let packed_start = 5 + table_size;
        
        if packed_start >= data.len() { return None; }
        
        let nonary_stream = unpack_nonary(&data[packed_start..])?;
        
        let mut output = Vec::with_capacity(original_size);
        let mut i = 0;
        
        while output.len() < original_size && i < nonary_stream.len() {
            let digit_count = (nonary_stream[i].value() as usize).min(9);
            i += 1;
            
            if i + digit_count > nonary_stream.len() { break; }
            
            let rank = decode_nonary(&nonary_stream[i..i + digit_count]);
            i += digit_count;
            
            let symbol = table.get_symbol(rank);
            output.push(symbol);
        }
        
        if output.len() != original_size {
            return None;
        }
        
        Some(Bytes::from(output))
    }
}

/// Hybrid compression: Vortex + Zstd for maximum compression
pub fn vortex_compress(data: &[u8]) -> Bytes {
    if data.len() < 64 {
        return VortexCodec::compress(data);
    }
    
    // Try Zstd first (usually better for general data)
    if let Ok(zstd_compressed) = zstd::encode_all(data, 3) {
        if zstd_compressed.len() < data.len() * 9 / 10 {
            let mut out = vec![2u8]; // Marker: zstd only
            out.extend_from_slice(&zstd_compressed);
            return Bytes::from(out);
        }
    }
    
    // Try Vortex for highly structured data
    let vortex_compressed = VortexCodec::compress(data);
    if vortex_compressed.len() < data.len() {
        return vortex_compressed;
    }
    
    // No compression beneficial
    let mut out = vec![0u8];
    out.extend_from_slice(data);
    Bytes::from(out)
}

/// Hybrid decompression
pub fn vortex_decompress(data: &[u8]) -> Option<Bytes> {
    if data.is_empty() { return None; }
    
    match data[0] {
        0 => Some(Bytes::copy_from_slice(&data[1..])), // Raw
        1 => VortexCodec::decompress(data), // Vortex
        2 => zstd::decode_all(&data[1..]).ok().map(Bytes::from), // Zstd
        3 => {
            // Segmented (handled separately)
            None
        }
        _ => None,
    }
}

/// Segmented Vortex compression for large data (P2P optimized)
pub async fn vortex_compress_segmented(data: &[u8], segment_size: usize) -> Bytes {
    if data.len() <= segment_size {
        return vortex_compress(data);
    }
    
    let chunks: Vec<_> = data.chunks(segment_size).map(|c| c.to_vec()).collect();
    let handles: Vec<_> = chunks.into_iter().map(|chunk| {
        tokio::spawn(async move { vortex_compress(&chunk) })
    }).collect();
    
    let mut output = BytesMut::new();
    output.extend_from_slice(&[3u8]); // Marker: segmented
    output.extend_from_slice(&(handles.len() as u32).to_le_bytes());
    
    for handle in handles {
        if let Ok(compressed) = handle.await {
            output.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
            output.extend_from_slice(&compressed);
        }
    }
    
    output.freeze()
}

/// Decompress segmented data
pub async fn vortex_decompress_segmented(data: &[u8]) -> Option<Bytes> {
    if data.is_empty() { return None; }
    
    if data[0] != 3 {
        return vortex_decompress(data);
    }
    
    if data.len() < 5 { return None; }
    let segment_count = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
    let mut offset = 5;
    let mut segments = Vec::with_capacity(segment_count);
    
    for _ in 0..segment_count {
        if offset + 4 > data.len() { return None; }
        let len = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
        ]) as usize;
        offset += 4;
        if offset + len > data.len() { return None; }
        segments.push(data[offset..offset + len].to_vec());
        offset += len;
    }
    
    let handles: Vec<_> = segments.into_iter().map(|chunk| {
        tokio::spawn(async move { vortex_decompress(&chunk) })
    }).collect();
    
    let mut output = BytesMut::new();
    for handle in handles {
        if let Ok(Some(decompressed)) = handle.await {
            output.extend_from_slice(&decompressed);
        } else {
            return None;
        }
    }
    
    Some(output.freeze())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonary_symbols() {
        for i in 1..=9 {
            let n = Nonary::from_u8(i).unwrap();
            assert_eq!(n.value(), i);
        }
        assert!(Nonary::from_u8(0).is_none());
        assert!(Nonary::from_u8(10).is_none());
    }

    #[test]
    fn test_nonary_encoding() {
        // Test encode/decode roundtrip
        for value in [0, 1, 8, 9, 80, 81, 100, 255, 1000] {
            let encoded = encode_nonary(value);
            let decoded = decode_nonary(&encoded);
            assert_eq!(value, decoded, "Failed for value {}", value);
        }
    }

    #[test]
    fn test_pack_unpack_nonary() {
        let symbols: Vec<Nonary> = (1..=9).cycle().take(100)
            .filter_map(Nonary::from_u8).collect();
        let packed = pack_nonary(&symbols);
        let unpacked = unpack_nonary(&packed).unwrap();
        assert_eq!(symbols, unpacked);
    }

    #[test]
    fn test_symbol_table() {
        let data = b"hello world";
        let table = SymbolTable::from_data(data);
        
        // 'l' appears 3 times, should be ranked high
        let l_rank = table.get_rank(b'l');
        let x_rank = table.get_rank(b'x'); // doesn't appear
        assert!(l_rank < x_rank);
        
        // Roundtrip
        let bytes = table.to_bytes();
        let (restored, _) = SymbolTable::from_bytes(&bytes).unwrap();
        assert_eq!(table.sorted, restored.sorted);
    }

    #[test]
    fn test_vortex_compress_decompress() {
        let data = b"hello world! this is a test of vortex compression.";
        let compressed = VortexCodec::compress(data);
        let decompressed = VortexCodec::decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_ref());
    }

    #[test]
    fn test_vortex_small_data() {
        let data = b"tiny";
        let compressed = VortexCodec::compress(data);
        let decompressed = VortexCodec::decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_ref());
    }

    #[test]
    fn test_vortex_repetitive_data() {
        let data = vec![42u8; 10000];
        let compressed = vortex_compress(&data);
        let decompressed = vortex_decompress(&compressed).unwrap();
        assert_eq!(data, decompressed.as_ref());
        // Highly repetitive data should compress well
        assert!(compressed.len() < data.len() / 10);
    }

    #[test]
    fn test_hybrid_compression() {
        let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let compressed = vortex_compress(&data);
        let decompressed = vortex_decompress(&compressed).unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[tokio::test]
    async fn test_segmented_compression() {
        let data = vec![0xABu8; 200_000];
        let compressed = vortex_compress_segmented(&data, 64 * 1024).await;
        let decompressed = vortex_decompress_segmented(&compressed).await.unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[test]
    fn test_binary_data() {
        let data: Vec<u8> = (0..=255).collect();
        let compressed = VortexCodec::compress(&data);
        let decompressed = VortexCodec::decompress(&compressed).unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[test]
    fn test_empty_data() {
        let data = b"";
        let compressed = VortexCodec::compress(data);
        let decompressed = VortexCodec::decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_ref());
    }

    #[test]
    fn test_random_data() {
        // Pseudo-random data (hard to compress)
        let data: Vec<u8> = (0..1000).map(|i| ((i * 7919 + 104729) % 256) as u8).collect();
        let compressed = vortex_compress(&data);
        let decompressed = vortex_decompress(&compressed).unwrap();
        assert_eq!(data, decompressed.as_ref());
    }
}
