//! SQLite storage adapter (ADR-002).
//!
//! `:memory:` databases are intentionally unsupported: the store keeps a
//! dedicated reader connection and a writer thread with a connection of its
//! own, so two `:memory:` connections would see two different databases.
//! Callers must pass a file path (tests use temporary files).

mod mapping;
mod schema;
mod writer;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender, SyncSender};
use std::sync::{Mutex, MutexGuard};
use std::thread::JoinHandle;

use rusqlite::Connection;
use tpt_core::domain::{Activity, CommitmentContract, Debt, Session, SessionGap, TimeEntry};
use tpt_core::error::StoreError;
use tpt_core::ports::Store;

use writer::WriteCommand;

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

/// Tables whose primary keys the store assigns client-side.
#[derive(Clone, Copy)]
enum RecordKind {
    Activity,
    TimeEntry,
    Session,
    Gap,
    Debt,
    Contract,
}

/// Number of id counters, one per [`RecordKind`].
const RECORD_KINDS: usize = 6;

impl RecordKind {
    const ALL: [Self; RECORD_KINDS] = [
        Self::Activity,
        Self::TimeEntry,
        Self::Session,
        Self::Gap,
        Self::Debt,
        Self::Contract,
    ];

    const fn index(self) -> usize {
        match self {
            Self::Activity => 0,
            Self::TimeEntry => 1,
            Self::Session => 2,
            Self::Gap => 3,
            Self::Debt => 4,
            Self::Contract => 5,
        }
    }

    const fn table(self) -> &'static str {
        match self {
            Self::Activity => "activities",
            Self::TimeEntry => "time_entries",
            Self::Session => "sessions",
            Self::Gap => "session_gaps",
            Self::Debt => "debts",
            Self::Contract => "commitment_contracts",
        }
    }
}

/// A SQLite-backed storage adapter.
///
/// Writes are stamped and enqueued without I/O; a dedicated thread owns the
/// write connection and applies them serially. Reads use a separate connection
/// guarded by a mutex, so a WAL database allows them to proceed while the
/// writer works.
///
/// The store assumes a single writer process (the daemon). Ids are assigned
/// client-side from per-table counters seeded with `MAX(id)` at open; an id is
/// never reused, not even when the write carrying it fails. Because write
/// failures surface asynchronously, the daemon must treat a failing [`Store::flush`]
/// as a failure condition.
pub struct SqliteStore {
    writer: SyncSender<WriteCommand>,
    writer_handle: Option<JoinHandle<()>>,
    reader: Mutex<Connection>,
    ids: [i64; RECORD_KINDS],
}

impl SqliteStore {
    /// Opens (creating it when missing) the database at `path`, applies the
    /// pragmas and runs the migration.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let mut reader = Connection::open(path).map_err(sql_backend)?;
        schema::configure(&reader)?;
        schema::migrate(&mut reader)?;
        let ids = seed_ids(&reader)?;

        let writer_connection = Connection::open(path).map_err(sql_backend)?;
        schema::configure(&writer_connection)?;
        let version = schema::schema_version(&writer_connection)?;
        if version != schema::SCHEMA_VERSION {
            return Err(backend(format!(
                "writer connection sees schema version {version}, expected {}",
                schema::SCHEMA_VERSION
            )));
        }

        let (writer, commands) = mpsc::sync_channel(writer::QUEUE_CAPACITY);
        let writer_handle = std::thread::Builder::new()
            .name("tpt-store-writer".to_owned())
            .spawn(move || writer::run(writer_connection, commands))
            .map_err(|error| backend(format!("cannot spawn the store writer thread: {error}")))?;

