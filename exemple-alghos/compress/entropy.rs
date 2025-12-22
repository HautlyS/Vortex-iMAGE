//! Advanced Entropy Analysis and Adaptive Radix Compression
//! 
//! Analyzes data entropy to select optimal encoding strategy:
//! - Low entropy: Run-length + Nonary
//! - Medium entropy: Arithmetic coding
//! - High entropy: Direct pass-through (incompressible)

use bytes::Bytes;

/// Entropy analysis result
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EntropyClass {
    /// Very low entropy (< 2 bits/byte) - highly compressible
    VeryLow,
    /// Low entropy (2-4 bits/byte) - good compression
    Low,
    /// Medium entropy (4-6 bits/byte) - moderate compression
    Medium,
    /// High entropy (6-7.5 bits/byte) - poor compression
    High,
    /// Near-random (> 7.5 bits/byte) - incompressible
    Random,
}

/// Calculate Shannon entropy of data (bits per byte)
pub fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
    
    let mut freq = [0u64; 256];
    for &byte in data {
        freq[byte as usize] += 1;
    }
    
    let len = data.len() as f64;
    let mut entropy = 0.0;
    
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    
    entropy
}

/// Classify data by entropy
pub fn classify_entropy(data: &[u8]) -> EntropyClass {
    let entropy = calculate_entropy(data);
    match entropy {
        e if e < 2.0 => EntropyClass::VeryLow,
        e if e < 4.0 => EntropyClass::Low,
        e if e < 6.0 => EntropyClass::Medium,
        e if e < 7.5 => EntropyClass::High,
        _ => EntropyClass::Random,
    }
}

/// Run-Length Encoding optimized for low-entropy data
/// Uses nonary symbols for run lengths
pub fn rle_encode(data: &[u8]) -> Vec<u8> {
    if data.is_empty() { return vec![]; }
    
    let mut output = Vec::with_capacity(data.len());
    let mut i = 0;
    
    while i < data.len() {
        let byte = data[i];
        let mut run_len = 1usize;
        
        while i + run_len < data.len() && data[i + run_len] == byte && run_len < 65535 {
            run_len += 1;
        }
        
        if run_len >= 4 {
            // Encode run: marker + byte + length (nonary-packed)
            output.push(0xFF); // Run marker
            output.push(byte);
            // Pack length in nonary-inspired format (base-9 digits)
            let mut len = run_len;
            let mut digits = Vec::new();
            while len > 0 {
                digits.push(((len % 9) + 1) as u8);
                len /= 9;
            }
            output.push(digits.len() as u8);
            output.extend(digits);
        } else {
            // Literal bytes
            for _ in 0..run_len {
                if byte == 0xFF {
                    output.push(0xFF);
                    output.push(0x00); // Escape
                } else {
                    output.push(byte);
                }
            }
        }
        
        i += run_len;
    }
    
    output
}

/// Run-Length Decoding
pub fn rle_decode(data: &[u8]) -> Option<Vec<u8>> {
    let mut output = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        if data[i] == 0xFF {
            if i + 1 >= data.len() { return None; }
            
            if data[i + 1] == 0x00 {
                // Escaped 0xFF
                output.push(0xFF);
                i += 2;
            } else {
                // Run
                if i + 3 >= data.len() { return None; }
                let byte = data[i + 1];
                let digit_count = data[i + 2] as usize;
                if i + 3 + digit_count > data.len() { return None; }
                
                let mut run_len = 0usize;
                let mut multiplier = 1usize;
                for j in 0..digit_count {
                    let digit = (data[i + 3 + j] - 1) as usize;
                    run_len += digit * multiplier;
                    multiplier *= 9;
                }
                
                output.extend(std::iter::repeat(byte).take(run_len));
                i += 3 + digit_count;
            }
        } else {
            output.push(data[i]);
            i += 1;
        }
    }
    
    Some(output)
}

/// Adaptive compression based on entropy analysis
pub fn adaptive_compress(data: &[u8]) -> Bytes {
    if data.len() < 32 {
        let mut out = vec![0u8]; // Marker: raw
        out.extend_from_slice(data);
        return Bytes::from(out);
    }
    
    let entropy_class = classify_entropy(data);
    
    match entropy_class {
        EntropyClass::VeryLow | EntropyClass::Low => {
            // Use RLE + Vortex
            let rle = rle_encode(data);
            let vortex = super::vortex_compress(&rle);
            
            if vortex.len() < data.len() {
                let mut out = vec![1u8]; // Marker: RLE + Vortex
                out.extend_from_slice(&(data.len() as u32).to_le_bytes());
                out.extend_from_slice(&vortex);
                return Bytes::from(out);
            }
        }
        EntropyClass::Medium => {
            // Use Vortex directly
            let vortex = super::vortex_compress(data);
            if vortex.len() < data.len() {
                let mut out = vec![2u8]; // Marker: Vortex
                out.extend_from_slice(&vortex);
                return Bytes::from(out);
            }
        }
        EntropyClass::High => {
            // Try Zstd
            if let Ok(zstd) = zstd::encode_all(data, 3) {
                if zstd.len() < data.len() {
                    let mut out = vec![3u8]; // Marker: Zstd
                    out.extend_from_slice(&zstd);
                    return Bytes::from(out);
                }
            }
        }
        EntropyClass::Random => {
            // Incompressible, store raw
        }
    }
    
    // Fallback: raw
    let mut out = vec![0u8];
    out.extend_from_slice(data);
    Bytes::from(out)
}

