use crate::options::upload::{Provider, UploadArgs};
use anyhow::{Context, Result, anyhow, bail};
use base64::Engine;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::StatusCode;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

#[derive(Debug)]
struct UploadFile {
    local_path: PathBuf,
    remote_path: String,
    size: u64,
}


fn fmt_gh_err(e: &octocrab::Error) -> String {
    match e {
        octocrab::Error::GitHub { source, .. } => {
            let status = source.status_code;
            let msg = &source.message;
            let docs = source
                .documentation_url
                .as_deref()
                .map(|u| format!(", docs: {}", u))
                .unwrap_or_default();
            format!("GitHub: {} (status: {:?}){}", msg, status, docs)
        }
        other => other.to_string(),
    }
}

/// Synchronous entrypoint (main calls this).
/// Internally spins a Tokio runtime for async GitHub calls.
pub fn run(args: UploadArgs) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().context("failed to init tokio runtime")?;
    rt.block_on(run_inner(args))
}

async fn run_inner(args: UploadArgs) -> Result<()> {
    if !args.input_dir.exists() {
        bail!("directory '{}' does not exist", args.input_dir.display());
    }
    if !args.input_dir.is_dir() {
        bail!("'{}' is not a directory", args.input_dir.display());
    }

    let files = collect_encrypted_files(&args.input_dir, args.verbose)?;
    if files.is_empty() {
        println!("{}", "No .lock files found to upload".yellow());
        return Ok(());
    }

    let total_bytes: u64 = files.iter().map(|f| f.size).sum();
    println!(
        "{} {} files ({:.2} MiB) to upload",
        "Found".green().bold(),
        files.len(),
        total_bytes as f64 / (1024.0 * 1024.0)
    );

    match args.provider {
        Provider::Github => upload_to_github(files, &args).await?,
        Provider::Drive => bail!("Google Drive provider is not yet implemented"),
        Provider::Onedrive => bail!("Microsoft OneDrive provider is not yet implemented"),
    }

    Ok(())
}

fn collect_encrypted_files(input_dir: &Path, verbose: bool) -> Result<Vec<UploadFile>> {
    let root = fs::canonicalize(input_dir)
        .with_context(|| format!("canonicalizing {}", input_dir.display()))?;

    // count files to estimate capacity
    let file_count = if verbose {
        println!("Scanning directory for .lock files...");
        WalkDir::new(&root)
            .follow_links(false)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry.path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("lock"))
                    .unwrap_or(false)
            })
            .count()
    } else {
        WalkDir::new(&root)
            .follow_links(false)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry.path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("lock"))
                    .unwrap_or(false)
            })
            .count()
    };

    // Pre-allocate with actual file count + small buffer for safety
    let capacity = file_count.max(lock::DEFAULT_VEC_CAPACITY); // At least default capacity
    if verbose && file_count > lock::LARGE_VEC_THRESHOLD {
        println!("Found {} files, pre-allocating memory for optimal performance", file_count);
    }
    let mut files = Vec::with_capacity(capacity);

    for entry in WalkDir::new(&root).follow_links(false) {
        let entry = entry?;

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        if !path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("lock"))
            .unwrap_or(false)
        {
            continue;
        }

        let rel_path = path
            .strip_prefix(&root)
            .with_context(|| format!("computing relative path for {}", path.display()))?;

        let remote_path = rel_path.to_string_lossy().replace('\\', "/");
        let metadata = fs::metadata(path)?;

        files.push(UploadFile {
            local_path: path.to_path_buf(),
            remote_path,
            size: metadata.len(),
        });
    }

    Ok(files)
}

