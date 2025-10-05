use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use anyhow::{Context, Result, anyhow, bail};
use std::{fs, path::Path};
use tempfile::NamedTempFile;

use crate::options::decrypt::DecryptArgs;
use lock::walk::{PlanEntry, build_decrypt_plan};
use lock::{ENCRYPTED_EXT, HEADER_LEN, MAGIC, MAGIC_LEN, NONCE_LEN, TAG_LEN};
use lock::{key_bytes, read_lock_key};

pub fn run(args: DecryptArgs) -> Result<()> {
    // 1) Load key
    let key = read_lock_key(&args.key_file)
        .with_context(|| format!("loading key from {}", args.key_file.display()))?;
    let raw: &[u8; 32] = key_bytes(&key);
    let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(raw));

    fs::create_dir_all(&args.output)
        .with_context(|| format!("creating {}", args.output.display()))?;

    // 2) Build plan via walk.rs (include_hidden: false for now)
    let plan = build_decrypt_plan(&args.input, &args.output, false, ENCRYPTED_EXT)?;

    // 3) Execute plan
    for PlanEntry { src, dst } in plan {
        // Skip if destination exists (--overwrite-policy <skip|owerwrite|rename>)
        if dst.exists() {
            eprintln!("skip (exists): {}", dst.display());
            continue;
        }

        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }

        // Read and sanity-check header
        let data = fs::read(&src).with_context(|| format!("reading {}", src.display()))?;
        if data.len() < HEADER_LEN + TAG_LEN {
            bail!("{}: file too short to be a valid .lock", src.display());
        }

        // Parse header: MAGIC | NONCE
        let (magic, rest) = data.split_at(MAGIC_LEN);
        if magic != MAGIC {
            bail!("{}: bad magic (not a LOCK1 file)", src.display());
        }
        let (nonce_bytes, ct) = rest.split_at(NONCE_LEN);

        // AAD = MAGIC || NONCE
        let mut aad = [0u8; HEADER_LEN];
        aad[..MAGIC_LEN].copy_from_slice(&MAGIC);
        aad[MAGIC_LEN..].copy_from_slice(nonce_bytes);

        // Decrypt
        let nonce = Nonce::from_slice(nonce_bytes);
        let pt = cipher
            .decrypt(nonce, Payload { msg: ct, aad: &aad })
            .map_err(|_| anyhow!("authentication failed: {}", src.display()))?;

        // Atomic write
        write_atomic_plain(&dst, &pt).with_context(|| format!("writing {}", dst.display()))?;

        println!("ok: {} -> {}", src.display(), dst.display());
    }

    Ok(())
}

/// Atomic write for plaintext bytes
fn write_atomic_plain(dst: &Path, pt: &[u8]) -> Result<()> {
    let parent = dst.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("creating temp in {}", parent.display()))?;

    use std::io::Write;
    tmp.write_all(pt)?;
    tmp.flush()?;

    #[cfg(unix)]
    {
        let _ = tmp.as_file().sync_all();
    }

    tmp.persist(dst).map(|_| ()).map_err(|e| e.error.into())
}
