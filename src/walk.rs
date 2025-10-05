use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use crate::utils::{is_hidden, is_image};

#[derive(Debug, Clone)]
pub struct PlanEntry {
    pub src: PathBuf,
    pub dst: PathBuf,
}

/// Build a plan of (src,dst) for all image files under `input_root`,
/// mirrored under `output_root`, with `enc_ext` appended to the filename.
/// e.g. "photo.jpg" -> "photo.jpg.lock"
pub fn build_encrypt_plan(
    input_root: &Path,
    output_root: &Path,
    include_hidden: bool,
    enc_ext: &str,
) -> Result<Vec<PlanEntry>> {
    let in_root = std::fs::canonicalize(input_root)
        .with_context(|| format!("canonicalizing {}", input_root.display()))?;
    // If output was just created, canonicalize may fail; fallback to given path
    let out_root = std::fs::canonicalize(output_root).unwrap_or_else(|_| output_root.to_path_buf());

    let mut plan = Vec::new();

    for entry in WalkDir::new(&in_root).follow_links(false) {
        let entry = entry?;
        if should_skip(&entry, include_hidden) {
            continue;
        }
        let src = entry.path();

        if !is_image(src) {
            continue;
        }

        // Map to mirrored destination + extension
        let rel = src
            .strip_prefix(&in_root)
            .context("computing relative path")?;
        let mut dst = out_root.join(rel);

        // Append enc_ext to filename: "name.ext" -> "name.ext{enc_ext}"
        let new_name = match dst.file_name() {
            Some(name) => format!("{}{}", name.to_string_lossy(), enc_ext),
            None => format!("file{}", enc_ext),
        };
        dst.set_file_name(new_name);

        plan.push(PlanEntry {
            src: src.to_path_buf(),
            dst,
        });
    }

    Ok(plan)
}

/// Build a plan of (src,dst) for all encrypted files under `input_root`
/// (files whose extension matches `enc_ext` like ".lock") mirrored to
/// `output_root`, with the encrypted suffix removed.
/// e.g. "photo.jpg.lock" -> "photo.jpg"
pub fn build_decrypt_plan(
    input_root: &Path,
    output_root: &Path,
    include_hidden: bool,
    enc_ext: &str, // e.g. ".lock"
) -> Result<Vec<PlanEntry>> {
    let in_root = std::fs::canonicalize(input_root)
        .with_context(|| format!("canonicalizing {}", input_root.display()))?;
    // If output was just created, canonicalize may fail; fallback to given path
    let out_root = std::fs::canonicalize(output_root).unwrap_or_else(|_| output_root.to_path_buf());

    let mut plan = Vec::new();
    let ext_no_dot = enc_ext.trim_start_matches('.');

    for entry in WalkDir::new(&in_root).follow_links(false) {
        let entry = entry?;
        if should_skip(&entry, include_hidden) {
            continue;
        }
        let src = entry.path();

        // Only take files with extension == ext_no_dot (e.g., "lock")
        let is_enc = src
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case(ext_no_dot))
            .unwrap_or(false);
        if !is_enc {
            continue;
        }

        // Map to mirrored destination with ".lock" stripped
        let rel = src
            .strip_prefix(&in_root)
            .context("computing relative path")?;
        let mut dst = out_root.join(rel);

        // For "name.ext.lock" -> use file_stem() which yields "name.ext"
        match src.file_stem() {
            Some(stem) => dst.set_file_name(stem),
            None => dst.set_file_name(format!("file")),
        };

        plan.push(PlanEntry {
            src: src.to_path_buf(),
            dst,
        });
    }

    Ok(plan)
}

fn should_skip(entry: &DirEntry, include_hidden: bool) -> bool {
    let md = match entry.metadata() {
        Ok(m) => m,
        Err(_) => return true,
    };
    if md.is_dir() {
        return true;
    }
    if !include_hidden && is_hidden(entry.path()) {
        return true;
    }
    false
}