async fn upload_to_github(files: Vec<UploadFile>, args: &UploadArgs) -> Result<()> {
    // Prefer CLI arg; fall back to env var.
    let token = args
        .github_token
        .clone()
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
        .ok_or_else(|| {
            anyhow!("GitHub token is required (use --github-token or set GITHUB_TOKEN)")
        })?;

    let repo_str = args
        .github_repo
        .as_ref()
        .ok_or_else(|| anyhow!("--github-repo is required and must be OWNER/REPO"))?;

    let (owner, repo) = parse_owner_repo(repo_str)
        .ok_or_else(|| anyhow!("--github-repo must be in OWNER/REPO form"))?;

    let gh = octocrab::Octocrab::builder()
        .personal_token(token)
        .build()
        .context("failed to initialize GitHub client")?;

    // only own personal (user-owned) repos allowed (no organizations).
    // Public or private both allowed.
    let me = gh
        .current()
        .user()
        .await
        .map_err(|e| anyhow!("failed to fetch authenticated user: {}", fmt_gh_err(&e)))?;
    if me.login != owner {
        bail!(
            "upload blocked: target OWNER '{}' is not the authenticated user '{}'. \
             Only your own personal repos are allowed.",
            owner,
            me.login
        );
    }

    let repo_meta = gh.repos(&owner, &repo).get().await.map_err(|e| {
        anyhow!(
            "failed to read repo metadata for {}/{}: {}",
            owner,
            repo,
            fmt_gh_err(&e)
        )
    })?;

    // owner type must be "User" (not "Organization")
    if let Some(own) = &repo_meta.owner
        && own.r#type != "User" {
        bail!(
            "upload blocked: {}/{} belongs to an organization. Only personal user repos are allowed.",
            owner,
            repo
        );
    }

    // // repo must be private
    // if !repo_meta.private.unwrap_or(false) {
    //     bail!(
    //         "upload blocked: {}/{} is public. Only private repos are allowed.",
    //         owner,
    //         repo
    //     );
    // }
    if !repo_meta.private.unwrap_or(false) {
        println!(
            "{}",
            "Warning: Target repo is PUBLIC. Your .lock files (ciphertext) will be visible to everyone."
                .yellow()
        );
    }

    println!(
        "{} Uploading to {}/{} (branch: {})",
        "Info:".cyan().bold(),
        owner,
        repo,
        args.github_branch
    );

    let total_files = files.len();
    let total_bytes: u64 = files.iter().map(|f| f.size).sum();
    let start_time = Instant::now();
    
    let mut uploaded = 0usize;
    let mut failed = 0usize;
    let mut uploaded_bytes = 0u64;

    if args.verbose {
        // Verbose mode: show each file upload
        for (index, file) in files.iter().enumerate() {
            let current = index + 1;
            match upload_file_contents_api(
                &gh,
                &owner,
                &repo,
                &args.github_branch,
                &file.local_path,
                &file.remote_path,
            )
            .await
            {
                Ok(_) => {
                    uploaded += 1;
                    uploaded_bytes += file.size;
                    let uploaded_mb = uploaded_bytes as f64 / (1024.0 * 1024.0);
                    let remaining_mb = (total_bytes - uploaded_bytes) as f64 / (1024.0 * 1024.0);
                    println!(
                        "[{}/{}] ok: {} -> {} ({:.2} MB/{:.2} MB)",
                        current,
                        total_files,
                        file.local_path.display(),
                        file.remote_path,
                        uploaded_mb,
                        remaining_mb
                    );
                }
                Err(e) => {
                    failed += 1;
                    eprintln!(
                        "[{}/{}] {} {}: {}",
                        current,
                        total_files,
                        "Error".red().bold(),
                        file.remote_path,
                        e
                    );
                }
            }
        }
    } else {
        // Non-verbose mode: show smooth continuous progress bar with MB tracking
        let pb = ProgressBar::new(total_bytes);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({msg})")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );

        // Shared state for continuous progress
        let progress_state = Arc::new(Mutex::new((
            0u64,    // current_progress
            0u64,    // current_file_progress
            0usize,  // current_file_index
            0u64,    // current_file_size
            0usize,  // uploaded_files
            std::time::Instant::now(), // file_start_time
        )));

        // Start continuous progress update thread
        let pb_clone = pb.clone();
        let progress_state_clone = progress_state.clone();
        let total_files = files.len();
        let progress_handle = std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(50)); // Reduced frequency
                let now = std::time::Instant::now();
                
                let (current_progress, _current_file_progress, current_file_index, current_file_size, uploaded_files, file_start_time) = {
                    let state = progress_state_clone.lock().unwrap();
                    (state.0, state.1, state.2, state.3, state.4, state.5)
                };
                
                if current_file_index >= total_files {
                    break;
                }
                
                // Calculate smooth progress for current file
                let mut new_file_progress = 0u64;
                if current_file_size > 0 {
                    let elapsed = now.duration_since(file_start_time).as_secs_f64();
                    
                    // Simplified progress calculation for better performance
                    let progress_ratio = if elapsed < 0.1 {
                        elapsed * 2.0  // Quick start
                    } else if elapsed < 0.5 {
                        0.2 + (elapsed - 0.1) * 1.5  // Fast middle
                    } else {
                        0.8 + (elapsed - 0.5) * 0.4  // Quick finish
                    };
                    
                    new_file_progress = (current_file_size as f64 * progress_ratio.min(1.0)) as u64;
                }
                
                let total_progress = current_progress + new_file_progress;
                let remaining_files = total_files - uploaded_files;
                
                pb_clone.set_position(total_progress);
                pb_clone.set_message(format!("{}/{}", uploaded_files, remaining_files));
            }
        });

        // Upload files sequentially to avoid GitHub API conflicts
        for (index, file) in files.iter().enumerate() {
            // Update current file info for progress thread
            {
                let mut state = progress_state.lock().unwrap();
                state.2 = index; // current_file_index
                state.3 = file.size; // current_file_size
                state.5 = std::time::Instant::now(); // file_start_time
            }

            // Perform the actual upload
            match upload_file_contents_api(
                &gh,
                &owner,
                &repo,
                &args.github_branch,
                &file.local_path,
                &file.remote_path,
            )
            .await
            {
                Ok(_) => {
                    uploaded += 1;
                    uploaded_bytes += file.size;
                    let mut state = progress_state.lock().unwrap();
                    state.0 += file.size; // current_progress
                    state.4 += 1; // uploaded_files
                }
                Err(e) => {
                    failed += 1;
                    let mut state = progress_state.lock().unwrap();
                    state.0 += file.size; // Still count as processed
                    eprintln!("\n{} {}: {}", "Error".red().bold(), file.remote_path, e);
                }
            }
        }

        // Signal progress thread to stop
        {
            let mut state = progress_state.lock().unwrap();
            state.2 = total_files; // current_file_index
        }
        let _ = progress_handle.join();

        pb.finish_and_clear();
    }

    let elapsed = start_time.elapsed();
    let uploaded_mb = uploaded_bytes as f64 / (1024.0 * 1024.0);
    
    println!(
        "{} {} uploaded, {} failed ({:.2} MiB in {:.2}s)",
        "Completed:".green().bold(),
        uploaded,
        failed,
        uploaded_mb,
        elapsed.as_secs_f64()
    );

    if failed > 0 {
        bail!("{} file(s) failed to upload", failed);
    }

    Ok(())
}

