/// Magic bytes to identify file format + version (v1).
/// Header: MAGIC(5) | NONCE(12) | CIPHERTEXT + 16 Bytes MAC
pub const MAGIC: [u8; 5] = *b"LOCK1";

/// Lengths for header and AEAD
pub const MAGIC_LEN: usize = 5;
pub const NONCE_LEN: usize = 12; // 96-bit AES-GCM nonce
pub const TAG_LEN: usize = 16; // 128-bit GCM authentication tag
pub const HEADER_LEN: usize = MAGIC_LEN + NONCE_LEN;

/// extension for encrypted files
pub const ENCRYPTED_EXT: &str = ".lock";
