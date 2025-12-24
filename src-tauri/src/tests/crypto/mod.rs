//! Crypto Module Tests
//!
//! Organized by functionality:
//! - `keypair_tests` - Keypair generation, handles, rotation
//! - `encryption_tests` - Hybrid encryption, AAD, password-based
//! - `signature_tests` - Signing and verification
//! - `token_tests` - Token encryption, versioning, migration
//! - `property_tests` - Property-based tests with proptest

pub mod keypair_tests;
pub mod encryption_tests;
pub mod signature_tests;
pub mod token_tests;
pub mod property_tests;
