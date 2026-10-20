# ChronoBackup

**A snapshot-based file backup and recovery CLI built with Rust.**

ChronoBackup is a command-line utility that helps you back up folders, track file changes, browse backup snapshots, restore previous file versions, and verify backup integrity using BLAKE3 hashing.

The project is built with Rust and Cargo, with a focus on filesystem operations, content hashing, snapshot management, and reliable file recovery.

## Features

- **Directory Initialization:** Configure a source directory and a separate backup directory.
- **File Scanning:** Recursively scan files and calculate their BLAKE3 hashes.
- **Change Detection:** Identify added, modified, unchanged, and deleted files.
- **Snapshot Backups:** Create timestamped snapshots of your source directory.
- **Snapshot History:** List available snapshots, their creation times, and file counts.
- **File Restoration:** Restore files from a selected snapshot, with explicit permission required to overwrite existing files.
- **Integrity Verification:** Compare stored files against their recorded BLAKE3 hashes to detect missing or corrupted files.
- **CLI Interface:** Manage backup operations through simple terminal commands.

## Tech Stack

- **Rust** — Core programming language
- **Cargo** — Dependency management and project builds
- **Clap** — Command-line argument parsing
- **WalkDir** — Recursive directory traversal
- **Serde and Serde JSON** — Configuration and snapshot metadata serialization
- **BLAKE3** — File content hashing
- **Chrono** — Human-readable timestamps

## Prerequisites

Before using ChronoBackup, install:

