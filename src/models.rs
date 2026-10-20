use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub source: PathBuf,
    pub backup: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub snapshot_id: String,
    pub created_at: u64,
    pub files: BTreeMap<String, String>,
}
