use anyhow::{Context, Result, bail};
use std::{fs, path::Path};
use zeroize::Zeroizing;

pub struct Aes256GcmKey(pub Zeroizing<[u8; 32]>);

impl std::fmt::Debug for Aes256GcmKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Aes256GcmKey(REDACTED)")
    }
}

pub fn read_lock_key(path: &Path) -> Result<Aes256GcmKey> {
    let raw = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;

    // trim common whitespace (\n, \r\n, spaces)
    let s = raw.trim();

    // 32 bytes validation before decoding
    if s.len() != 64 {
        bail!(
            "invalid key: expected 64 hex chars (32 bytes), found {}",
            s.len()
        );
    }

    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("invalid key: must contain only 0-9, a-f, A-F");
    }

    // decode into a zerozing buffer
    let decoded = Zeroizing::new(
        hex::decode(s).with_context(|| format!("decoding hex in {}", path.display()))?,
    );

    if decoded.len() != 32 {
        bail!(
            "invalid key: decoded length was {}, expected 32",
            decoded.len()
        );
    }

    // Move into fixed-size array protected by Zeroizing
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&decoded);
    Ok(Aes256GcmKey(Zeroizing::new(arr)))
}

/// Borrow raw bytes (no copies).
pub fn key_bytes(key: &Aes256GcmKey) -> &[u8; 32] {
    &key.0
}
