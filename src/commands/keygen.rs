use crate::options::keygen::{Algo, KeygenArgs};
use anyhow::{Context, Result, anyhow};
use colored::*;
use rand::{TryRngCore, rngs::OsRng};
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};

pub fn run(args: KeygenArgs) -> Result<()> {
    if args.out.exists() && !args.force {
        eprintln!(
            "{}",
            format!(
                "'{}' already exists. Use -f/--force to overwrite.",
                args.out.display()
            )
            .white()
        );
        std::process::exit(1);
    }

    match args.algo {
        Algo::AesGcm => generate_aes_gcm_key(&args.out, args.force)?,
    };

    println!(
        "\n{} {} {}\n",
        "[SUCCESS]".green(),
        "AES-256-GCM key generated and saved to".white(),
        args.out.display().to_string().cyan()
    );
    println!("{}", "IMPORTANT SECURITY NOTICE".yellow());
    println!(
        "{}",
        "  - Keep lock.key safe and make multiple backups!".white()
    );
    println!(
        "{}",
        "  - Without this key, your data will be lost forever.".white()
    );
    Ok(())
}

fn generate_aes_gcm_key(out: &std::path::Path, force: bool) -> Result<()> {
    // 32-byte key for AES-256-GCM
    let mut key = [0u8; 32];
    OsRng
        .try_fill_bytes(&mut key)
        .context("OsRng failed to generate key bytes")?;

    let hex_key = hex::encode(key);

    #[cfg(unix)]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        let mut opts = OpenOptions::new();
        opts.write(true).create(true).mode(0o600);
        if force {
            opts.truncate(true);
        } else {
            opts.create_new(true);
        }
        match opts.open(out) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::AlreadyExists && !force => {
                return Err(anyhow!(
                    "'{}' already exists. Use -f/--force to overwrite.",
                    out.display()
                ));
            }
            Err(e) => return Err(e).with_context(|| format!("creating {}", out.display())),
        }
    };

    #[cfg(not(unix))]
    let mut file = {
        let mut opts = OpenOptions::new();
        opts.write(true).create(true);
        if force {
            opts.truncate(true);
        } else {
            opts.create_new(true);
        }
        match opts.open(out) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::AlreadyExists && !force => {
                return Err(anyhow!(
                    "'{}' already exists. Use -f/--force to overwrite.",
                    out.display()
                ));
            }
            Err(e) => return Err(e).with_context(|| format!("creating {}", out.display())),
        }
    };

    // Write hex + newline, check errors
    writeln!(file, "{hex_key}").with_context(|| format!("writing {}", out.display()))?;
    file.flush()?;

    // Wipe key from memory
    use zeroize::Zeroize;
    key.zeroize();

    Ok(())
}
