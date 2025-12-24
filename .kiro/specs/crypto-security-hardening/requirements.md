# Requirements Document

## Introduction

This specification defines security hardening enhancements for the Vortex Image cryptography module. The enhancements address critical security concerns including unsafe code in Dilithium signing, machine-bound token encryption weaknesses, keypair exposure to frontend, missing key rotation mechanisms, token version handling, missing AAD in AEAD, and Clone on secret types.

## Glossary

- **Crypto_Module**: The Rust cryptography module (`crypto.rs`) providing hybrid post-quantum encryption
- **Keychain_Service**: OS-level secure storage (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- **Keypair_Store**: In-memory store for hybrid keypairs with opaque handles
- **Token_Encryptor**: Component responsible for encrypting/decrypting machine-bound tokens
- **Key_Rotator**: Component managing keypair rotation and re-encryption
- **Frontend_Crypto**: TypeScript composable (`useCrypto.ts`) for frontend crypto operations
- **AAD**: Associated Authenticated Data bound to AEAD ciphertext
- **Opaque_Handle**: A numeric identifier that references a keypair without exposing key bytes
- **SecretBytes**: Zeroizing wrapper for secret key material
- **Compress_Module**: The Rust compression module (`compress.rs`) providing multi-algorithm compression

## Requirements

### Requirement 1: Pin PQ Crypto Library Versions

**User Story:** As a developer, I want cryptographic library versions pinned in Cargo.toml, so that memory layout assumptions in unsafe code remain valid across builds.

#### Acceptance Criteria

1. THE Crypto_Module SHALL pin `pqc_dilithium` to exact version `=0.2.0` in Cargo.toml
2. THE Crypto_Module SHALL pin `pqc_kyber` to exact version `=0.7.1` in Cargo.toml
3. WHEN the pinned library versions are updated, THE Crypto_Module SHALL include integration tests that verify signing/verification compatibility
4. THE Crypto_Module SHALL document the memory layout assumptions in code comments

### Requirement 2: Remove Unsafe Dilithium Signing Code

**User Story:** As a security engineer, I want the unsafe transmute code removed from Dilithium signing, so that the module doesn't rely on internal memory layout assumptions.

#### Acceptance Criteria

1. THE Crypto_Module SHALL replace `std::mem::transmute` with safe byte-based keypair reconstruction
2. THE Crypto_Module SHALL use the `sign_dilithium_safe` method that reconstructs keypairs from stored bytes
3. WHEN signing data, THE Crypto_Module SHALL NOT use `std::ptr::read` or `std::mem::transmute` on keypair structures
4. THE Crypto_Module SHALL validate keypair byte lengths before reconstruction

### Requirement 3: Implement OS Keychain Integration

**User Story:** As a user, I want my encryption tokens stored in the OS keychain, so that they are protected by platform-level security rather than predictable machine identifiers.

#### Acceptance Criteria

1. THE Keychain_Service SHALL store tokens using macOS Keychain on macOS platforms
2. THE Keychain_Service SHALL store tokens using Windows Credential Manager on Windows platforms
3. THE Keychain_Service SHALL store tokens using Secret Service API on Linux platforms
4. WHEN the OS keychain is unavailable, THE Keychain_Service SHALL log a warning and fall back to salted machine-key encryption
5. THE Keychain_Service SHALL use the service identifier "com.vortex.image.crypto" for all keychain entries
6. IF keychain storage fails, THEN THE Keychain_Service SHALL return a descriptive error

### Requirement 4: Remove Keypair Bytes from Frontend

**User Story:** As a security engineer, I want keypair secret bytes kept only in the Rust backend, so that the attack surface is minimized.

#### Acceptance Criteria

1. THE Keypair_Store SHALL assign opaque numeric handles to generated keypairs
2. THE Frontend_Crypto SHALL receive only the opaque handle and public bundle, not keypair bytes
3. WHEN the frontend requests signing or decryption, THE Frontend_Crypto SHALL pass the opaque handle to the backend
4. THE Crypto_Module SHALL look up keypairs by handle in the Keypair_Store
5. THE Keypair_Store SHALL remove keypairs when the handle is released
6. THE Frontend_Crypto SHALL NOT store `keypair_bytes` in Vue reactive state

### Requirement 5: Implement Key Rotation Mechanism

**User Story:** As a user, I want to rotate my encryption keys periodically, so that compromised keys have limited impact.

#### Acceptance Criteria

1. THE Key_Rotator SHALL generate a new keypair while preserving the old keypair for decryption
2. THE Key_Rotator SHALL maintain a history of rotated keypairs per handle
3. WHEN decrypting data, THE Crypto_Module SHALL attempt decryption with current and all rotated keypairs
4. THE Key_Rotator SHALL provide a `rotate_keypair` command accessible from the frontend
5. THE Key_Rotator SHALL include key creation timestamp and rotation count in keypair metadata
6. THE Key_Rotator SHALL support re-encrypting data with the new key

### Requirement 6: Update Token Format to v4

**User Story:** As a developer, I want tokens upgraded to v4 format with proper AAD, so that token security is improved.

#### Acceptance Criteria

1. THE Token_Encryptor SHALL use version byte `0x04` for new tokens
2. THE Token_Encryptor SHALL include random salt in v4 tokens
3. THE Token_Encryptor SHALL bind context (service identifier, timestamp) as AAD in AEAD encryption
4. WHEN decrypting v2 tokens, THE Token_Encryptor SHALL automatically re-encrypt as v4 and return the upgraded token
5. WHEN decrypting v3 tokens, THE Token_Encryptor SHALL automatically re-encrypt as v4 and return the upgraded token
6. THE Token_Encryptor SHALL reject tokens with unsupported version bytes

### Requirement 7: Add AAD to AEAD Encryption

**User Story:** As a security engineer, I want associated data bound to ciphertext, so that ciphertext substitution attacks are prevented.

#### Acceptance Criteria

1. THE Crypto_Module SHALL accept optional AAD parameter in encryption functions
2. WHEN AAD is provided, THE Crypto_Module SHALL use `Payload { msg, aad }` in ChaCha20-Poly1305 encryption
3. THE Crypto_Module SHALL store AAD hash in EncryptedPayload for verification
4. WHEN decrypting, THE Crypto_Module SHALL verify AAD matches the stored hash
5. IF AAD verification fails, THEN THE Crypto_Module SHALL return a decryption error

### Requirement 8: Remove Clone from SecretBytes

**User Story:** As a security engineer, I want Clone removed from secret types, so that accidental duplication of secret material is prevented.

#### Acceptance Criteria

1. THE SecretBytes type SHALL NOT derive Clone
2. THE SecretKey32 type SHALL NOT derive Clone
3. THE SecretBytes type SHALL provide an explicit `clone_secret()` method for auditable cloning
4. THE HybridKeypair type SHALL NOT derive Clone
5. WHEN secret material must be duplicated, THE Crypto_Module SHALL use the explicit clone method

### Requirement 9: Add Session Timeout to Frontend

**User Story:** As a user, I want my unlocked keypair to automatically lock after inactivity, so that my keys are protected if I step away.

#### Acceptance Criteria

1. THE Frontend_Crypto SHALL track the last activity timestamp
2. WHEN the inactivity timeout (default 15 minutes) is exceeded, THE Frontend_Crypto SHALL automatically lock the keypair
3. THE Frontend_Crypto SHALL reset the activity timer on crypto operations
4. THE Frontend_Crypto SHALL provide a configurable timeout duration
5. WHEN the keypair is locked, THE Frontend_Crypto SHALL clear the handle from memory

### Requirement 10: Improve Machine Key Fallback Security

**User Story:** As a developer, I want the machine key fallback to log warnings, so that weak security configurations are visible.

#### Acceptance Criteria

1. WHEN falling back to weak machine identifiers, THE Token_Encryptor SHALL log a warning message
2. THE Token_Encryptor SHALL include additional entropy sources in the fallback (process ID, boot time if available)
3. THE Token_Encryptor SHALL document the security implications of the fallback in code comments
4. THE Token_Encryptor SHALL prefer keychain storage over machine-key encryption when available

### Requirement 11: Validate Keypair Handle from Frontend

**User Story:** As a developer, I want to validate that a stored keypair handle is still valid in the backend, so that stale handles are detected before crypto operations fail.

#### Acceptance Criteria

1. THE Crypto_Module SHALL provide a `validate_keypair_handle` Tauri command
2. WHEN a valid handle is provided, THE Crypto_Module SHALL return true
3. WHEN an invalid or expired handle is provided, THE Crypto_Module SHALL return false
4. THE Frontend_Crypto SHALL call validate_keypair_handle when restoring a session

### Requirement 12: Compression Result Transparency

**User Story:** As a developer, I want to know whether compression was actually applied, so that I can handle uncompressed data correctly.

#### Acceptance Criteria

1. WHEN data is too small to compress, THE Compress_Module SHALL return a result indicating compression was skipped
2. THE CompressionResult type SHALL include a `was_compressed` boolean field
3. WHEN an invalid algorithm name is provided to strict functions, THE Compress_Module SHALL return an error
4. THE Compress_Module SHALL NOT silently default to Zstd for invalid algorithm names in strict mode

### Requirement 13: Document Unsafe Code Safety Invariants

**User Story:** As a security engineer, I want all unsafe code blocks documented with safety invariants, so that future maintainers understand the requirements.

#### Acceptance Criteria

1. THE Crypto_Module SHALL document all unsafe blocks with SAFETY comments
2. THE SAFETY comments SHALL list all invariants that must hold for the unsafe code to be sound
3. THE Crypto_Module SHALL include compile-time assertions where possible to verify invariants
4. WHEN library versions are pinned for unsafe code, THE Cargo.toml SHALL document why
