//! SQLite backend. Schema changes are append-only migrations.
use crate::store::StoreError;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

pub const MIGRATIONS: &[&str] = &[include_str!("../migrations/001_init.sql")];

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        StoreError::Storage(error.to_string())
    }
}

pub struct SqliteStore {
    #[allow(dead_code)]
    conn: Mutex<Connection>,
}
impl SqliteStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        Self::init(Connection::open(path)?)
    }
    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }
    fn init(mut conn: Connection) -> Result<Self, StoreError> {
        conn.pragma_update(None, "foreign_keys", true)?;
        if conn
            .pragma_query_value(None, "journal_mode", |row| row.get::<_, String>(0))
            .unwrap_or_default()
            != "memory"
        {
            let _ = conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| {
                row.get::<_, String>(0)
            });
        }
        conn.busy_timeout(Duration::from_secs(5))?;
        migrate(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    #[allow(dead_code)]
    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Connection>, StoreError> {
        self.conn.lock().map_err(|_| StoreError::Poisoned)
    }
}
fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))?;
    let current = usize::try_from(current).unwrap_or(usize::MAX);
    if current > MIGRATIONS.len() {
        return Err(StoreError::Storage(format!(
            "database schema v{current} is newer than this app supports"
        )));
    }
    if current == MIGRATIONS.len() {
        return Ok(());
    }
    let tx = conn.transaction()?;
    for sql in &MIGRATIONS[current..] {
        tx.execute_batch(sql)?;
    }
    tx.pragma_update(None, "user_version", MIGRATIONS.len() as i64)?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    fn user_version(path: &Path) -> i64 {
        Connection::open(path)
            .unwrap()
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap()
    }
    #[test]
    fn migrates_once_to_the_latest_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("job-radar.db");
        drop(SqliteStore::open(&path).unwrap());
        assert_eq!(user_version(&path), MIGRATIONS.len() as i64);
        drop(SqliteStore::open(&path).unwrap());
        assert_eq!(user_version(&path), MIGRATIONS.len() as i64);
    }
    #[test]
    fn refuses_a_newer_schema_without_touching_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("job-radar.db");
        Connection::open(&path)
            .unwrap()
            .pragma_update(None, "user_version", 99)
            .unwrap();
        let Err(StoreError::Storage(detail)) = SqliteStore::open(&path) else {
            panic!("a newer schema must be refused")
        };
        assert!(detail.contains("newer"));
        assert_eq!(user_version(&path), 99);
        let tables: i64 = Connection::open(&path)
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tables, 0);
    }
}
