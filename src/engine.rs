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

pub fn history(backup_arg: &Path) -> Result<(), Box<dyn Error>> {
    let config = storage::load_config(backup_arg)?;
    let snapshots = storage::list_snapshots(&config.backup)?;

    println!("ChronoBackup — Snapshot History\n");

    if snapshots.is_empty() {
        println!("No snapshots found. Run `chrono backup` first.");
        return Ok(());
    }

    println!("{:<30} {:<20} {}", "Snapshot ID", "Created At", "Files");
    println!("{}", "-".repeat(65));

    for (_, manifest) in &snapshots {
        let created_at = chrono::DateTime::from_timestamp(manifest.created_at as i64, 0)
            .map(|time| {
                time.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .unwrap_or_else(|| "Unknown time".to_string());

        println!(
            "{:<30} {:<20} {}",
            manifest.snapshot_id,
            created_at,
            manifest.files.len()
        );
    }

    println!("\nTotal snapshots: {}", snapshots.len());

    Ok(())
}

pub fn restore(
    backup_arg: &Path,
    snapshot_id: &str,
    overwrite: bool,
) -> Result<(), Box<dyn Error>> {
    let config = storage::load_config(backup_arg)?;
    let snapshots = storage::list_snapshots(&config.backup)?;

    let (snapshot_dir, manifest) = snapshots
        .into_iter()
        .find(|(_, manifest)| manifest.snapshot_id == snapshot_id)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Snapshot '{snapshot_id}' was not found."),
            )
        })?;

    let files_dir = snapshot_dir.join("files");

    let mut restore_files = Vec::new();

    for relative in manifest.files.keys() {
        let relative_path = PathBuf::from(relative);

        if relative_path.as_os_str().is_empty()
            || relative_path
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            return Err(format!("Unsafe path found in snapshot: {relative}").into());
        }

        let source_file = files_dir.join(&relative_path);
        let destination_file = config.source.join(&relative_path);

        let mut parent = config.source.clone();

        if let Some(parent_relative) = relative_path.parent() {
            for component in parent_relative.components() {
                parent.push(component);

                if let Ok(metadata) = fs::symlink_metadata(&parent) {
                    if metadata.file_type().is_symlink() || !metadata.is_dir() {
                        return Err(
                            format!("Unsafe destination directory: {}", parent.display()).into(),
                        );
                    }
                }
            }
        }

        if let Ok(metadata) = fs::symlink_metadata(&destination_file) {
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "Refusing to overwrite a symbolic link: {}",
                    destination_file.display()
                )
                .into());
            }

            if !metadata.is_file() {
                return Err(format!(
                    "Destination is not a regular file: {}",
                    destination_file.display()
                )
                .into());
            }

            if !overwrite {
                return Err(format!(
                    "File already exists: {}. \
                     Use --overwrite to replace existing files.",
                    destination_file.display()
                )
                .into());
            }
        }

        if !source_file.is_file() {
            return Err(format!("Snapshot file is missing: {}", source_file.display()).into());
        }

        restore_files.push((source_file, destination_file));
    }

    for (source_file, destination_file) in &restore_files {
        if let Some(parent) = destination_file.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(source_file, destination_file)?;
        println!("Restored: {}", destination_file.display());
    }

    println!(
        "\nRestore completed. Files restored: {}",
        restore_files.len()
    );

    Ok(())
}

pub fn verify(backup_arg: &Path, snapshot_id: &str) -> Result<(), Box<dyn Error>> {
    let config = storage::load_config(backup_arg)?;
    let snapshots = storage::list_snapshots(&config.backup)?;

    let (snapshot_dir, manifest) = snapshots
        .into_iter()
        .find(|(_, manifest)| manifest.snapshot_id == snapshot_id)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Snapshot '{snapshot_id}' was not found."),
            )
        })?;

    let files_dir = snapshot_dir.join("files");
    let mut verified = 0;
    let mut corrupted = 0;
    let mut missing = 0;

    println!("ChronoBackup — Snapshot Verification\n");

    for (relative_path, expected_hash) in &manifest.files {
        let file_path = files_dir.join(relative_path);

        if !file_path.is_file() {
            println!("MISSING    {relative_path}");
            missing += 1;
            continue;
        }

        let contents = fs::read(&file_path)?;
        let actual_hash = blake3::hash(&contents).to_hex().to_string();

        if &actual_hash == expected_hash {
            println!("OK         {relative_path}");
            verified += 1;
        } else {
            println!("CORRUPTED  {relative_path}");
            corrupted += 1;
        }
    }

    println!("\nVerification summary:");
    println!("Verified:  {verified}");
    println!("Corrupted: {corrupted}");
    println!("Missing:   {missing}");

    if corrupted > 0 || missing > 0 {
        println!("Verification FAILED.");
        return Err(std::io::Error::other("Snapshot integrity check failed.").into());
    }

    println!("Verification PASSED.");
    Ok(())
}
