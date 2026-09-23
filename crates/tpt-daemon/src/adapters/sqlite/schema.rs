//! SQLite DDL and schema migration (ADR-002).

use rusqlite::{Connection, TransactionBehavior};

use tpt_core::error::StoreError;

use super::{backend, sql_backend};

/// Schema version produced by this adapter.
pub(super) const SCHEMA_VERSION: i64 = 1;

/// Applies the connection-level pragmas in their required order.
///
/// `auto_vacuum` must be configured before the first table is created, and the
/// value returned by `journal_mode = WAL` is verified because some filesystems
/// silently refuse WAL.
pub(super) fn configure(connection: &Connection) -> Result<(), StoreError> {
    connection
        .execute_batch("PRAGMA auto_vacuum = INCREMENTAL;")
        .map_err(sql_backend)?;

    let journal_mode: String = connection
        .query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))
        .map_err(sql_backend)?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        return Err(backend(format!(
            "journal_mode is '{journal_mode}', expected 'wal'"
        )));
    }

    connection
        .execute_batch(
            "PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )
        .map_err(sql_backend)?;

    Ok(())
}

/// Reads `PRAGMA user_version` from the open connection.
pub(super) fn schema_version(connection: &Connection) -> Result<i64, StoreError> {
    connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(sql_backend)
}

/// Migrates the database to [`SCHEMA_VERSION`], refusing newer schemas.
///
/// Version `0` applies the full v1 schema and seals `user_version = 1` inside a
/// single `BEGIN IMMEDIATE` transaction, so an interrupted migration leaves no
/// partial schema behind.
pub(super) fn migrate(connection: &mut Connection) -> Result<(), StoreError> {
    let version = schema_version(connection)?;
    if version > SCHEMA_VERSION {
        return Err(backend(format!(
            "database schema version {version} is newer than the supported {SCHEMA_VERSION}"
        )));
    }
    if version == SCHEMA_VERSION {
        return Ok(());
    }

    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(sql_backend)?;
    transaction.execute_batch(DDL).map_err(sql_backend)?;
    transaction
        .execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION};"))
        .map_err(sql_backend)?;
    transaction.commit().map_err(sql_backend)?;

    Ok(())
}

/// Full v1 schema: 10 tables and 9 indexes, exactly as specified by ADR-002.
const DDL: &str = r#"
-- Projects
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color_hex TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Activities (the unit time is credited to)
CREATE TABLE activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
    target_minutes INTEGER,
    schedule_rule TEXT,               -- JSON; NULL = no schedule (allowed)
    tracking_mode TEXT NOT NULL CHECK(tracking_mode IN ('TIMER','MANUAL')),
    pending_extra_ratio REAL NOT NULL DEFAULT 0,   -- accumulated refinancing (caps at 0.40)
    archived INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Tags
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE activity_tags (
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (activity_id, tag_id)
);

-- Timer sessions
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    mode TEXT NOT NULL CHECK(mode IN ('FLOWTIME','POMODORO','FOCUS')),
    target_seconds INTEGER,           -- NULL on Flowtime
    elapsed_seconds INTEGER NOT NULL DEFAULT 0,
    credited_seconds INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK(status IN
        ('RUNNING','BREAK','AWAITING','SUSPENDED','COMPLETED','ABORTED','ABORTED_PENALIZED')),
    cycle INTEGER NOT NULL DEFAULT 1,
    total_cycles INTEGER,             -- N Pomodoro cycles; NULL on other modes
    strictness TEXT NOT NULL CHECK(strictness IN ('OFF','L0','L1','L2')),
    started_at TEXT NOT NULL,
    ended_at TEXT,
    aborted_with TEXT CHECK(aborted_with IN
        ('USER','IDLE','SCREEN_OFF','SUSPENDED','DAEMON_KILLED')),
    accumulated_gap_seconds INTEGER NOT NULL DEFAULT 0,
    gap_ratio REAL NOT NULL DEFAULT 0,
    rating INTEGER CHECK(rating BETWEEN 1 AND 10),    -- optional
    rating_scale INTEGER CHECK(rating_scale IN (5,7,10)),
    reflection TEXT                                    -- optional, independent from rating
);

-- Time entries (manual / derived from sessions / imported / penalties)
CREATE TABLE time_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    source TEXT NOT NULL CHECK(source IN ('MANUAL','SESSION','IMPORTED','PENALTY')),
    seconds INTEGER NOT NULL,          -- negative ONLY when source = 'PENALTY'
    day TEXT NOT NULL,                 -- YYYY-MM-DD local
    note TEXT,
    session_id INTEGER REFERENCES sessions(id) ON DELETE SET NULL,
    external_id TEXT UNIQUE,           -- Super Productivity id -> idempotent dedupe
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (source = 'PENALTY' OR seconds >= 0)
);

-- Reparation debt (ADR-004 section 6)
CREATE TABLE debts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    level TEXT NOT NULL CHECK(level IN ('L1','L2')),
    base_seconds INTEGER NOT NULL,     -- amount when created
    growth_per_day_seconds INTEGER NOT NULL DEFAULT 300,   -- +5 min/day
    cap_seconds INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    due_at TEXT NOT NULL,              -- L1: +15 days - L2: +7 days
    paid_seconds INTEGER NOT NULL DEFAULT 0,
    closed_at TEXT,
    closed_reason TEXT CHECK(closed_reason IN ('PAID','REFINANCED','EXPIRED'))
);

-- Extensions (+) applied to a session: additive contract audit
CREATE TABLE session_additions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    seconds INTEGER NOT NULL CHECK(seconds > 0),
    at TEXT NOT NULL
);

-- Presence gaps detected during a session
CREATE TABLE session_gaps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK(kind IN ('IDLE','SCREEN_OFF','SUSPENDED')),
    seconds INTEGER NOT NULL CHECK(seconds >= 0),
    deducted_seconds INTEGER NOT NULL CHECK(deducted_seconds >= 0),
    qualified INTEGER NOT NULL DEFAULT 0,   -- 1 when it exceeded min_gap_seconds
    strictness TEXT NOT NULL,
    at TEXT NOT NULL
);

-- Commitment contract with its strictness level
CREATE TABLE commitment_contracts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    level TEXT NOT NULL CHECK(level IN ('OFF','L0','L1','L2')),
    term_seconds INTEGER NOT NULL,
    started_at TEXT NOT NULL,
    ends_at TEXT NOT NULL,
    params_snapshot TEXT NOT NULL,     -- JSON with ALL level parameters (ADR-004 section 10)
    checksum TEXT NOT NULL,            -- covers level + term + params_snapshot
    closed_at TEXT
);

CREATE INDEX idx_activities_project ON activities(project_id);
CREATE INDEX idx_activities_archived ON activities(archived);
CREATE INDEX idx_entries_activity_day ON time_entries(activity_id, day);
CREATE INDEX idx_entries_day ON time_entries(day);
CREATE INDEX idx_entries_source ON time_entries(source);
CREATE INDEX idx_sessions_activity ON sessions(activity_id, started_at);
CREATE INDEX idx_sessions_started ON sessions(started_at);
CREATE INDEX idx_gaps_session ON session_gaps(session_id);
CREATE INDEX idx_debts_open ON debts(activity_id, closed_at);
"#;
