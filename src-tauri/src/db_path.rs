//! Database location and private filesystem creation.
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
pub const RELEASE_DIR: &str = "dev.nandomoreira.jobradar";
pub const DEBUG_DIR: &str = "dev.nandomoreira.jobradar-local";
pub const DB_FILE: &str = "job-radar.db";
pub const ENV_VAR: &str = "JOB_RADAR_DB";
pub const CONFIG_FILE: &str = "crawler.json";
/// A blank override is ignored; any other value is kept byte for byte, Unicode or not.
pub fn db_path(env_override: Option<&OsStr>, data_dir: &Path, debug: bool) -> PathBuf {
    let is_blank = |value: &&OsStr| value.to_str().is_some_and(|v| v.trim().is_empty());
    match env_override.filter(|value| !value.is_empty() && !is_blank(value)) {
        Some(path) => PathBuf::from(path),
        None => app_dir(data_dir, debug).join(DB_FILE),
    }
}
/// The app's own folder, kept apart between debug and release builds.
pub fn app_dir(data_dir: &Path, debug: bool) -> PathBuf {
    data_dir.join(if debug { DEBUG_DIR } else { RELEASE_DIR })
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
            db_path(Some(OsStr::new("/tmp/scratch.db")), data, true),
            Path::new("/tmp/scratch.db")
        );
        assert_eq!(
            db_path(Some(OsStr::new("   ")), data, false),
            Path::new("/data/dev.nandomoreira.jobradar/job-radar.db")
        );
    }
    #[cfg(unix)]
    #[test]
    fn env_override_keeps_a_non_unicode_path() {
        use std::os::unix::ffi::OsStrExt;
        let raw = OsStr::from_bytes(b"/tmp/caf\xe9.db");
        assert_eq!(
            db_path(Some(raw), Path::new("/data"), false),
            Path::new(raw)
        );
    }
    #[test]
    fn release_dir_matches_the_tauri_identifier() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(config["identifier"], RELEASE_DIR);
    }
}