/// Adaptive decompression
pub fn adaptive_decompress(data: &[u8]) -> Option<Bytes> {
    if data.is_empty() { return None; }
    
    match data[0] {
        0 => Some(Bytes::copy_from_slice(&data[1..])), // Raw
        1 => {
            // RLE + Vortex
            if data.len() < 5 { return None; }
            let _original_size = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);
            let vortex_data = super::vortex_decompress(&data[5..])?;
            let rle_decoded = rle_decode(&vortex_data)?;
            Some(Bytes::from(rle_decoded))
        }
        2 => {
            // Vortex
            super::vortex_decompress(&data[1..])
        }
        3 => {
            // Zstd
            zstd::decode_all(&data[1..]).ok().map(Bytes::from)
        }
        _ => None,
    }
}

/// Multi-radix encoder: dynamically switches between bases
/// Based on local data patterns
pub struct MultiRadixEncoder {
    buffer: Vec<u8>,
    current_radix: u8,
}

impl MultiRadixEncoder {
    pub fn new() -> Self {
        Self { buffer: Vec::new(), current_radix: 9 }
    }
    
    /// Encode value in current radix
    pub fn encode_value(&mut self, value: u64) {
        let radix = self.current_radix as u64;
        let mut v = value;
        let mut digits = Vec::new();
        
        loop {
            digits.push(((v % radix) + 1) as u8); // 1-indexed
            v /= radix;
            if v == 0 { break; }
        }
        
        // Length prefix (1 byte for small values)
        self.buffer.push(digits.len() as u8);
        self.buffer.extend(digits);
    }
    
    /// Switch radix based on data pattern
    pub fn adapt_radix(&mut self, sample: &[u8]) {
        let entropy = calculate_entropy(sample);
        self.current_radix = match entropy {
            e if e < 2.0 => 3,  // Low entropy: smaller radix
            e if e < 4.0 => 5,
            e if e < 6.0 => 7,
            _ => 9,            // High entropy: larger radix
        };
        self.buffer.push(0x00); // Radix change marker
        self.buffer.push(self.current_radix);
    }
    
    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_calculation() {
        // Uniform data has low entropy
        let uniform = vec![42u8; 1000];
        assert!(calculate_entropy(&uniform) < 0.1);
        
        // Random-ish data has high entropy
        let varied: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        assert!(calculate_entropy(&varied) > 7.0);
    }

    #[test]
    fn test_entropy_classification() {
        let uniform = vec![42u8; 1000];
        assert_eq!(classify_entropy(&uniform), EntropyClass::VeryLow);
        
        let random: Vec<u8> = (0..1000).map(|i| ((i * 7919) % 256) as u8).collect();
        assert!(matches!(classify_entropy(&random), EntropyClass::High | EntropyClass::Random));
    }

    #[test]
    fn test_rle_roundtrip() {
        let data = b"aaaaaabbbbccccccccdddddddddddd";
        let encoded = rle_encode(data);
        let decoded = rle_decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_rle_with_marker_byte() {
        let data = vec![0xFF; 100];
        let encoded = rle_encode(&data);
        let decoded = rle_decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_adaptive_compress_low_entropy() {
        let data = vec![42u8; 10000];
        let compressed = adaptive_compress(&data);
        let decompressed = adaptive_decompress(&compressed).unwrap();
        assert_eq!(data, decompressed.as_ref());
        assert!(compressed.len() < data.len() / 50);
    }

    #[test]
    fn test_adaptive_compress_medium_entropy() {
        let data: Vec<u8> = (0..10000).map(|i| (i % 64) as u8).collect();
        let compressed = adaptive_compress(&data);
        let decompressed = adaptive_decompress(&compressed).unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[test]
    fn test_adaptive_compress_high_entropy() {
        let data: Vec<u8> = (0..10000).map(|i| ((i * 7919) % 256) as u8).collect();
        let compressed = adaptive_compress(&data);
        let decompressed = adaptive_decompress(&compressed).unwrap();
        assert_eq!(data, decompressed.as_ref());
    }

    #[test]
    fn test_multi_radix_encoder() {
        let mut encoder = MultiRadixEncoder::new();
        encoder.encode_value(42);
        encoder.encode_value(1000);
        encoder.adapt_radix(&[0u8; 100]); // Low entropy sample
        encoder.encode_value(42);
        let output = encoder.finish();
        assert!(!output.is_empty());
    }
}
