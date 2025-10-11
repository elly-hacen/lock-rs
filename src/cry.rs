use anyhow::{Context, Result, anyhow, bail};
use std::{fs, path::Path};
use zeroize::Zeroizing;

use crate::{KEYFILE_HEADER_LEN, KEYFILE_MAGIC, KEYFILE_MAGIC_LEN, SALT_LEN};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use argon2::{Algorithm, Argon2, Params, Version};

/// Wrapper for a 32-byte AES-256-GCM key that zeroizes on drop.
pub struct Aes256GcmKey(pub Zeroizing<[u8; 32]>);

impl std::fmt::Debug for Aes256GcmKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Aes256GcmKey(REDACTED)")
    }
}

/// Read the encryption key from disk.
///
/// Supports two formats:
/// 1) plaintext hex: file contains exactly 64 hex chars (32 bytes).
/// 2) Passphrase-protected (binary):
///    LKEY1 | SALT(16) | NONCE(12) | AES-256-GCM(PT=32B key, AAD=header) || TAG(16).
///
/// If the file is protected (starts with `LKEY1`) and no `passphrase` is provided,
/// this returns an error prompting the caller to use `--passphrase-prompt`.
pub fn read_lock_key(path: &Path, passphrase: Option<&str>) -> Result<Aes256GcmKey> {
    let raw = fs::read(path).with_context(|| format!("reading {}", path.display()))?;

    // Passphrase-protected keyfile
    if raw.starts_with(&KEYFILE_MAGIC) {
        let pass = passphrase.ok_or_else(|| {
            anyhow!(
                "'{}' is passphrase-protected; run with --passphrase-prompt (or provide a passphrase)",
                path.display()
            )
        })?;

        if raw.len() < KEYFILE_HEADER_LEN + 16
        /* GCM tag */
        {
            bail!("{}: keyfile too short", path.display());
        }

        let salt = &raw[KEYFILE_MAGIC_LEN..KEYFILE_MAGIC_LEN + SALT_LEN];
        let nonce_bytes = &raw[KEYFILE_MAGIC_LEN + SALT_LEN..KEYFILE_HEADER_LEN];
        let ct = &raw[KEYFILE_HEADER_LEN..];

        // Derive 32B wrap key via Argon2id (fixed params for v1)
        let mut wrap = Zeroizing::new([0u8; 32]);
        // m = 64 MiB, t = 3, p = 1, out_len = 32
        let params =
            Params::new(64 * 1024, 3, 1, Some(32)).map_err(|e| anyhow!("argon2 params: {e}"))?;
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        argon
            .hash_password_into(pass.as_bytes(), salt, &mut *wrap)
            .map_err(|e| anyhow!("argon2 derive: {e}"))?;

        // Cipher to unwrap the data key
        let cipher =
            Aes256Gcm::new_from_slice(&wrap[..]).map_err(|e| anyhow!("init cipher: {e}"))?;
        let nonce = Nonce::from_slice(nonce_bytes);

        // AAD = full header (MAGIC | SALT | NONCE)
        let mut aad = [0u8; KEYFILE_HEADER_LEN];
        aad[..KEYFILE_MAGIC_LEN].copy_from_slice(&KEYFILE_MAGIC);
        aad[KEYFILE_MAGIC_LEN..KEYFILE_MAGIC_LEN + SALT_LEN].copy_from_slice(salt);
        aad[KEYFILE_MAGIC_LEN + SALT_LEN..].copy_from_slice(nonce_bytes);

        // Decrypt and immediately zeroize the temporary buffer after use
        let mut pt = cipher
            .decrypt(nonce, Payload { msg: ct, aad: &aad })
            .map_err(|_| anyhow!("invalid passphrase or corrupted keyfile"))?;

        if pt.len() != 32 {
            bail!("decrypted key length was {}, expected 32", pt.len());
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&pt);

        // wipe decrypted key material from the temporary Vec<u8>
        use zeroize::Zeroize;
        pt.zeroize();

        return Ok(Aes256GcmKey(Zeroizing::new(arr)));
    }

    // Legacy plaintext hex path
    let s = std::str::from_utf8(&raw)
        .with_context(|| format!("{} is not UTF-8; expected hex", path.display()))?
        .trim();

    if s.len() != 64 {
        bail!(
            "invalid key: expected 64 hex chars (32 bytes), found {}",
            s.len()
        );
    }
    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("invalid key: must contain only 0-9, a-f, A-F");
    }

    let decoded = Zeroizing::new(
        hex::decode(s).with_context(|| format!("decoding hex in {}", path.display()))?,
    );
    if decoded.len() != 32 {
        bail!(
            "invalid key: decoded length was {}, expected 32",
            decoded.len()
        );
    }

    let mut arr = [0u8; 32];
    arr.copy_from_slice(&decoded);
    Ok(Aes256GcmKey(Zeroizing::new(arr)))
}

/// Borrow raw key bytes (no copies).
pub fn key_bytes(key: &Aes256GcmKey) -> &[u8; 32] {
    &key.0
}

/// Load key with interactive retries
///
/// - If `passphrase_prompt` is set OR file is protected (LKEY1), prompt up to 3 times
/// - On success, return key; on 3 failures, print and exit(1)
pub fn load_key_with_retries(key_path: &Path, passphrase_prompt: bool) -> Result<Aes256GcmKey> {
    // Check if keyfile is protected by reading first few bytes
    let is_protected = fs::read(key_path)
        .ok()
        .map(|bytes| bytes.starts_with(&KEYFILE_MAGIC))
        .unwrap_or(false);

    if passphrase_prompt || is_protected {
        for attempt in 1..=3 {
            let msg = if attempt == 1 {
                "Keyfile passphrase: "
            } else {
                "Incorrect password. Try again: "
            };
            let pass = Zeroizing::new(rpassword::prompt_password(msg)?);
            match read_lock_key(key_path, Some(pass.as_str())) {
                Ok(k) => return Ok(k),
                Err(_) if attempt < 3 => {}
                Err(_) => {
                    eprintln!("Incorrect password (3 attempts). Exiting.");
                    std::process::exit(1);
                }
            }
        }
        unreachable!();
    } else {
        // Plaintext hex key path
        read_lock_key(key_path, None)
    }
}