        Ok(Self {
            writer,
            writer_handle: Some(writer_handle),
            reader: Mutex::new(reader),
            ids,
        })
    }

    /// Schema version recorded in the database header (`PRAGMA user_version`).
    pub fn schema_version(&self) -> Result<i64, StoreError> {
        let connection = self.reader_lock()?;
        schema::schema_version(&connection)
    }

    /// Truncates the WAL and reclaims free pages.
    pub fn checkpoint(&self) -> Result<(), StoreError> {
        self.request(WriteCommand::Checkpoint)
    }

    /// Assigns (or validates) the id of a record about to be written.
    ///
    /// An explicit id advances the counter past itself, so a later `id: None`
    /// can never reuse it.
    fn allocate_id(&mut self, kind: RecordKind, explicit: Option<i64>) -> i64 {
        let counter = &mut self.ids[kind.index()];
        match explicit {
            Some(id) => {
                *counter = (*counter).max(id);
                id
            }
            None => {
                *counter += 1;
                *counter
            }
        }
    }

    fn enqueue(&self, command: WriteCommand) -> Result<(), StoreError> {
        self.writer
            .send(command)
            .map_err(|_| backend("store writer thread is not running"))
    }

    /// Sends a command and waits for its reply.
    fn request<T>(
        &self,
        command: impl FnOnce(Sender<Result<T, StoreError>>) -> WriteCommand,
    ) -> Result<T, StoreError> {
        let (sender, receiver) = mpsc::channel();
        self.enqueue(command(sender))?;
        receiver
            .recv()
            .map_err(|_| backend("store writer thread dropped the reply channel"))?
    }

    /// Locks the reader connection, reporting poisoning as a backend failure.
    fn reader_lock(&self) -> Result<MutexGuard<'_, Connection>, StoreError> {
        self.reader
            .lock()
            .map_err(|_| backend("reader connection mutex is poisoned"))
    }

    /// Stops the writer thread so tests can exercise the dead-thread path.
    #[cfg(test)]
    fn stop_writer_for_test(&mut self) {
        let _ = self.enqueue(WriteCommand::Shutdown);
        if let Some(handle) = self.writer_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Seeds the per-table id counters from the highest stored id.
fn seed_ids(connection: &Connection) -> Result<[i64; RECORD_KINDS], StoreError> {
    let mut ids = [0_i64; RECORD_KINDS];
    for kind in RecordKind::ALL {
        let sql = format!("SELECT COALESCE(MAX(id), 0) FROM {}", kind.table());
        ids[kind.index()] = connection
            .query_row(&sql, [], |row| row.get(0))
            .map_err(sql_backend)?;
    }
    Ok(ids)
}

impl Store for SqliteStore {
    fn save_activity(&mut self, activity: &Activity) -> Result<i64, StoreError> {
        let mut record = activity.clone();
        let id = self.allocate_id(RecordKind::Activity, record.id);
        record.id = Some(id);
        self.enqueue(WriteCommand::SaveActivity(record))?;
        Ok(id)
    }

    fn load_activity(&self, id: i64) -> Result<Option<Activity>, StoreError> {
        let connection = self.reader_lock()?;
        mapping::load_activity(&connection, id)
    }

    fn list_activities(&self, include_archived: bool) -> Result<Vec<Activity>, StoreError> {
        let connection = self.reader_lock()?;
        mapping::list_activities(&connection, include_archived)
    }

    fn save_time_entry(&mut self, entry: &TimeEntry) -> Result<i64, StoreError> {
        let mut record = entry.clone();
        let id = self.allocate_id(RecordKind::TimeEntry, record.id);
        record.id = Some(id);
        self.enqueue(WriteCommand::SaveTimeEntry(record))?;
        Ok(id)
    }

    fn list_time_entries(&self, activity_id: i64) -> Result<Vec<TimeEntry>, StoreError> {
        let connection = self.reader_lock()?;
        mapping::list_time_entries(&connection, activity_id)
    }

    fn save_session(&mut self, session: &Session) -> Result<i64, StoreError> {
        let mut record = session.clone();
        let id = self.allocate_id(RecordKind::Session, record.id);
        record.id = Some(id);
        self.enqueue(WriteCommand::SaveSession(record))?;
        Ok(id)
    }

    fn load_session(&self, id: i64) -> Result<Option<Session>, StoreError> {
        let connection = self.reader_lock()?;
        mapping::load_session(&connection, id)
    }

    fn save_gap(&mut self, gap: &SessionGap) -> Result<i64, StoreError> {
        let mut record = gap.clone();
        let id = self.allocate_id(RecordKind::Gap, record.id);
        record.id = Some(id);
        self.enqueue(WriteCommand::SaveGap(record))?;
        Ok(id)
    }

    fn save_debt(&mut self, debt: &Debt) -> Result<i64, StoreError> {
        let mut record = debt.clone();
        let id = self.allocate_id(RecordKind::Debt, record.id);
        record.id = Some(id);
        self.enqueue(WriteCommand::SaveDebt(record))?;
        Ok(id)
    }

    fn list_open_debts(&self, activity_id: i64) -> Result<Vec<Debt>, StoreError> {
        let connection = self.reader_lock()?;
        mapping::list_open_debts(&connection, activity_id)
    }

    fn save_contract(&mut self, contract: &CommitmentContract) -> Result<i64, StoreError> {
        let mut record = contract.clone();
        let id = self.allocate_id(RecordKind::Contract, record.id);
        record.id = Some(id);
        self.enqueue(WriteCommand::SaveContract(record))?;
        Ok(id)
    }

    fn load_active_contract(&self) -> Result<Option<CommitmentContract>, StoreError> {
        let connection = self.reader_lock()?;
        mapping::load_active_contract(&connection)
    }

    fn flush(&mut self) -> Result<(), StoreError> {
        self.request(WriteCommand::Flush)
    }
}

impl Drop for SqliteStore {
    fn drop(&mut self) {
        // Best effort, in order, and never panicking: the daemon may be
        // shutting down while the writer thread is already gone.
        let _ = self.flush();
        let _ = self.checkpoint();
        let _ = self.enqueue(WriteCommand::Shutdown);
        if let Some(handle) = self.writer_handle.take() {
            let _ = handle.join();
        }
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
    use std::path::{Path, PathBuf};
    use std::sync::mpsc;
    use std::time::Duration;

    use rusqlite::Connection;
    use tpt_core::domain::{
        AbortReason, Activity, CommitmentContract, Debt, DebtCloseReason, DebtLevel, GapKind,
        RatingScale, ScheduleRule, Session, SessionGap, SessionMode, SessionStatus,
        StrictnessLevel, TimeEntry, TimeEntrySource, TrackingMode,
    };
    use tpt_core::error::StoreError;
    use tpt_core::ports::Store;

    use super::writer::WriteCommand;
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

    fn open_temp_store() -> (tempfile::TempDir, SqliteStore) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("metrics.db");
        let store = SqliteStore::open(&path).unwrap();
        (dir, store)
    }

    fn wal_path_for(path: &Path) -> PathBuf {
        let mut raw = path.as_os_str().to_owned();
        raw.push("-wal");
        PathBuf::from(raw)
    }

    fn sample_activity(name: &str, created_at: i64) -> Activity {
        Activity {
            id: None,
            name: name.to_owned(),
            project_id: None,
            target_minutes: Some(90),
            schedule_rule: Some(ScheduleRule::Weekdays),
            tracking_mode: TrackingMode::Timer,
            pending_extra_ratio: 0.25,
            archived: false,
            created_at,
        }
    }

    fn sample_time_entry(activity_id: i64) -> TimeEntry {
        TimeEntry {
            id: None,
            activity_id,
            source: TimeEntrySource::Manual,
            seconds: 1800,
            day: "2026-09-20".to_owned(),
            note: Some("pairing".to_owned()),
            session_id: None,
            external_id: Some("sp-42".to_owned()),
            created_at: 1_700_000_000,
        }
    }

    fn sample_gap(session_id: i64) -> SessionGap {
        SessionGap {
            id: None,
            session_id,
            kind: GapKind::Idle,
            seconds: 480,
            deducted_seconds: 420,
            qualified: true,
            strictness: StrictnessLevel::L1,
            at: 1_700_000_500,
        }
    }

    fn sample_debt(activity_id: i64) -> Debt {
        Debt {
            id: None,
            activity_id,
            level: DebtLevel::L1,
            base_seconds: 300,
            growth_per_day_seconds: 300,
            cap_seconds: 1200,
            created_at: 1_700_000_000,
            due_at: 1_701_296_000,
            paid_seconds: 60,
            closed_at: None,
            closed_reason: None,
        }
    }

    fn sample_contract() -> CommitmentContract {
        CommitmentContract {
            id: None,
            level: StrictnessLevel::L2,
            term_seconds: 604_800,
            started_at: 1_700_000_000,
            ends_at: 1_700_604_800,
            params_snapshot: r#"{"gap_ratio":0.25}"#.to_owned(),
            checksum: "checksum-1".to_owned(),
            closed_at: None,
        }
    }

    #[test]
    fn store_writer_persists_session_without_blocking() {
        let (_dir, mut store) = open_temp_store();

        let activity_id = store
            .save_activity(&sample_activity("Deep work", 1_700_000_000))
            .unwrap();
        store.flush().unwrap();

        let (release_sender, release_receiver) = mpsc::channel();
        store
            .writer
            .send(WriteCommand::BlockUntil(release_receiver))
            .unwrap();

        let session = Session::new(
            activity_id,
            SessionMode::Focus,
            StrictnessLevel::L1,
            Some(1500),
            None,
            1_700_000_100,
        );
        let (done_sender, done_receiver) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            let id = store.save_session(&session).unwrap();
            done_sender.send(()).unwrap();
            (store, id)
        });

        assert!(
            done_receiver.recv_timeout(Duration::from_secs(2)).is_ok(),
            "save_session must return without waiting for the writer thread"
        );
        release_sender.send(()).unwrap();

        let (mut store, session_id) = handle.join().unwrap();
        store.flush().unwrap();

        let loaded = store.load_session(session_id).unwrap().unwrap();
        assert_eq!(loaded.activity_id, activity_id);
        assert_eq!(loaded.mode, SessionMode::Focus);
        assert_eq!(loaded.started_at, 1_700_000_100);
    }

    #[test]
    fn store_does_not_reuse_ids_after_explicit_id_save() {
        let (_dir, mut store) = open_temp_store();

        let mut explicit = sample_activity("Explicit id", 1_700_000_000);
        explicit.id = Some(1);
        assert_eq!(store.save_activity(&explicit).unwrap(), 1);

        let assigned = sample_activity("Assigned id", 1_700_000_001);
        assert_eq!(store.save_activity(&assigned).unwrap(), 2);

        store.flush().unwrap();
        let ids: Vec<i64> = store
            .list_activities(true)
            .unwrap()
            .iter()
            .map(|activity| activity.id.unwrap())
            .collect();
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn store_returns_backend_when_writer_thread_is_dead() {
        let (_dir, mut store) = open_temp_store();
        store.stop_writer_for_test();

        let error = store
            .save_activity(&sample_activity("Orphan", 1_700_000_000))
            .err()
            .unwrap();

        assert!(matches!(error, StoreError::Backend { .. }));
    }

    #[test]
    fn store_rejects_future_schema_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("metrics.db");
        drop(SqliteStore::open(&path).unwrap());
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .execute_batch("PRAGMA user_version = 99;")
                .unwrap();
        }

        let error = SqliteStore::open(&path).err().unwrap();

        assert!(matches!(error, StoreError::Backend { .. }));
    }

    #[test]
    fn store_rejects_unknown_enum_text() {
        let (_dir, mut store) = open_temp_store();
        let id = store
            .save_activity(&sample_activity("Typo", 1_700_000_000))
            .unwrap();
        store.flush().unwrap();
        {
            let connection = store.reader.lock().unwrap();
            // Simulate corruption: the CHECK constraint would reject this value.
            connection
                .execute_batch("PRAGMA ignore_check_constraints = ON;")
                .unwrap();
            connection
                .execute(
                    "UPDATE activities SET tracking_mode = 'TURBO' WHERE id = ?1",
                    [id],
                )
                .unwrap();
        }

        let error = store.load_activity(id).err().unwrap();

        assert!(matches!(error, StoreError::Backend { .. }));
    }

    #[test]
    fn checkpoint_truncates_wal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("metrics.db");
        let mut store = SqliteStore::open(&path).unwrap();
        let id = store
            .save_activity(&sample_activity("Wal", 1_700_000_000))
            .unwrap();
        store.flush().unwrap();

        let wal_path = wal_path_for(&path);
        assert!(
            std::fs::metadata(&wal_path).unwrap().len() > 0,
            "the wal must hold frames after flush"
        );

        store.checkpoint().unwrap();

        assert_eq!(
            std::fs::metadata(&wal_path).unwrap().len(),
            0,
            "wal_checkpoint(TRUNCATE) must leave the wal empty"
        );
        assert!(store.load_activity(id).unwrap().is_some());
    }

    #[test]
    fn store_flushes_without_pending_writes() {
        let (_dir, mut store) = open_temp_store();

        assert!(store.flush().is_ok());
    }

    #[test]
    fn store_reports_deferred_write_error_once_and_keeps_processing() {
        let (_dir, mut store) = open_temp_store();
        let activity_id = store
            .save_activity(&sample_activity("Checks", 1_700_000_000))
            .unwrap();
        store.flush().unwrap();

        let mut invalid = sample_time_entry(activity_id);
        invalid.seconds = -1;
        assert!(store.save_time_entry(&invalid).is_ok());

        let valid = sample_time_entry(activity_id);
        let valid_id = store.save_time_entry(&valid).unwrap();

        let error = store.flush().err().unwrap();
        assert!(matches!(error, StoreError::Backend { .. }));
        assert!(
            store.flush().is_ok(),
            "flush must clear the first deferred error"
        );

        let entries = store.list_time_entries(activity_id).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, Some(valid_id));
    }

    #[test]
    fn store_round_trips_activity() {
        let (_dir, mut store) = open_temp_store();
        let expected = sample_activity("Read a book", 1_700_000_000);
        let id = store.save_activity(&expected).unwrap();
        store.flush().unwrap();

        let loaded = store.load_activity(id).unwrap().unwrap();

        assert_eq!(loaded, expected_with_id(&expected, id));
        assert_eq!(store.list_activities(false).unwrap().len(), 1);
        assert_eq!(store.list_activities(true).unwrap().len(), 1);
    }

    fn expected_with_id(activity: &Activity, id: i64) -> Activity {
        let mut expected = activity.clone();
        expected.id = Some(id);
        expected
    }

    #[test]
    fn store_round_trips_time_entry() {
        let (_dir, mut store) = open_temp_store();
        let activity_id = store
            .save_activity(&sample_activity("Entries", 1_700_000_000))
            .unwrap();
        store.flush().unwrap();
        let expected = sample_time_entry(activity_id);
        let id = store.save_time_entry(&expected).unwrap();
        store.flush().unwrap();

        let loaded = store.list_time_entries(activity_id).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, Some(id));
        assert_eq!(loaded[0].source, TimeEntrySource::Manual);
        assert_eq!(loaded[0].seconds, 1800);
        assert_eq!(loaded[0].day, "2026-09-20");
        assert_eq!(loaded[0].note.as_deref(), Some("pairing"));
        assert_eq!(loaded[0].external_id.as_deref(), Some("sp-42"));
        assert_eq!(loaded[0].created_at, 1_700_000_000);
    }

    #[test]
    fn store_round_trips_session() {
        let (_dir, mut store) = open_temp_store();
        let activity_id = store
            .save_activity(&sample_activity("Sessions", 1_700_000_000))
            .unwrap();
        store.flush().unwrap();

        let mut session = Session::new(
            activity_id,
            SessionMode::Pomodoro,
            StrictnessLevel::L2,
            Some(1500),
            Some(4),
            1_700_000_000,
        );
        session.elapsed_seconds = 900;
        session.credited_seconds = 840;
        session.status = SessionStatus::AbortedPenalized;
        session.cycle = 2;
        session.ended_at = Some(1_700_000_900);
        session.aborted_with = Some(AbortReason::Idle);
        session.accumulated_gap_seconds = 60;
        session.gap_ratio = 0.04;
        session.rating = Some(4);
        session.rating_scale = Some(RatingScale::Five);
        session.reflection = Some("tired".to_owned());
        let id = store.save_session(&session).unwrap();
        store.flush().unwrap();

        let loaded = store.load_session(id).unwrap().unwrap();

        assert_eq!(loaded.activity_id, activity_id);
        assert_eq!(loaded.mode, SessionMode::Pomodoro);
        assert_eq!(loaded.status, SessionStatus::AbortedPenalized);
        assert_eq!(loaded.strictness, StrictnessLevel::L2);
        assert_eq!(loaded.total_cycles, Some(4));
        assert_eq!(loaded.cycle, 2);
        assert_eq!(loaded.target_seconds, Some(1500));
        assert_eq!(loaded.elapsed_seconds, 900);
        assert_eq!(loaded.credited_seconds, 840);
        assert_eq!(loaded.started_at, 1_700_000_000);
        assert_eq!(loaded.ended_at, Some(1_700_000_900));
        assert_eq!(loaded.aborted_with, Some(AbortReason::Idle));
        assert_eq!(loaded.accumulated_gap_seconds, 60);
        assert!((loaded.gap_ratio - 0.04).abs() < f64::EPSILON);
        assert_eq!(loaded.rating, Some(4));
        assert_eq!(loaded.rating_scale, Some(RatingScale::Five));
        assert_eq!(loaded.reflection.as_deref(), Some("tired"));
    }

    #[test]
    fn store_round_trips_gap() {
        let (_dir, mut store) = open_temp_store();
        let activity_id = store
            .save_activity(&sample_activity("Gaps", 1_700_000_000))
            .unwrap();
        store.flush().unwrap();
        let session_id = store
            .save_session(&Session::new(
                activity_id,
                SessionMode::Flowtime,
                StrictnessLevel::L1,
                None,
                None,
                1_700_000_000,
            ))
            .unwrap();
        store.flush().unwrap();

        let expected = sample_gap(session_id);
        let id = store.save_gap(&expected).unwrap();
        store.flush().unwrap();

        let connection = store.reader.lock().unwrap();
        let (kind, seconds, deducted, qualified, strictness, at): (
            String,
            i64,
            i64,
            bool,
            String,
            i64,
        ) = connection
            .query_row(
                "SELECT kind, seconds, deducted_seconds, qualified, strictness, \
                 CAST(strftime('%s', at) AS INTEGER) FROM session_gaps WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .unwrap();
        assert_eq!(kind, "IDLE");
        assert_eq!(seconds, 480);
        assert_eq!(deducted, 420);
        assert!(qualified);
        assert_eq!(strictness, "L1");
        assert_eq!(at, 1_700_000_500);
    }

    #[test]
    fn store_round_trips_debt_and_lists_only_open_ones() {
        let (_dir, mut store) = open_temp_store();
        let activity_id = store
            .save_activity(&sample_activity("Debts", 1_700_000_000))
            .unwrap();
        store.flush().unwrap();

        let open_id = store.save_debt(&sample_debt(activity_id)).unwrap();
        let mut closed = sample_debt(activity_id);
        closed.closed_at = Some(1_700_100_000);
        closed.closed_reason = Some(DebtCloseReason::Paid);
        let closed_id = store.save_debt(&closed).unwrap();
        store.flush().unwrap();

        let open = store.list_open_debts(activity_id).unwrap();

        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, Some(open_id));
        assert_eq!(open[0].level, DebtLevel::L1);
        assert_eq!(open[0].base_seconds, 300);
        assert_eq!(open[0].growth_per_day_seconds, 300);
        assert_eq!(open[0].cap_seconds, 1200);
        assert_eq!(open[0].due_at, 1_701_296_000);
        assert_eq!(open[0].paid_seconds, 60);

        let connection = store.reader.lock().unwrap();
        let (closed_at, reason): (Option<i64>, Option<String>) = connection
            .query_row(
                "SELECT CAST(strftime('%s', closed_at) AS INTEGER), closed_reason \
                 FROM debts WHERE id = ?1",
                [closed_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(closed_at, Some(1_700_100_000));
        assert_eq!(reason.as_deref(), Some("PAID"));
    }

    #[test]
    fn store_round_trips_contract_and_loads_the_active_one() {
        let (_dir, mut store) = open_temp_store();

        let mut closed = sample_contract();
        closed.closed_at = Some(1_700_100_000);
        closed.checksum = "closed".to_owned();
        store.save_contract(&closed).unwrap();
        let active = sample_contract();
        let active_id = store.save_contract(&active).unwrap();
        store.flush().unwrap();

        let loaded = store.load_active_contract().unwrap().unwrap();

        assert_eq!(loaded.id, Some(active_id));
        assert_eq!(loaded.level, StrictnessLevel::L2);
        assert_eq!(loaded.term_seconds, 604_800);
        assert_eq!(loaded.started_at, 1_700_000_000);
        assert_eq!(loaded.ends_at, 1_700_604_800);
        assert_eq!(loaded.params_snapshot, r#"{"gap_ratio":0.25}"#);
        assert_eq!(loaded.checksum, "checksum-1");
        assert_eq!(loaded.closed_at, None);
    }

    #[test]
    fn store_updates_existing_record_instead_of_duplicating() {
        let (_dir, mut store) = open_temp_store();
        let id = store
            .save_activity(&sample_activity("Before", 1_700_000_000))
            .unwrap();
        store.flush().unwrap();

        let mut updated = store.load_activity(id).unwrap().unwrap();
        updated.name = "After".to_owned();
        assert_eq!(store.save_activity(&updated).unwrap(), id);
        store.flush().unwrap();

        let loaded = store.load_activity(id).unwrap().unwrap();
        assert_eq!(loaded.name, "After");
        assert_eq!(store.list_activities(true).unwrap().len(), 1);
    }
}
