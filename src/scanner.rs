use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn scan_files(root: &Path) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    let mut files = BTreeMap::new();

    for entry in WalkDir::new(root) {
        let entry = entry?;

        if !entry.file_type().is_file() {
            continue;
        }

        let relative_path = entry.path().strip_prefix(root)?;

        let relative_path = relative_path.to_string_lossy().replace('\\', "/");

        let contents = fs::read(entry.path())?;

        let hash = blake3::hash(&contents).to_hex().to_string();

        files.insert(relative_path, hash);
    }

    Ok(files)
}
