/// Magic bytes to identify file format + version (v1).
/// Header: MAGIC(5) | NONCE(12) | CIPHERTEXT + 16 Bytes MAC
pub const MAGIC: [u8; 5] = *b"LOCK1";

/// Lengths for header and AEAD
pub const MAGIC_LEN: usize = 5;
pub const NONCE_LEN: usize = 12; // 96-bit AES-GCM nonce
pub const TAG_LEN: usize = 16; // 128-bit GCM authentication tag
pub const HEADER_LEN: usize = MAGIC_LEN + NONCE_LEN;

pub const MAX_PATH_LEN: usize = 4096;
pub const SMALL_BUFFER_SIZE: usize = 1024; // For small operations
pub const DEFAULT_VEC_CAPACITY: usize = 100; // Default capacity for vectors
pub const LARGE_VEC_THRESHOLD: usize = 1000; // Threshold for large vector operations

/// extension for encrypted files
pub const ENCRYPTED_EXT: &str = ".lock";

/// ----- passphrase-protected keyfile format -----
/// Format v1: KEYFILE_MAGIC(5) | SALT(16) | NONCE(12) | AES-256-GCM(PT=32B key, AAD=header) || TAG(16)
pub const KEYFILE_MAGIC: [u8; 5] = *b"LKEY1";
pub const KEYFILE_MAGIC_LEN: usize = 5;
pub const SALT_LEN: usize = 16;
pub const KEYFILE_HEADER_LEN: usize = KEYFILE_MAGIC_LEN + SALT_LEN + NONCE_LEN;
