//! Compression Module Tests
//!
//! Organized by functionality:
//! - `algorithm_tests` - Individual algorithm roundtrips and edge cases
//! - `roundtrip_tests` - Full compression/decompression cycles
//! - `file_tests` - File-based compression with checksums

pub mod algorithm_tests;
pub mod roundtrip_tests;
pub mod file_tests;
