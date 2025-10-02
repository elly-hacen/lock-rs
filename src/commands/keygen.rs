use crate::options::keygen::{Algo, KeygenArgs};
use anyhow::{Context, Result};
use colored::*;
use rand::{TryRngCore, rngs::OsRng};
use std::fs::OpenOptions;
use std::io::Write;

pub fn run(args: KeygenArgs) -> Result<()> {
    if args.out.exists() && !args.force {
        eprintln!("{}", "Error: lock.key already exists!".white());
        eprintln!("{}", "Use -f/--force to overwrite.".white());
        std::process::exit(1);
    }

    match args.algo {
        Algo::Aes => generate_aes_key(&args.out, args.force)?,
    };

    println!(
        "\n{} {} {}",
        "[SUCCESS]".green(),
        "AES-256 key generated and saved to".white(),
        args.out.display().to_string().cyan()
    );

    println!();
    println!("{}", "IMPORTANT SECURITY NOTICE".yellow());
    println!();
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

fn generate_aes_key(out: &std::path::Path, force: bool) -> Result<()> {
    // 32-byte AES-256 key
    let mut key = [0u8; 32];
    OsRng
        .try_fill_bytes(&mut key)
        .context("OsRng failed to generate key bytes")?;

    let hex_key = hex::encode(key);

    #[cfg(unix)]
    let open_file = || -> Result<std::fs::File> {
        use std::os::unix::fs::OpenOptionsExt;
        let mut opts = OpenOptions::new();
        opts.write(true).create(true).mode(0o600);
        if force {
            opts.truncate(true);
        } else {
            opts.create(true);
        }
        match opts.open(out) {
            Ok(f) => Ok(f),
            Err(e) => Err(e).with_context(|| format!("creating {}", out.display())),
        }
    };

    #[cfg(not(unix))]
    let open_file = || -> Result<std::fs::File> {
        let mut opts = OpenOptions::new();
        opts.write(true).create(true);
        if force {
            opts.truncate(true);
        } else {
            opts.create(true);
        }
        match opts.open(out) {
            Ok(f) => Ok(f),
            Err(e) => Err(e).with_context(|| format!("creating {}", out.display())),
        }
    };

    let mut file = open_file()?;

    writeln!(file, "{hex_key}").with_context(|| format!("writing {}", out.display()))?;
    file.flush()?;

    // wipe key from memory
    use zeroize::Zeroize;
    key.zeroize();

    Ok(())
}
