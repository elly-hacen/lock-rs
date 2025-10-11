use crate::options::keygen::{Algo, KeygenArgs};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use anyhow::{Context, Result, anyhow};
use argon2::{Algorithm, Argon2, Params, Version};
use colored::*;
use lock::{KEYFILE_MAGIC, NONCE_LEN, SALT_LEN};
use rand::{TryRngCore, rngs::OsRng};
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use zeroize::Zeroizing;

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
        Algo::AesGcm => generate_aes_gcm_key(&args.out, args.force, args.passphrase_prompt)?,
    };

    println!(
        "\n{} {} {}\n",
        "[SUCCESS]".green(),
        if args.passphrase_prompt {
            "Passphrase-protected AES-256-GCM key saved to".white()
        } else {
            "AES-256-GCM key generated and saved to".white()
        },
        args.out.display().to_string().cyan()
    );
    println!("{}", "IMPORTANT SECURITY NOTICE".yellow());
    println!(
        "{}",
        "  - Keep lock.key safe and make multiple backups!".white()
    );
    println!(
        "{}",
        "  - Without this key (and passphrase if set), your data will be lost forever.".white()
    );
    Ok(())
}

fn open_out(out: &std::path::Path, force: bool) -> Result<std::fs::File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut opts = OpenOptions::new();
        opts.write(true).create(true).mode(0o600);
        if force {
            opts.truncate(true);
        } else {
            opts.create_new(true);
        }
        match opts.open(out) {
            Ok(f) => Ok(f),
            Err(e) if e.kind() == ErrorKind::AlreadyExists && !force => Err(anyhow!(
                "'{}' already exists. Use -f/--force to overwrite.",
                out.display()
            )),
            Err(e) => Err(e).with_context(|| format!("creating {}", out.display())),
        }
    }
    #[cfg(not(unix))]
    {
        let mut opts = OpenOptions::new();
        opts.write(true).create(true);
        if force {
            opts.truncate(true);
        } else {
            opts.create_new(true);
        }
        match opts.open(out) {
            Ok(f) => Ok(f),
            Err(e) if e.kind() == ErrorKind::AlreadyExists && !force => Err(anyhow!(
                "'{}' already exists. Use -f/--force to overwrite.",
                out.display()
            )),
            Err(e) => Err(e).with_context(|| format!("creating {}", out.display())),
        }
    }
}

fn generate_aes_gcm_key(out: &std::path::Path, force: bool, passphrase_prompt: bool) -> Result<()> {
    let mut key = [0u8; 32];
    OsRng
        .try_fill_bytes(&mut key)
        .context("OsRng failed to generate key bytes")?;
    let mut file = open_out(out, force)?;

    if !passphrase_prompt {
        let hex_key = hex::encode(key);
        writeln!(file, "{hex_key}").with_context(|| format!("writing {}", out.display()))?;
        file.flush()?;
    } else {
        // passphrases are zeroized on drop
        let pass1 = Zeroizing::new(rpassword::prompt_password("Set passphrase: ")?);
        let pass2 = Zeroizing::new(rpassword::prompt_password("Confirm passphrase: ")?);
        if pass1.as_str() != pass2.as_str() {
            return Err(anyhow!("passphrases did not match"));
        }

        let params =
            Params::new(64 * 1024, 3, 1, Some(32)).map_err(|e| anyhow!("argon2 params: {e}"))?;
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut wrap = Zeroizing::new([0u8; 32]);
        let mut salt = [0u8; SALT_LEN];
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng
            .try_fill_bytes(&mut salt)
            .context("OsRng failed to generate salt")?;
        OsRng
            .try_fill_bytes(&mut nonce_bytes)
            .context("OsRng failed to generate nonce")?;

        argon
            .hash_password_into(pass1.as_bytes(), &salt, &mut *wrap)
            .map_err(|e| anyhow!("argon2 derive: {e}"))?;

        let cipher =
            Aes256Gcm::new_from_slice(&wrap[..]).map_err(|e| anyhow!("init cipher: {e}"))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Use stack-allocated array instead of heap Vec
        let mut aad = [0u8; KEYFILE_MAGIC.len() + SALT_LEN + NONCE_LEN];
        let mut offset = 0;
        aad[offset..offset + KEYFILE_MAGIC.len()].copy_from_slice(&KEYFILE_MAGIC);
        offset += KEYFILE_MAGIC.len();
        aad[offset..offset + SALT_LEN].copy_from_slice(&salt);
        offset += SALT_LEN;
        aad[offset..offset + NONCE_LEN].copy_from_slice(&nonce_bytes);

        let ct = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: &key,
                    aad: &aad,
                },
            )
            .map_err(|_| anyhow!("encrypting keyfile"))?;

        use std::io::Write;
        file.write_all(&KEYFILE_MAGIC)?;
        file.write_all(&salt)?;
        file.write_all(&nonce_bytes)?;
        file.write_all(&ct)?;
        file.flush()?;
        // pass1/pass2 are dropped here → zeroized
    }

    use zeroize::Zeroize;
    key.zeroize();
    Ok(())
}
