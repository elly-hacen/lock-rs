use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use crate::DEFAULT_VEC_CAPACITY;
use crate::utils::{is_hidden, is_image};

#[derive(Debug, Clone)]
pub struct PlanEntry {
    pub src: PathBuf,
    pub dst: PathBuf,
}

/// Used with `WalkDir::filter_entry(...)` to decide whether to keep this entry
/// and (if it's a directory) descend into it.
///
/// Behavior:
/// - If `include_hidden == false`, any path whose last component starts with `.` is skipped.
///   • For directories, returning `false` prunes the entire subtree (perf!).
///   • For files, it simply skips that one file.
/// - If `include_hidden == true`, nothing is filtered here.
///
/// Notes:
/// - "Hidden" is defined Unix-style via dot-prefix
#[inline]
fn should_descend(e: &DirEntry, include_hidden: bool) -> bool {
    if include_hidden {
        true
    } else {
        !is_hidden(e.path())
    }
}

/// Builds a plan of (source, destination) paths for all image files under `input_root`.
///
/// The destination path is mirrored under `output_root`, with the encryption extension
/// (`enc_ext`) appended to the filename.
///
/// # Example
/// If `enc_ext` is ".lock", "photo.jpg" -> "photo.jpg.lock"
pub fn build_encrypt_plan(
    input_root: &Path,
    output_root: &Path,
    include_hidden: bool,
    enc_ext: &str,
) -> Result<Vec<PlanEntry>> {
    // Get the absolute, normalized path for the input root.
    let in_root = std::fs::canonicalize(input_root)
        .with_context(|| format!("canonicalizing {}", input_root.display()))?;

    // Ensure output directory exists before canonicalizing
    std::fs::create_dir_all(output_root)
        .with_context(|| format!("creating output directory {}", output_root.display()))?;

    // Canonicalize the output root
    let out_root = std::fs::canonicalize(output_root)
        .with_context(|| format!("canonicalizing {}", output_root.display()))?;

    // First pass: count files to estimate capacity
    let file_count = WalkDir::new(&in_root)
        .follow_links(false)
        .same_file_system(true)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| should_descend(entry, include_hidden))
        .count();

    // Pre-allocate with actual file count + small buffer
    let capacity = file_count.max(DEFAULT_VEC_CAPACITY);
    let mut plan = Vec::with_capacity(capacity);

    for entry in WalkDir::new(&in_root)
        .follow_links(false)
        .same_file_system(true)
        .into_iter()
        .filter_entry(|e| should_descend(e, include_hidden))
    {
        let entry = entry?;

        // Skip directories; only process files
        if entry.file_type().is_dir() {
            continue;
        }

        let src = entry.path();
        if !is_image(src) {
            continue;
        }

        // --- Determine Destination Path (dst) ---

        // Compute the relative path from the input root.
        let rel = src
            .strip_prefix(&in_root)
            .with_context(|| format!("computing relative path for {}", src.display()))?;
        let mut dst = out_root.join(rel);

        // append `enc_ext` to the filename using OsString (non-UTF-8 safe).
        let base = dst
            .file_name()
            .unwrap_or_else(|| OsStr::new("file")) // Use "file" if no filename exists (e.g., "/")
            .to_os_string();
        let mut new_name = base;
        new_name.push(enc_ext);
        dst.set_file_name(new_name);

        plan.push(PlanEntry {
            src: src.to_path_buf(),
            dst,
        });
    }

    Ok(plan)
}

/// Builds a plan of (source, destination) paths for all encrypted files
/// under `input_root` whose extension matches `enc_ext`.
///
/// The destination path is mirrored under `output_root` with the encryption
/// suffix stripped.
///
/// # Example
/// If `enc_ext` is ".lock", "photo.jpg.lock" -> "photo.jpg"
pub fn build_decrypt_plan(
    input_root: &Path,
    output_root: &Path,
    include_hidden: bool,
    enc_ext: &str, // e.g. ".lock"
) -> Result<Vec<PlanEntry>> {
    // Get the absolute, normalized path for the input root.
    let in_root = std::fs::canonicalize(input_root)
        .with_context(|| format!("canonicalizing {}", input_root.display()))?;

    // Ensure output directory exists before canonicalizing
    std::fs::create_dir_all(output_root)
        .with_context(|| format!("creating output directory {}", output_root.display()))?;

    // Canonicalize the output root
    let out_root = std::fs::canonicalize(output_root)
        .with_context(|| format!("canonicalizing {}", output_root.display()))?;

    // First pass: count files to estimate capacity
    let file_count = WalkDir::new(&in_root)
        .follow_links(false)
        .same_file_system(true)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| should_descend(entry, include_hidden))
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case(enc_ext.trim_start_matches('.')))
                .unwrap_or(false)
        })
        .count();

    // Pre-allocate with actual file count + small buffer
    let capacity = file_count.max(DEFAULT_VEC_CAPACITY);
    let mut plan = Vec::with_capacity(capacity);
    // Trim leading dot from extension for comparison
    let ext_no_dot = enc_ext.trim_start_matches('.');

    for entry in WalkDir::new(&in_root)
        .follow_links(false)
        .same_file_system(true)
        .into_iter()
        .filter_entry(|e| should_descend(e, include_hidden))
    {
        let entry = entry?;

        // Skip directories.
        if entry.file_type().is_dir() {
            continue;
        }

        let src = entry.path();

        // Check if the file's last extension matches the encryption extension (case-insensitive).
        let is_enc = src
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case(ext_no_dot))
            .unwrap_or(false);

        if !is_enc {
            continue;
        }

        // Compute the relative path from the input root.
        let rel = src
            .strip_prefix(&in_root)
            .with_context(|| format!("computing relative path for {}", src.display()))?;
        let mut dst = out_root.join(rel);

        // Strip the encryption extension: file_stem() returns the filename without its final extension.
        match src.file_stem() {
            Some(stem) if !stem.is_empty() => dst.set_file_name(stem),
            _ => dst.set_file_name("file"),
        };

        plan.push(PlanEntry {
            src: src.to_path_buf(),
            dst,
        });
    }

    Ok(plan)
}
