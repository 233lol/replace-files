use clap::Parser;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Find and replace files matching an old file's content (by hash) with a new file.
///
/// Scans the target directory recursively for files that have the same
/// filename, file size, and content hash as the OLD file, then replaces
/// them with a copy of the NEW file.
///
/// Usage: replace-files <OLD_FILE> <NEW_FILE> <TARGET_DIR> [--dry-run]
#[derive(Parser, Debug)]
#[command(name = "replace-files", verbatim_doc_comment)]
struct Args {
    /// Path to the old file (used as the matching template)
    old: PathBuf,

    /// Path to the new file (replacement content)
    new: PathBuf,

    /// Target directory to scan and replace files in
    target: PathBuf,

    /// Dry run: list files that would be replaced but do not actually replace
    #[arg(long)]
    dry_run: bool,
}

fn compute_hash(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn main() {
    let args = Args::parse();

    // Validate old file
    if !args.old.is_file() {
        eprintln!(
            "Error: old file '{}' is not a file or does not exist.",
            args.old.display()
        );
        std::process::exit(1);
    }

    // Validate new file
    if !args.new.is_file() {
        eprintln!(
            "Error: new file '{}' is not a file or does not exist.",
            args.new.display()
        );
        std::process::exit(1);
    }

    // Validate target directory
    if !args.target.is_dir() {
        eprintln!(
            "Error: target '{}' is not a directory or does not exist.",
            args.target.display()
        );
        std::process::exit(1);
    }

    // Get old file name and size (used as matching criteria)
    let old_name = match args.old.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            eprintln!("Error: could not determine old file name.");
            std::process::exit(1);
        }
    };

    let old_size = match fs::metadata(&args.old) {
        Ok(meta) => meta.len(),
        Err(e) => {
            eprintln!("Error: could not read old file metadata: {}", e);
            std::process::exit(1);
        }
    };

    // Pre-compute old file hash
    let old_hash = match compute_hash(&args.old) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let mut replaced_count = 0u64;
    let mut matched_count = 0u64;

    for entry in WalkDir::new(&args.target).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();

        // Skip directories
        if !entry_path.is_file() {
            continue;
        }

        // Skip the new file itself if it happens to be inside the target directory
        if entry_path == args.new {
            continue;
        }

        let entry_name = match entry_path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Step 1: Check filename match against the OLD file name
        if entry_name != old_name {
            continue;
        }

        // Step 2: Check file size match (fast filter)
        let entry_size = match fs::metadata(entry_path) {
            Ok(meta) => meta.len(),
            Err(_) => continue,
        };

        if entry_size != old_size {
            continue;
        }

        // Step 3: Compute hash and compare against the OLD file hash
        let entry_hash = match compute_hash(entry_path) {
            Ok(hash) => hash,
            Err(e) => {
                eprintln!("Warning: could not hash {}: {}", entry_path.display(), e);
                continue;
            }
        };

        if entry_hash != old_hash {
            continue;
        }

        matched_count += 1;

        // File matches — replace it with the NEW file
        if args.dry_run {
            println!("[DRY RUN] Would replace: {};Size: {}", entry_path.display(),entry_size);
        } else {
            match fs::copy(&args.new, entry_path) {
                Ok(_) => {
                    println!("Replaced: {}", entry_path.display());
                    replaced_count += 1;
                }
                Err(e) => {
                    eprintln!(
                        "Error: failed to replace {}: {}",
                        entry_path.display(),
                        e
                    );
                }
            }
        }
    }

    if args.dry_run {
        println!(
            "\nDry run complete. {} file(s) would be replaced.",
            matched_count
        );
    } else {
        println!(
            "\nDone. {} file(s) replaced (out of {} matching).",
            replaced_count, matched_count
        );
    }
}