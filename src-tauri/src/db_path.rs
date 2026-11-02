//! Database location and private filesystem creation.
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
pub const RELEASE_DIR: &str = "dev.nandomoreira.jobradar";
pub const DEBUG_DIR: &str = "dev.nandomoreira.jobradar-local";
pub const DB_FILE: &str = "job-radar.db";
pub const ENV_VAR: &str = "JOB_RADAR_DB";
pub fn db_path(env_override: Option<&str>, data_dir: &Path, debug: bool) -> PathBuf {
    match env_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(path) => PathBuf::from(path),
        None => data_dir
            .join(if debug { DEBUG_DIR } else { RELEASE_DIR })
            .join(DB_FILE),
    }
}

pub fn db_path_os(env_override: Option<&OsStr>, data_dir: &Path, debug: bool) -> PathBuf {
    match env_override.filter(|value| !value.is_empty()) {
        Some(path) => PathBuf::from(path),
        None => data_dir
            .join(if debug { DEBUG_DIR } else { RELEASE_DIR })
            .join(DB_FILE),
    }
}
pub fn prepare(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        create_private_dir(parent)?;
    }
    match create_private_file(path) {
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        other => other,
    }
}
#[cfg(unix)]
fn create_private_dir(dir: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
}
#[cfg(not(unix))]
fn create_private_dir(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)
}
#[cfg(unix)]
fn create_private_file(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map(drop)
}
#[cfg(not(unix))]
fn create_private_file(path: &Path) -> io::Result<()> {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map(drop)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picks_the_directory_for_the_build() {
        let data = Path::new("/data/share");
        assert_eq!(
            db_path(None, data, false),
            Path::new("/data/share/dev.nandomoreira.jobradar/job-radar.db")
        );
        assert_eq!(
            db_path(None, data, true),
            Path::new("/data/share/dev.nandomoreira.jobradar-local/job-radar.db")
        );
    }
    #[test]
    fn env_override_wins_unless_empty() {
        let data = Path::new("/data");
        assert_eq!(
            db_path(Some("/tmp/scratch.db"), data, true),
            Path::new("/tmp/scratch.db")
        );
        assert_eq!(
            db_path(Some("   "), data, false),
            Path::new("/data/dev.nandomoreira.jobradar/job-radar.db")
        );
    }
    #[test]
    fn release_dir_matches_the_tauri_identifier() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(config["identifier"], RELEASE_DIR);
    }
}
