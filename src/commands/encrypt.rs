use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use anyhow::{Context, Result, anyhow};
use rand::{TryRngCore, rngs::OsRng};
use std::{fs, path::Path};
use tempfile::NamedTempFile;

use crate::options::encrypt::EncryptArgs;
use lock::walk::{PlanEntry, build_encrypt_plan};
use lock::{ENCRYPTED_EXT, HEADER_LEN, MAGIC, MAGIC_LEN, NONCE_LEN};
use lock::{key_bytes, read_lock_key};

pub fn run(args: EncryptArgs) -> Result<()> {
    // 1) Load key (OWNED)
    let key = read_lock_key(&args.key_file)
        .with_context(|| format!("loading key from {}", args.key_file.display()))?;

    // 2) Borrow raw bytes (no copy)
    let raw: &[u8; 32] = key_bytes(&key);

    // 3) Cipher
    let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(raw));

    fs::create_dir_all(&args.output)
        .with_context(|| format!("creating {}", args.output.display()))?;

    // 4) Build plan via walk.rs
    let plan = build_encrypt_plan(
        &args.input,
        &args.output,
        args.include_hidden,
        ENCRYPTED_EXT,
    )?;

    // 5) Execute plan
    for PlanEntry { src, dst } in plan {
        // Parent dir
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        // Overwrite policy:
        // If the destination already exists:
        //   - With `--overwrite`: delete the existing file first.
        //   - Without `--overwrite`: print a user-level “skip” message and continue
        if dst.exists() {
            if args.overwrite {
                // pre-remove existing to avoid conflicts
                let _ = fs::remove_file(&dst);
            } else {
                eprintln!("skip (exists): {}", dst.display());
                continue;
            }
        }

        // Read plaintext (OWNED Vec<u8>)
        let pt = fs::read(&src).with_context(|| format!("reading {}", src.display()))?;

        // Nonce per file
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng
            .try_fill_bytes(&mut nonce_bytes)
            .context("OsRng failed to generate nonce")?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // AAD = MAGIC || NONCE
        let mut aad_buf = [0u8; HEADER_LEN];
        aad_buf[..MAGIC_LEN].copy_from_slice(&MAGIC);
        aad_buf[MAGIC_LEN..MAGIC_LEN + NONCE_LEN].copy_from_slice(&nonce_bytes);
        let aad: &[u8] = &aad_buf;

        // Encrypt
        let ct = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: pt.as_ref(),
                    aad,
                },
            )
            .map_err(|_| anyhow!("encrypting {}", src.display()))?;

        // Atomic write
        write_atomic(&dst, &MAGIC, &nonce_bytes, &ct)
            .with_context(|| format!("writing {}", dst.display()))?;

        println!("ok: {} -> {}", src.display(), dst.display());
    }

    Ok(())
}

/// Write MAGIC | NONCE | CIPHERTEXT atomically
fn write_atomic(dst: &Path, magic: &[u8; 5], nonce12: &[u8; 12], ct: &[u8]) -> Result<()> {
    let parent = dst.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("creating temp in {}", parent.display()))?;

    use std::io::Write;
    tmp.write_all(magic)?;
    tmp.write_all(nonce12)?;
    tmp.write_all(ct)?;
    tmp.flush()?;

    #[cfg(unix)]
    {
        let _ = tmp.as_file().sync_all();
    }

    // Atomic rename into place
    tmp.persist(dst).map(|_| ()).map_err(|e| e.error.into())
}
