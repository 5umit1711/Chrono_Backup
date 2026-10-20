use crate::models::{Config, Manifest};
use crate::scanner::scan_files;
use crate::storage;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn init(source_arg: &Path, backup_arg: &Path) -> Result<(), Box<dyn Error>> {
    let source = fs::canonicalize(source_arg)?;

    fs::create_dir_all(backup_arg)?;
    let backup = fs::canonicalize(backup_arg)?;

    if backup.starts_with(&source) || source.starts_with(&backup) {
        return Err("Source and backup directories must not contain each other.".into());
    }

    fs::create_dir_all(storage::snapshots_dir(&backup))?;

    let config = Config { source, backup };

    storage::save_config(&config)?;

    println!("ChronoBackup initialized successfully.");
    println!("Source:  {}", config.source.display());
    println!("Backup:  {}", config.backup.display());

    Ok(())
}

pub fn status(backup_arg: &Path) -> Result<(), Box<dyn Error>> {
    let config = storage::load_config(backup_arg)?;
    let current = scan_files(&config.source)?;
    let previous = storage::latest_snapshot(&config.backup)?;

    let previous_files = previous.as_ref().map(|(_, manifest)| &manifest.files);

    let mut added = 0;
    let mut modified = 0;
    let mut unchanged = 0;
    let mut deleted = 0;

    for (path, hash) in &current {
        match previous_files.and_then(|files| files.get(path)) {
            None => {
                added += 1;
                println!("+ {path}  NEW");
            }
            Some(old_hash) if old_hash != hash => {
                modified += 1;
                println!("* {path}  MODIFIED");
            }
            Some(_) => {
                unchanged += 1;
                println!("= {path}  UNCHANGED");
            }
        }
    }

    if let Some((_, manifest)) = previous {
        for path in manifest.files.keys() {
            if !current.contains_key(path) {
                deleted += 1;
                println!("- {path}  DELETED FROM SOURCE");
            }
        }
    }

    println!("\nSummary:");
    println!("Added:     {added}");
    println!("Modified:  {modified}");
    println!("Unchanged: {unchanged}");
    println!("Deleted:   {deleted}");

    if added + modified + deleted == 0 {
        println!("No changes detected.");
    }

    Ok(())
}

pub fn backup(backup_arg: &Path) -> Result<(), Box<dyn Error>> {
    let config = storage::load_config(backup_arg)?;
    let files = scan_files(&config.source)?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let snapshot_id = now.as_nanos().to_string();

    let snapshot_dir = storage::snapshots_dir(&config.backup).join(&snapshot_id);

    let files_dir = snapshot_dir.join("files");
    fs::create_dir_all(&files_dir)?;

    for relative in files.keys() {
        let relative_path = PathBuf::from(relative);
        let source_path = config.source.join(&relative_path);
        let destination_path = files_dir.join(&relative_path);

        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(source_path, destination_path)?;
    }

    let manifest = Manifest {
        snapshot_id: snapshot_id.clone(),
        created_at: now.as_secs(),
        files,
    };

    let json = serde_json::to_string_pretty(&manifest)?;
    fs::write(snapshot_dir.join("manifest.json"), json)?;

    println!("Backup completed successfully.");
    println!("Snapshot ID: {snapshot_id}");
    println!("Files backed up: {}", manifest.files.len());

    Ok(())
}