async fn upload_file_contents_api(
    gh: &octocrab::Octocrab,
    owner: &str,
    repo: &str,
    branch: &str,
    local_path: &Path,
    remote_path: &str,
) -> Result<()> {
    // Read and base64-encode (Contents API requires base64).
    let content = fs::read(local_path)
        .with_context(|| format!("reading {}", local_path.display()))?;
    
    let content_base64 = base64::engine::general_purpose::STANDARD.encode(&content);

    let existing = gh
        .repos(owner, repo)
        .get_content()
        .path(remote_path)
        .r#ref(branch)
        .send()
        .await;

    // When the path points to a file, the list contains one item with type == "file".
    let maybe_sha = match existing {
        Ok(mut list) => {
            let items = list.take_items();
            items
                .into_iter()
                .find(|c| c.r#type == "file")
                .map(|c| c.sha)
        }
        Err(e) => {
            if let octocrab::Error::GitHub { source, .. } = &e {
                if source.status_code != StatusCode::NOT_FOUND {
                    eprintln!("{} {}", "Lookup warning:".yellow().bold(), fmt_gh_err(&e));
                }
            } else {
                eprintln!("{} {}", "Lookup warning:".yellow().bold(), e);
            }
            None
        }
    };

    let message = if maybe_sha.is_some() {
        format!("Update {remote_path}")
    } else {
        format!("Add {remote_path}")
    };

    if let Some(sha) = maybe_sha {
        // UPDATE path
        if let Err(e) = gh
            .repos(owner, repo)
            .update_file(remote_path, &message, &content_base64, &sha)
            .branch(branch)
            .send()
            .await
        {
            return Err(anyhow!(
                "updating {} on branch {}: {}",
                remote_path,
                branch,
                fmt_gh_err(&e)
            ));
        }
    } else {
        // CREATE path
        if let Err(e) = gh
            .repos(owner, repo)
            .create_file(remote_path, &message, &content_base64)
            .branch(branch)
            .send()
            .await
        {
            return Err(anyhow!(
                "creating {} on branch {}: {}",
                remote_path,
                branch,
                fmt_gh_err(&e)
            ));
        }
    }

    Ok(())
}

fn parse_owner_repo(s: &str) -> Option<(String, String)> {
    let (owner, repo) = s.split_once('/')?;
    Some((owner.to_string(), repo.to_string()))
}
