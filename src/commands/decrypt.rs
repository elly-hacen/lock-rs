use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use anyhow::{Context, Result, anyhow, bail};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::{fs, path::Path};
use tempfile::NamedTempFile;
use zeroize::Zeroizing;

use crate::options::decrypt::DecryptArgs;
use lock::walk::{PlanEntry, build_decrypt_plan};
use lock::{ENCRYPTED_EXT, HEADER_LEN, KEYFILE_MAGIC, MAGIC, MAGIC_LEN, NONCE_LEN, TAG_LEN};
use lock::{key_bytes, read_lock_key};

fn is_protected_keyfile(path: &Path) -> bool {
    match fs::read(path) {
        Ok(bytes) => bytes.starts_with(&KEYFILE_MAGIC),
        Err(_) => false,
    }
}

fn load_key_with_retries(key_path: &Path, passphrase_prompt: bool) -> Result<lock::Aes256GcmKey> {
    if passphrase_prompt || is_protected_keyfile(key_path) {
        for attempt in 1..=3 {
            let msg = if attempt == 1 {
                "Keyfile passphrase: "
            } else {
                "Incorrect password. Try again: "
            };
            let pass = Zeroizing::new(rpassword::prompt_password(msg)?);
            match read_lock_key(key_path, Some(pass.as_str())) {
                Ok(k) => return Ok(k),
                Err(_) if attempt < 3 => { /* keep looping */ }
                Err(_) => {
                    eprintln!("Incorrect password (3 attempts). Exiting.");
                    std::process::exit(1);
                }
            }
        }
        unreachable!();
    } else {
        read_lock_key(key_path, None)
    }
}

pub fn run(args: DecryptArgs) -> Result<()> {
    // 1) Load key (handles retries/prompting/zeroization)
    let key = load_key_with_retries(&args.key_file, args.passphrase_prompt)
        .with_context(|| format!("loading key from {}", args.key_file.display()))?;
    let raw: &[u8; 32] = key_bytes(&key);
    let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(raw));

    // 2) Ensure output dir
    fs::create_dir_all(&args.output)
        .with_context(|| format!("creating {}", args.output.display()))?;

    // 3) Build plan
    let plan = build_decrypt_plan(
        &args.input,
        &args.output,
        args.include_hidden,
        ENCRYPTED_EXT,
    )?;
    let total = plan.len();

    // 4) UI & parallelism
    if !args.verbose {
        // progress bar + parallel
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::with_template("[{elapsed_precise}] {bar:40} {pos}/{len}").unwrap(),
        );

        let threads = args.jobs.unwrap_or_else(|| rayon::current_num_threads());
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();

        let result = pool.install(|| {
            plan.par_iter()
                .try_for_each(|PlanEntry { src, dst }| -> Result<()> {
                    if dst.exists() {
                        pb.inc(1);
                        return Ok(());
                    }

                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)
                            .with_context(|| format!("creating {}", parent.display()))?;
                    }

                    let data =
                        fs::read(&src).with_context(|| format!("reading {}", src.display()))?;
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
                    write_atomic_plain(&dst, &pt)
                        .with_context(|| format!("writing {}", dst.display()))?;

                    pb.inc(1);
                    Ok(())
                })
        });

        match result {
            Ok(()) => pb.finish_with_message("done"),
            Err(_) => pb.finish_with_message("failed"),
        }
        result
    } else {
        // verbose: serial with per-file lines
        for (idx, PlanEntry { src, dst }) in plan.into_iter().enumerate() {
            let n = idx + 1;

            if dst.exists() {
                println!(
                    "[{}/{}] skip (exists): {} -> {}",
                    n,
                    total,
                    src.display(),
                    dst.display()
                );
                continue;
            }

            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }

            let data = fs::read(&src).with_context(|| format!("reading {}", src.display()))?;
            if data.len() < HEADER_LEN + TAG_LEN {
                bail!("{}: file too short to be a valid .lock", src.display());
            }

            let (magic, rest) = data.split_at(MAGIC_LEN);
            if magic != MAGIC {
                bail!("{}: bad magic (not a LOCK1 file)", src.display());
            }
            let (nonce_bytes, ct) = rest.split_at(NONCE_LEN);

            let mut aad = [0u8; HEADER_LEN];
            aad[..MAGIC_LEN].copy_from_slice(&MAGIC);
            aad[MAGIC_LEN..].copy_from_slice(nonce_bytes);

            let nonce = Nonce::from_slice(nonce_bytes);
            let pt = cipher
                .decrypt(nonce, Payload { msg: ct, aad: &aad })
                .map_err(|_| anyhow!("authentication failed: {}", src.display()))?;

            write_atomic_plain(&dst, &pt).with_context(|| format!("writing {}", dst.display()))?;

            println!(
                "[{}/{}] ok: {} -> {}",
                n,
                total,
                src.display(),
                dst.display()
            );
        }
        Ok(())
    }
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
