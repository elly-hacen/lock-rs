use anyhow::{Context, Result};
use colored::*;
use serde_json::json;
use std::fs;

use crate::options::inspect::InspectArgs;
use lock::{KEYFILE_MAGIC, MAGIC, MAGIC_LEN, NONCE_LEN, SALT_LEN};

pub fn run(args: InspectArgs) -> Result<()> {
    let path = &args.file;
    let data = fs::read(path).with_context(|| format!("reading {}", path.display()))?;

    if data.len() < MAGIC_LEN {
        if args.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "file": path.display().to_string(),
                    "error": "file too short"
                }))?
            );
        } else {
            eprintln!(
                "{} {}",
                "Error:".red().bold(),
                format!("{} is too short to be a valid file", path.display()).white()
            );
        }
        return Ok(());
    }

    // ----------- LOCK1 encrypted file -----------
    if data.starts_with(&MAGIC) {
        let nonce_hex = if data.len() >= MAGIC_LEN + NONCE_LEN {
            Some(hex::encode(&data[MAGIC_LEN..MAGIC_LEN + NONCE_LEN]))
        } else {
            None
        };

        if args.json {
            let j = json!({
                "file": path.display().to_string(),
                "type": "encrypted",
                "version": "LOCK1",
                "algorithm": "AES-256-GCM",
                "header_length": MAGIC_LEN + NONCE_LEN,
                "nonce": nonce_hex
            });
            println!("{}", serde_json::to_string_pretty(&j)?);
        } else {
            println!("{}", "Encrypted file detected".green().bold());
            println!("{} {}", "File:".cyan(), path.display());
            println!("{} {}", "Type:".cyan(), "LOCK1 (encrypted data)".white());
            println!("{} {}", "Algorithm:".cyan(), "AES-256-GCM".white());
            println!(
                "{} {}",
                "Header length:".cyan(),
                (MAGIC_LEN + NONCE_LEN).to_string().white()
            );
            match nonce_hex {
                Some(n) => println!("{} {}", "Nonce (hex):".cyan(), n.white()),
                None => println!("{} {}", "Nonce:".cyan(), "(missing / truncated)".white()),
            }
        }
    }
    // ----------- LKEY1 protected keyfile -----------
    else if data.starts_with(&KEYFILE_MAGIC) {
        let salt_hex;
        let nonce_hex;
        if data.len() >= 5 + SALT_LEN + NONCE_LEN {
            salt_hex = Some(hex::encode(&data[5..5 + SALT_LEN]));
            nonce_hex = Some(hex::encode(&data[5 + SALT_LEN..5 + SALT_LEN + NONCE_LEN]));
        } else {
            salt_hex = None;
            nonce_hex = None;
        }

        if args.json {
            let j = json!({
                "file": path.display().to_string(),
                "type": "keyfile",
                "version": "LKEY1",
                "protected": true,
                "salt": salt_hex,
                "nonce": nonce_hex
            });
            println!("{}", serde_json::to_string_pretty(&j)?);
        } else {
            println!("{}", "Keyfile detected".yellow().bold());
            println!("{} {}", "File:".cyan(), path.display());
            println!(
                "{} {}",
                "Type:".cyan(),
                "LKEY1 passphrase-protected key".white()
            );
            println!(
                "{} {}",
                "Format:".cyan(),
                "LKEY1 | SALT(16) | NONCE(12) | AES-GCM(ciphertext+tag)".white()
            );
            match salt_hex {
                Some(s) => println!("{} {}", "Salt (hex):".cyan(), s.white()),
                None => println!("{} {}", "Salt:".cyan(), "(missing / truncated)".white()),
            }
            match nonce_hex {
                Some(n) => println!("{} {}", "Nonce (hex):".cyan(), n.white()),
                None => println!("{} {}", "Nonce:".cyan(), "(missing / truncated)".white()),
            }
        }
    }
    // ----------- Unknown format -----------
    else {
        if args.json {
            let j = json!({
                "file": path.display().to_string(),
                "type": "unknown",
                "message": "Unrecognized file format"
            });
            println!("{}", serde_json::to_string_pretty(&j)?);
        } else {
            println!("{}", "Unrecognized file format".red().bold());
            println!("{} {}", "File:".cyan(), path.display());
        }
    }

    Ok(())
}
