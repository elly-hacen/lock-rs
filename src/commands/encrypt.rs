use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use anyhow::{Context, Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use rand::{TryRngCore, rngs::OsRng};
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::{fs, path::Path};
use tempfile::NamedTempFile;
use zeroize::Zeroizing;

use crate::options::encrypt::EncryptArgs;
use lock::walk::{PlanEntry, build_encrypt_plan};
use lock::{ENCRYPTED_EXT, HEADER_LEN, KEYFILE_MAGIC, MAGIC, MAGIC_LEN, NONCE_LEN};
use lock::{key_bytes, read_lock_key};

fn is_protected_keyfile(path: &Path) -> bool {
    match fs::read(path) {
        Ok(bytes) => bytes.starts_with(&KEYFILE_MAGIC),
        Err(_) => false,
    }
}

/// Load key with interactive retries:
/// - If `passphrase_prompt` is set OR file is protected (LKEY1), prompt up to 3 times
/// - On success, return key; on 3 failures, print and exit(1)
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

pub fn run(args: EncryptArgs) -> Result<()> {
    // Load key (handles retries/prompting/zeroization)
    let key = load_key_with_retries(&args.key_file, args.passphrase_prompt)
        .with_context(|| format!("loading key from {}", args.key_file.display()))?;
    let raw: &[u8; 32] = key_bytes(&key);

    fs::create_dir_all(&args.output)
        .with_context(|| format!("creating {}", args.output.display()))?;

    let plan = build_encrypt_plan(
        &args.input,
        &args.output,
        args.include_hidden,
        ENCRYPTED_EXT,
    )?;
    let total = plan.len();
    let overwrite = args.overwrite;

    // --jobs by rayon pool
    let threads = args.jobs.unwrap_or_else(|| rayon::current_num_threads());
    let pool = ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap();

    if !args.verbose {
        // ---- progress bar (parallel) ----
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::with_template("[{elapsed_precise}] {bar:40} {pos}/{len}").unwrap(),
        );

        let result = pool.install(|| {
            plan.par_iter()
                .try_for_each(|PlanEntry { src, dst }| -> Result<()> {
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)
                            .with_context(|| format!("creating {}", parent.display()))?;
                    }
                    if dst.exists() {
                        if overwrite {
                            let _ = fs::remove_file(&dst);
                        } else {
                            pb.inc(1);
                            return Ok(());
                        }
                    }

                    // Read plaintext
                    let pt =
                        fs::read(&src).with_context(|| format!("reading {}", src.display()))?;

                    // Per-file nonce
                    let mut nonce_bytes = [0u8; NONCE_LEN];
                    OsRng
                        .try_fill_bytes(&mut nonce_bytes)
                        .context("OsRng failed to generate nonce")?;
                    let nonce = Nonce::from_slice(&nonce_bytes);

                    // AAD = MAGIC || NONCE
                    let mut aad_buf = [0u8; HEADER_LEN];
                    aad_buf[..MAGIC_LEN].copy_from_slice(&MAGIC);
                    aad_buf[MAGIC_LEN..MAGIC_LEN + NONCE_LEN].copy_from_slice(&nonce_bytes);

                    // Local cipher per task
                    let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(raw));

                    // Encrypt
                    let ct = cipher
                        .encrypt(
                            nonce,
                            Payload {
                                msg: pt.as_ref(),
                                aad: &aad_buf,
                            },
                        )
                        .map_err(|_| anyhow!("encrypting {}", src.display()))?;

                    write_atomic(dst, &MAGIC, &nonce_bytes, &ct)
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
        // ---- verbose mode (serial) ----
        for (idx, PlanEntry { src, dst }) in plan.into_iter().enumerate() {
            let n = idx + 1;

            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            if dst.exists() {
                if overwrite {
                    let _ = fs::remove_file(&dst);
                } else {
                    println!(
                        "[{}/{}] skip (exists): {} -> {}",
                        n,
                        total,
                        src.display(),
                        dst.display()
                    );
                    continue;
                }
            }

            let pt = fs::read(&src).with_context(|| format!("reading {}", src.display()))?;

            let mut nonce_bytes = [0u8; NONCE_LEN];
            OsRng
                .try_fill_bytes(&mut nonce_bytes)
                .context("OsRng failed to generate nonce")?;
            let nonce = Nonce::from_slice(&nonce_bytes);

            let mut aad_buf = [0u8; HEADER_LEN];
            aad_buf[..MAGIC_LEN].copy_from_slice(&MAGIC);
            aad_buf[MAGIC_LEN..MAGIC_LEN + NONCE_LEN].copy_from_slice(&nonce_bytes);

            let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(raw));
            let ct = cipher
                .encrypt(
                    nonce,
                    Payload {
                        msg: pt.as_ref(),
                        aad: &aad_buf,
                    },
                )
                .map_err(|_| anyhow!("encrypting {}", src.display()))?;

            write_atomic(&dst, &MAGIC, &nonce_bytes, &ct)
                .with_context(|| format!("writing {}", dst.display()))?;

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
