use crate::models::{Config, Manifest};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub fn metadata_dir(backup: &Path) -> PathBuf {
    backup.join(".chronobackup")
}

pub fn snapshots_dir(backup: &Path) -> PathBuf {
    metadata_dir(backup).join("snapshots")
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn Error>> {
    let directory = metadata_dir(&config.backup);
    fs::create_dir_all(&directory)?;

    let path = directory.join("config.json");
    let json = serde_json::to_string_pretty(config)?;

    fs::write(path, json)?;
    Ok(())
}

pub fn load_config(backup_arg: &Path) -> Result<Config, Box<dyn Error>> {
    let backup = fs::canonicalize(backup_arg)?;

    let path = metadata_dir(&backup).join("config.json");
    let json = fs::read_to_string(path)?;

    let config: Config = serde_json::from_str(&json)?;
    Ok(config)
}

pub fn latest_snapshot(backup: &Path) -> Result<Option<(PathBuf, Manifest)>, Box<dyn Error>> {
    let directory = snapshots_dir(backup);
    let mut paths = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            paths.push(entry.path());
        }
    }

    paths.sort();

    for path in paths.into_iter().rev() {
        let manifest_path = path.join("manifest.json");

        // Ignore incomplete snapshots without a manifest.
        if !manifest_path.is_file() {
            continue;
        }

        let json = fs::read_to_string(&manifest_path)?;
        let manifest: Manifest = serde_json::from_str(&json)?;

        return Ok(Some((path, manifest)));
    }

    Ok(None)
}
