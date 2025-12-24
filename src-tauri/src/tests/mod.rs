//! Test Module Organization
//!
//! This module organizes all tests into logical categories:
//! - `crypto/` - Cryptographic operation tests
//! - `compress/` - Compression algorithm tests  
//! - `integration/` - End-to-end security pipeline tests
//!
//! Run all tests: `cargo test`
//! Run specific module: `cargo test crypto::` or `cargo test compress::`

#[cfg(test)]
pub mod crypto;

#[cfg(test)]
pub mod compress;

#[cfg(test)]
pub mod integration;
