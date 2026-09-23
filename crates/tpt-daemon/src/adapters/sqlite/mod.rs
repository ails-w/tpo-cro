//! SQLite storage adapter (ADR-002).
//!
//! `:memory:` databases are intentionally unsupported: the store keeps a
//! dedicated reader connection and a writer thread with a connection of its
//! own, so two `:memory:` connections would see two different databases.
//! Callers must pass a file path (tests use temporary files).

mod schema;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;
use tpt_core::error::StoreError;

/// Builds a [`StoreError::Backend`] with a contextual message.
pub(super) fn backend(message: impl Into<String>) -> StoreError {
    StoreError::Backend {
        message: message.into(),
    }
}

/// Converts a `rusqlite` failure into a [`StoreError::Backend`].
pub(super) fn sql_backend(error: rusqlite::Error) -> StoreError {
    backend(format!("sqlite: {error}"))
}

/// A SQLite-backed storage adapter.
pub struct SqliteStore {
    reader: Mutex<Connection>,
}

impl SqliteStore {
    /// Opens (creating it when missing) the database at `path`, applies the
    /// pragmas and runs the migration.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let mut connection = Connection::open(path).map_err(sql_backend)?;
        schema::configure(&connection)?;
        schema::migrate(&mut connection)?;

        Ok(Self {
            reader: Mutex::new(connection),
        })
    }

    /// Schema version recorded in the database header (`PRAGMA user_version`).
    pub fn schema_version(&self) -> Result<i64, StoreError> {
        let connection = self.reader_lock()?;
        schema::schema_version(&connection)
    }

    /// Locks the reader connection, reporting poisoning as a backend failure.
    fn reader_lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, StoreError> {
        self.reader
            .lock()
            .map_err(|_| backend("reader connection mutex is poisoned"))
    }
}

/// Resolves a database path specification and creates its parent directory.
///
/// Supported forms:
/// - `~` or `~/...` expands to `$HOME`.
/// - `$XDG_DATA_HOME` or `$XDG_DATA_HOME/...` expands to the XDG data home,
///   falling back to `$HOME/.local/share` when the variable is unset or empty.
/// - Anything else is used as-is (relative paths stay relative to the cwd).
pub fn resolve_db_path(spec: &str) -> Result<PathBuf, StoreError> {
    let path = expand_db_path(spec)?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|error| {
                backend(format!(
                    "cannot create database directory '{}': {error}",
                    parent.display()
                ))
            })?;
        }
    }
    Ok(path)
}

/// Expands `~` and `$XDG_DATA_HOME` without touching the filesystem.
fn expand_db_path(spec: &str) -> Result<PathBuf, StoreError> {
    if spec == "~" {
        return home_dir();
    }
    if let Some(rest) = spec.strip_prefix("~/") {
        return Ok(home_dir()?.join(rest));
    }
    if spec == "$XDG_DATA_HOME" || spec == "${XDG_DATA_HOME}" {
        return data_home();
    }
    for prefix in ["$XDG_DATA_HOME/", "${XDG_DATA_HOME}/"] {
        if let Some(rest) = spec.strip_prefix(prefix) {
            return Ok(data_home()?.join(rest));
        }
    }
    Ok(PathBuf::from(spec))
}

/// The user home directory, from `$HOME`.
fn home_dir() -> Result<PathBuf, StoreError> {
    let home = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| backend("cannot expand '~': HOME is not set"))?;
    Ok(PathBuf::from(home))
}

/// The XDG data home, falling back to `$HOME/.local/share`.
fn data_home() -> Result<PathBuf, StoreError> {
    match std::env::var_os("XDG_DATA_HOME") {
        Some(value) if !value.is_empty() => Ok(PathBuf::from(value)),
        _ => Ok(home_dir()?.join(".local/share")),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rusqlite::Connection;

    use super::{SqliteStore, expand_db_path, resolve_db_path};

    const TABLES: [&str; 10] = [
        "projects",
        "activities",
        "tags",
        "activity_tags",
        "time_entries",
        "sessions",
        "debts",
        "session_additions",
        "session_gaps",
        "commitment_contracts",
    ];

    const INDEXES: [&str; 9] = [
        "idx_activities_project",
        "idx_activities_archived",
        "idx_entries_activity_day",
        "idx_entries_day",
        "idx_entries_source",
        "idx_sessions_activity",
        "idx_sessions_started",
        "idx_gaps_session",
        "idx_debts_open",
    ];

    fn object_exists(connection: &Connection, kind: &str, name: &str) -> bool {
        connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = ?1 AND name = ?2",
                [kind, name],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
            > 0
    }

    #[test]
    fn store_creates_schema_and_indexes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("metrics.db");

        let store = SqliteStore::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), 1);

        let connection = store.reader.lock().unwrap();
        let journal_mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode.to_lowercase(), "wal");

        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(foreign_keys, 1);

        for table in TABLES {
            assert!(
                object_exists(&connection, "table", table),
                "missing table: {table}"
            );
        }
        for index in INDEXES {
            assert!(
                object_exists(&connection, "index", index),
                "missing index: {index}"
            );
        }
    }

    #[test]
    fn resolve_db_path_creates_missing_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested/dir/metrics.db");

        let resolved = resolve_db_path(&nested.to_string_lossy()).unwrap();

        assert_eq!(resolved, nested);
        assert!(nested.parent().unwrap().is_dir());
    }

    #[test]
    fn resolve_db_path_expands_tilde_and_xdg_data_home() {
        let Some(home) = std::env::var_os("HOME") else {
            return;
        };
        assert_eq!(
            expand_db_path("~/metrics.db").unwrap(),
            Path::new(&home).join("metrics.db")
        );

        let expected_data_home = match std::env::var_os("XDG_DATA_HOME") {
            Some(value) if !value.is_empty() => Path::new(&value).to_path_buf(),
            _ => Path::new(&home).join(".local/share"),
        };
        assert_eq!(
            expand_db_path("$XDG_DATA_HOME/tpt/metrics.db").unwrap(),
            expected_data_home.join("tpt/metrics.db")
        );
    }

    #[test]
    #[ignore = "real-disk validation of `chattr +C` on BTRFS; run explicitly with --ignored"]
    fn real_disk_store_opens_on_compressed_directory() {
        let path = resolve_db_path("~/.local/share/tpt/metrics.db").unwrap();

        let store = SqliteStore::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), 1);
        assert!(path.is_file());
    }
}