1. **Rust and Cargo:** Install them using [rustup](https://rustup.rs/).
2. **Git:** Required if you clone the repository from GitHub.

Verify your installation:

```bash
rustc --version
cargo --version
git --version
```

## Installation and Setup

### 1. Clone the repository

Replace the placeholder URL with the URL of your GitHub repository.

```bash
git clone <YOUR_GITHUB_REPOSITORY_URL>
cd ChronoBackup
```

If your repository uses a different folder name, enter that directory instead.

### 2. Build the project

```bash
cargo build
```

Cargo downloads the required dependencies and compiles the project.

### 3. Check available commands

```bash
cargo run -- --help
```

You should see the available commands, including `init`, `status`, `backup`, `history`, `restore`, and `verify`.

## Usage Guide

The examples below use Cargo to run ChronoBackup directly from the source code.

### 1. Initialize a backup

Configure the folder you want to protect and the folder where snapshots will be stored.

```bash
cargo run -- init --source ./MyProject --backup ./MyProject_Backup
```

This creates the backup directory and stores ChronoBackup's configuration and metadata.

**Important:** Keep the source and backup directories separate. ChronoBackup rejects configurations where either directory contains the other.

### 2. Check file changes

```bash
cargo run -- status --backup ./MyProject_Backup
```

This compares the current source files with the latest snapshot and reports:

- `NEW` — A file has been added.
- `MODIFIED` — A file's content has changed.
- `UNCHANGED` — A file matches the latest snapshot.
- `DELETED FROM SOURCE` — A previously backed-up file is no longer in the source directory.

Run this command before creating a backup to review the changes.

### 3. Create a backup snapshot

```bash
cargo run -- backup --backup ./MyProject_Backup
```

ChronoBackup creates a new snapshot containing copies of the current source files and a manifest recording their paths and BLAKE3 hashes.

You can create multiple snapshots as your project evolves.

### 4. View snapshot history

```bash
cargo run -- history --backup ./MyProject_Backup
```

This lists available snapshots, including their snapshot IDs, creation times, and number of files.

Copy the ID of the snapshot you want to inspect, verify, or restore.

### 5. Restore a snapshot

Replace `YOUR_SNAPSHOT_ID` with an ID displayed by the history command.

```bash
cargo run -- restore --backup ./MyProject_Backup --snapshot YOUR_SNAPSHOT_ID
```

By default, restoration refuses to overwrite existing files.

If you intentionally want to replace existing files with their backed-up versions, use:

```bash
cargo run -- restore --backup ./MyProject_Backup --snapshot YOUR_SNAPSHOT_ID --overwrite
```

**Warning:** The `--overwrite` flag replaces matching destination files. Review the selected snapshot and test restoration on disposable data before using it with important files.

Restoration does not automatically delete files that were added to the source directory after the selected snapshot was created.

### 6. Verify snapshot integrity

```bash
cargo run -- verify --backup ./MyProject_Backup --snapshot YOUR_SNAPSHOT_ID
```

ChronoBackup recalculates the BLAKE3 hash of each file listed in the selected snapshot's manifest and compares it with the recorded hash.

- `OK` — The file matches its recorded hash.
- `CORRUPTED` — The file's content differs from the recorded hash.
- `MISSING` — The file is missing from the snapshot.

The command reports whether the integrity check passed or failed.

## Example Workflow

A typical workflow looks like this:

```bash
# Initialize the backup configuration
cargo run -- init --source ./MyProject --backup ./MyProject_Backup

# Review current changes
cargo run -- status --backup ./MyProject_Backup

# Create a snapshot
cargo run -- backup --backup ./MyProject_Backup

# View saved snapshots
cargo run -- history --backup ./MyProject_Backup

# Verify a selected snapshot
cargo run -- verify --backup ./MyProject_Backup --snapshot YOUR_SNAPSHOT_ID

# Restore a selected snapshot when needed
cargo run -- restore --backup ./MyProject_Backup --snapshot YOUR_SNAPSHOT_ID
```

After making changes to your source project, run `status` to inspect the differences and `backup` to save a new snapshot.

## Project Structure

```text
ChronoBackup/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── .gitignore
└── src/
    ├── main.rs
    ├── cli.rs
    ├── models.rs
    ├── scanner.rs
    ├── storage.rs
    └── engine.rs
```

| File | Responsibility |
|---|---|
| `main.rs` | Application entry point and command dispatch |
| `cli.rs` | CLI commands and argument definitions |
| `models.rs` | Configuration and snapshot manifest structures |
| `scanner.rs` | Recursive file scanning and BLAKE3 hashing |
| `storage.rs` | Configuration persistence and snapshot metadata |
| `engine.rs` | Initialization, status, backup, history, restore, and verification logic |

## Backup Storage Structure

ChronoBackup stores its metadata and snapshots inside the configured backup directory.

```text
MyProject_Backup/
└── .chronobackup/
    ├── config.json
    └── snapshots/
        └── <snapshot-id>/
            ├── manifest.json
            └── files/
                ├── example.txt
                └── docs/
                    └── readme.md
```

- `config.json` stores the configured source and backup paths.
- `manifest.json` records the snapshot ID, creation time, file paths, and hashes.
- `files/` contains copies of the files saved in that snapshot.

Keep the `.chronobackup` directory intact. It contains the metadata needed to discover, verify, and restore snapshots.

## Current Limitations

- Each snapshot stores full file copies; incremental backups and content deduplication are not implemented.
- Restoration does not remove files that are absent from the selected snapshot.
- Integrity verification checks files listed in the manifest; it does not currently report every unexpected extra file.
- A failure during a multi-file restore can leave some files restored before the operation stops.
- Keep independent copies of critical data. ChronoBackup should not be your only backup for irreplaceable files.

## Future Improvements

Potential future enhancements include:

- Incremental backups and content deduplication
- Automated backup scheduling
- Live filesystem monitoring
- Safer, transactional restoration
- Additional integrity and recovery safeguards
- Improved terminal output and an interactive terminal UI
- Automated cross-platform release builds

## Contributing

Contributions, bug reports, and feature suggestions are welcome.

1. Fork the repository.
2. Create a feature branch.
3. Make your changes.
4. Open a pull request describing the changes.

## License

Choose an open-source license before publishing the project. For example, the MIT License is a common option for personal developer tools. If you choose MIT, add a `LICENSE` file containing the appropriate license text.

---

**ChronoBackup — Back up your files. Track your changes. Restore with confidence.**