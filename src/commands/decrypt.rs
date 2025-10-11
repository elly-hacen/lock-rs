use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, Key, KeyInit, Payload},
};
use anyhow::{Context, Result, anyhow, bail};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::{fs, path::Path};
use tempfile::NamedTempFile;

use crate::options::decrypt::DecryptArgs;
use lock::walk::{PlanEntry, build_decrypt_plan};
use lock::{ENCRYPTED_EXT, HEADER_LEN, MAGIC, MAGIC_LEN, NONCE_LEN, TAG_LEN};
use lock::{key_bytes, load_key_with_retries};

pub fn run(args: DecryptArgs) -> Result<()> {
    // 1) Load key (handles retries/prompting/zeroization)
    let key = load_key_with_retries(&args.key_file, args.passphrase_prompt)
        .with_context(|| format!("loading key from {}", args.key_file.display()))?;
    let raw: &[u8; 32] = key_bytes(&key);

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

        let threads = args.jobs.unwrap_or_else(rayon::current_num_threads);
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();

        let result = pool.install(|| {
            plan.par_iter()
                .map_init(
                    || Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(raw)),
                    |cipher, PlanEntry { src, dst }| -> Result<()> {
                        if dst.exists() {
                            pb.inc(1);
                            return Ok(());
                        }

                        if let Some(parent) = dst.parent() {
                            fs::create_dir_all(parent)
                                .with_context(|| format!("creating {}", parent.display()))?;
                        }

                        let data =
                            fs::read(src).with_context(|| format!("reading {}", src.display()))?;
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
                        aad[MAGIC_LEN..HEADER_LEN].copy_from_slice(nonce_bytes);

                        // Decrypt
                        let nonce = Nonce::from_slice(nonce_bytes);
                        let mut pt = cipher
                            .decrypt(nonce, Payload { msg: ct, aad: &aad })
                            .map_err(|_| anyhow!("authentication failed: {}", src.display()))?;

                        // Atomic write
                        write_atomic_plain(dst, &pt)
                            .with_context(|| format!("writing {}", dst.display()))?;

                        // Zeroize plaintext from memory after writing
                        {
                            use zeroize::Zeroize;
                            pt.zeroize();
                        }

                        pb.inc(1);
                        Ok(())
                    },
                )
                .collect::<Result<Vec<_>>>()
                .map(|_| ())
        });

        match result {
            Ok(()) => pb.finish_with_message("done"),
            Err(_) => pb.finish_with_message("failed"),
        }
        result
    } else {
        // verbose: serial with per-file lines
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(raw));
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
            aad[MAGIC_LEN..HEADER_LEN].copy_from_slice(nonce_bytes);

            let nonce = Nonce::from_slice(nonce_bytes);
            let mut pt = cipher
                .decrypt(nonce, Payload { msg: ct, aad: &aad })
                .map_err(|_| anyhow!("authentication failed: {}", src.display()))?;

            write_atomic_plain(&dst, &pt).with_context(|| format!("writing {}", dst.display()))?;

            // Zeroize plaintext buffer
            {
                use zeroize::Zeroize;
                pt.zeroize();
            }

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

    // data hits disk on all platforms
    let _ = tmp.as_file().sync_all();

    // atomic renaming
    tmp.persist(dst).map(|_| ()).map_err(|e| e.error.into())
}
