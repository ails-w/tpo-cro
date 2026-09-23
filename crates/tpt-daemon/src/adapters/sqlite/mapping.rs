//! Row <-> domain mapping for the SQLite adapter (ADR-002).
//!
//! Timestamps are stored as SQLite `TEXT` datetimes: writes wrap the unix
//! seconds in `datetime(?N, 'unixepoch')` and reads unwrap them with
//! `CAST(strftime('%s', column) AS INTEGER)`. Enums travel as their
//! `SCREAMING_SNAKE_CASE` text and unknown values are backend errors.

use rusqlite::types::Type;
use rusqlite::{Connection, Row, ToSql};

use tpt_core::domain::{
    AbortReason, Activity, CommitmentContract, Debt, DebtCloseReason, DebtLevel, GapKind,
    RatingScale, ScheduleRule, Session, SessionGap, SessionMode, SessionStatus, StrictnessLevel,
    TimeEntry, TimeEntrySource, TrackingMode,
};
use tpt_core::error::StoreError;

use super::{backend, sql_backend};

/// Carries a domain-conversion failure through a `rusqlite::Result`.
fn conversion(message: impl Into<String>) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message.into(),
        )),
    )
}

/// Every record reaching the writer must already carry its assigned id.
fn require_id(id: Option<i64>, kind: &str) -> Result<i64, StoreError> {
    id.ok_or_else(|| backend(format!("{kind} reached the writer without an id")))
}

fn query_optional<T>(
    connection: &Connection,
    sql: &str,
    params: &[&dyn ToSql],
    map: impl FnOnce(&Row<'_>) -> rusqlite::Result<T>,
) -> Result<Option<T>, StoreError> {
    match connection.query_row(sql, params, map) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(sql_backend(error)),
    }
}

fn query_all<T>(
    connection: &Connection,
    sql: &str,
    params: &[&dyn ToSql],
    map: impl Fn(&Row<'_>) -> rusqlite::Result<T>,
) -> Result<Vec<T>, StoreError> {
    let mut statement = connection.prepare(sql).map_err(sql_backend)?;
    let rows = statement.query_map(params, map).map_err(sql_backend)?;
    let mut values = Vec::new();
    for row in rows {
        values.push(row.map_err(sql_backend)?);
    }
    Ok(values)
}

/// Reads an optional timestamp from its raw text and its cast value, so that
/// `NULL` and corrupted text can be told apart.
fn optional_timestamp(
    row: &Row<'_>,
    raw: usize,
    cast: usize,
    column: &str,
) -> rusqlite::Result<Option<i64>> {
    let raw: Option<String> = row.get(raw)?;
    let value: Option<i64> = row.get(cast)?;
    match (&raw, value) {
        (None, None) => Ok(None),
        (Some(_), Some(value)) => Ok(Some(value)),
        _ => Err(conversion(format!("malformed timestamp in {column}"))),
    }
}

// ---------------------------------------------------------------------------
// Enum text codecs
// ---------------------------------------------------------------------------

fn tracking_mode_text(mode: TrackingMode) -> &'static str {
    match mode {
        TrackingMode::Timer => "TIMER",
        TrackingMode::Manual => "MANUAL",
    }
}

fn parse_tracking_mode(text: &str) -> Result<TrackingMode, String> {
    match text {
        "TIMER" => Ok(TrackingMode::Timer),
        "MANUAL" => Ok(TrackingMode::Manual),
        other => Err(format!("activities.tracking_mode: unknown value '{other}'")),
    }
}

fn time_entry_source_text(source: TimeEntrySource) -> &'static str {
    match source {
        TimeEntrySource::Manual => "MANUAL",
        TimeEntrySource::Session => "SESSION",
        TimeEntrySource::Imported => "IMPORTED",
        TimeEntrySource::Penalty => "PENALTY",
    }
}

fn parse_time_entry_source(text: &str) -> Result<TimeEntrySource, String> {
    match text {
        "MANUAL" => Ok(TimeEntrySource::Manual),
        "SESSION" => Ok(TimeEntrySource::Session),
        "IMPORTED" => Ok(TimeEntrySource::Imported),
        "PENALTY" => Ok(TimeEntrySource::Penalty),
        other => Err(format!("time_entries.source: unknown value '{other}'")),
    }
}

fn session_mode_text(mode: SessionMode) -> &'static str {
    match mode {
        SessionMode::Flowtime => "FLOWTIME",
        SessionMode::Pomodoro => "POMODORO",
        SessionMode::Focus => "FOCUS",
    }
}

fn parse_session_mode(text: &str) -> Result<SessionMode, String> {
    match text {
        "FLOWTIME" => Ok(SessionMode::Flowtime),
        "POMODORO" => Ok(SessionMode::Pomodoro),
        "FOCUS" => Ok(SessionMode::Focus),
        other => Err(format!("sessions.mode: unknown value '{other}'")),
    }
}

fn session_status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Running => "RUNNING",
        SessionStatus::Break => "BREAK",
        SessionStatus::Awaiting => "AWAITING",
        SessionStatus::Suspended => "SUSPENDED",
        SessionStatus::Completed => "COMPLETED",
        SessionStatus::Aborted => "ABORTED",
        SessionStatus::AbortedPenalized => "ABORTED_PENALIZED",
    }
}

fn parse_session_status(text: &str) -> Result<SessionStatus, String> {
    match text {
        "RUNNING" => Ok(SessionStatus::Running),
        "BREAK" => Ok(SessionStatus::Break),
        "AWAITING" => Ok(SessionStatus::Awaiting),
        "SUSPENDED" => Ok(SessionStatus::Suspended),
        "COMPLETED" => Ok(SessionStatus::Completed),
        "ABORTED" => Ok(SessionStatus::Aborted),
        "ABORTED_PENALIZED" => Ok(SessionStatus::AbortedPenalized),
        other => Err(format!("sessions.status: unknown value '{other}'")),
    }
}

fn abort_reason_text(reason: AbortReason) -> &'static str {
    match reason {
        AbortReason::User => "USER",
        AbortReason::Idle => "IDLE",
        AbortReason::ScreenOff => "SCREEN_OFF",
        AbortReason::Suspended => "SUSPENDED",
        AbortReason::DaemonKilled => "DAEMON_KILLED",
    }
}

fn parse_abort_reason(text: &str) -> Result<AbortReason, String> {
    match text {
        "USER" => Ok(AbortReason::User),
        "IDLE" => Ok(AbortReason::Idle),
        "SCREEN_OFF" => Ok(AbortReason::ScreenOff),
        "SUSPENDED" => Ok(AbortReason::Suspended),
        "DAEMON_KILLED" => Ok(AbortReason::DaemonKilled),
        other => Err(format!("sessions.aborted_with: unknown value '{other}'")),
    }
}

fn gap_kind_text(kind: GapKind) -> &'static str {
    match kind {
        GapKind::Idle => "IDLE",
        GapKind::ScreenOff => "SCREEN_OFF",
        GapKind::Suspended => "SUSPENDED",
    }
}

fn strictness_text(level: StrictnessLevel) -> &'static str {
    match level {
        StrictnessLevel::Off => "OFF",
        StrictnessLevel::L0 => "L0",
        StrictnessLevel::L1 => "L1",
        StrictnessLevel::L2 => "L2",
    }
}

fn parse_strictness(text: &str) -> Result<StrictnessLevel, String> {
    match text {
        "OFF" => Ok(StrictnessLevel::Off),
        "L0" => Ok(StrictnessLevel::L0),
        "L1" => Ok(StrictnessLevel::L1),
        "L2" => Ok(StrictnessLevel::L2),
        other => Err(format!("strictness: unknown value '{other}'")),
    }
}

fn debt_level_text(level: DebtLevel) -> &'static str {
    match level {
        DebtLevel::L1 => "L1",
        DebtLevel::L2 => "L2",
    }
}

fn parse_debt_level(text: &str) -> Result<DebtLevel, String> {
    match text {
        "L1" => Ok(DebtLevel::L1),
        "L2" => Ok(DebtLevel::L2),
        other => Err(format!("debts.level: unknown value '{other}'")),
    }
}

fn debt_close_reason_text(reason: DebtCloseReason) -> &'static str {
    match reason {
        DebtCloseReason::Paid => "PAID",
        DebtCloseReason::Refinanced => "REFINANCED",
        DebtCloseReason::Expired => "EXPIRED",
    }
}

fn parse_debt_close_reason(text: &str) -> Result<DebtCloseReason, String> {
    match text {
        "PAID" => Ok(DebtCloseReason::Paid),
        "REFINANCED" => Ok(DebtCloseReason::Refinanced),
        "EXPIRED" => Ok(DebtCloseReason::Expired),
        other => Err(format!("debts.closed_reason: unknown value '{other}'")),
    }
}

/// The schema stores the rating scale as `INTEGER 5/7/10`, never as its serde name.
fn parse_rating_scale(value: i64) -> Result<RatingScale, String> {
    match value {
        5 => Ok(RatingScale::Five),
        7 => Ok(RatingScale::Seven),
        10 => Ok(RatingScale::Ten),
        other => Err(format!("rating_scale: unknown value {other}")),
    }
}

// ---------------------------------------------------------------------------
// Activities
// ---------------------------------------------------------------------------

const ACTIVITY_COLUMNS: &str = "id, name, project_id, target_minutes, schedule_rule, \
tracking_mode, pending_extra_ratio, archived, CAST(strftime('%s', created_at) AS INTEGER)";

const UPDATE_ACTIVITY: &str = "UPDATE activities SET name = :name, project_id = :project_id, \
target_minutes = :target_minutes, schedule_rule = :schedule_rule, tracking_mode = :tracking_mode, \
pending_extra_ratio = :pending_extra_ratio, archived = :archived, \
created_at = datetime(:created_at, 'unixepoch') WHERE id = :id";

const INSERT_ACTIVITY: &str = "INSERT INTO activities (id, name, project_id, target_minutes, \
schedule_rule, tracking_mode, pending_extra_ratio, archived, created_at) VALUES (:id, :name, \
:project_id, :target_minutes, :schedule_rule, :tracking_mode, :pending_extra_ratio, :archived, \
datetime(:created_at, 'unixepoch'))";

pub(super) fn upsert_activity(
    connection: &Connection,
    activity: &Activity,
) -> Result<(), StoreError> {
    let id = require_id(activity.id, "activity")?;
    let schedule_rule = match &activity.schedule_rule {
        Some(rule) => Some(
            serde_json::to_string(rule)
                .map_err(|error| backend(format!("activities.schedule_rule: {error}")))?,
        ),
        None => None,
    };
    let tracking_mode = tracking_mode_text(activity.tracking_mode);
    let target_minutes = activity.target_minutes.map(i64::from);
    let params: &[(&str, &dyn ToSql)] = &[
        (":id", &id),
        (":name", &activity.name),
        (":project_id", &activity.project_id),
        (":target_minutes", &target_minutes),
        (":schedule_rule", &schedule_rule),
        (":tracking_mode", &tracking_mode),
        (":pending_extra_ratio", &activity.pending_extra_ratio),
        (":archived", &activity.archived),
        (":created_at", &activity.created_at),
    ];

    let updated = connection
        .execute(UPDATE_ACTIVITY, params)
        .map_err(sql_backend)?;
    if updated == 0 {
        connection
            .execute(INSERT_ACTIVITY, params)
            .map_err(sql_backend)?;
    }
    Ok(())
}

fn activity_from_row(row: &Row<'_>) -> rusqlite::Result<Activity> {
    let schedule_rule: Option<String> = row.get(4)?;
    let schedule_rule = match schedule_rule {
        Some(text) => Some(
            serde_json::from_str::<ScheduleRule>(&text)
                .map_err(|error| conversion(format!("activities.schedule_rule: {error}")))?,
        ),
        None => None,
    };
    let tracking_mode: String = row.get(5)?;
    let target_minutes: Option<i64> = row.get(3)?;

    Ok(Activity {
        id: Some(row.get(0)?),
        name: row.get(1)?,
        project_id: row.get(2)?,
        target_minutes: target_minutes
            .map(|value| {
                u32::try_from(value)
                    .map_err(|error| conversion(format!("activities.target_minutes: {error}")))
            })
            .transpose()?,
        schedule_rule,
        tracking_mode: parse_tracking_mode(&tracking_mode).map_err(conversion)?,
        pending_extra_ratio: row.get(6)?,
        archived: row.get(7)?,
        created_at: row.get(8)?,
    })
}

pub(super) fn load_activity(
    connection: &Connection,
    id: i64,
) -> Result<Option<Activity>, StoreError> {
    let sql = format!("SELECT {ACTIVITY_COLUMNS} FROM activities WHERE id = ?1");
    query_optional(connection, &sql, &[&id], activity_from_row)
}

pub(super) fn list_activities(
    connection: &Connection,
    include_archived: bool,
) -> Result<Vec<Activity>, StoreError> {
    let filter = if include_archived {
        " ORDER BY id"
    } else {
        " WHERE archived = 0 ORDER BY id"
    };
    let sql = format!("SELECT {ACTIVITY_COLUMNS} FROM activities{filter}");
    query_all(connection, &sql, &[], activity_from_row)
}

// ---------------------------------------------------------------------------
// Time entries
// ---------------------------------------------------------------------------

const TIME_ENTRY_COLUMNS: &str = "id, activity_id, source, seconds, day, note, session_id, \
external_id, CAST(strftime('%s', created_at) AS INTEGER)";

const UPDATE_TIME_ENTRY: &str = "UPDATE time_entries SET activity_id = :activity_id, \
source = :source, seconds = :seconds, day = :day, note = :note, session_id = :session_id, \
external_id = :external_id, created_at = datetime(:created_at, 'unixepoch') WHERE id = :id";

const INSERT_TIME_ENTRY: &str = "INSERT INTO time_entries (id, activity_id, source, seconds, day, \
note, session_id, external_id, created_at) VALUES (:id, :activity_id, :source, :seconds, :day, \
:note, :session_id, :external_id, datetime(:created_at, 'unixepoch'))";

pub(super) fn upsert_time_entry(
    connection: &Connection,
    entry: &TimeEntry,
) -> Result<(), StoreError> {
    let id = require_id(entry.id, "time entry")?;
    let source = time_entry_source_text(entry.source);
    let params: &[(&str, &dyn ToSql)] = &[
        (":id", &id),
        (":activity_id", &entry.activity_id),
        (":source", &source),
        (":seconds", &entry.seconds),
        (":day", &entry.day),
        (":note", &entry.note),
        (":session_id", &entry.session_id),
        (":external_id", &entry.external_id),
        (":created_at", &entry.created_at),
    ];

    let updated = connection
        .execute(UPDATE_TIME_ENTRY, params)
        .map_err(sql_backend)?;
    if updated == 0 {
        connection
            .execute(INSERT_TIME_ENTRY, params)
            .map_err(sql_backend)?;
    }
    Ok(())
}

fn time_entry_from_row(row: &Row<'_>) -> rusqlite::Result<TimeEntry> {
    let source: String = row.get(2)?;

    Ok(TimeEntry {
        id: Some(row.get(0)?),
        activity_id: row.get(1)?,
        source: parse_time_entry_source(&source).map_err(conversion)?,
        seconds: row.get(3)?,
        day: row.get(4)?,
        note: row.get(5)?,
        session_id: row.get(6)?,
        external_id: row.get(7)?,
        created_at: row.get(8)?,
    })
}

pub(super) fn list_time_entries(
    connection: &Connection,
    activity_id: i64,
) -> Result<Vec<TimeEntry>, StoreError> {
    let sql = format!(
        "SELECT {TIME_ENTRY_COLUMNS} FROM time_entries WHERE activity_id = ?1 ORDER BY day, id"
    );
    query_all(connection, &sql, &[&activity_id], time_entry_from_row)
}

// ---------------------------------------------------------------------------
// Sessions
// ---------------------------------------------------------------------------

const SESSION_COLUMNS: &str = "id, activity_id, mode, target_seconds, elapsed_seconds, \
credited_seconds, status, cycle, total_cycles, strictness, \
CAST(strftime('%s', started_at) AS INTEGER), ended_at, CAST(strftime('%s', ended_at) AS INTEGER), \
aborted_with, accumulated_gap_seconds, gap_ratio, rating, rating_scale, reflection";

const UPDATE_SESSION: &str = "UPDATE sessions SET activity_id = :activity_id, mode = :mode, \
target_seconds = :target_seconds, elapsed_seconds = :elapsed_seconds, \
credited_seconds = :credited_seconds, status = :status, cycle = :cycle, \
total_cycles = :total_cycles, strictness = :strictness, \
started_at = datetime(:started_at, 'unixepoch'), \
ended_at = CASE WHEN :ended_at IS NULL THEN NULL ELSE datetime(:ended_at, 'unixepoch') END, \
aborted_with = :aborted_with, accumulated_gap_seconds = :accumulated_gap_seconds, \
gap_ratio = :gap_ratio, rating = :rating, rating_scale = :rating_scale, \
reflection = :reflection WHERE id = :id";

const INSERT_SESSION: &str = "INSERT INTO sessions (id, activity_id, mode, target_seconds, \
elapsed_seconds, credited_seconds, status, cycle, total_cycles, strictness, started_at, ended_at, \
aborted_with, accumulated_gap_seconds, gap_ratio, rating, rating_scale, reflection) VALUES (:id, \
:activity_id, :mode, :target_seconds, :elapsed_seconds, :credited_seconds, :status, :cycle, \
:total_cycles, :strictness, datetime(:started_at, 'unixepoch'), CASE WHEN :ended_at IS NULL THEN \
NULL ELSE datetime(:ended_at, 'unixepoch') END, :aborted_with, :accumulated_gap_seconds, \
:gap_ratio, :rating, :rating_scale, :reflection)";

pub(super) fn upsert_session(connection: &Connection, session: &Session) -> Result<(), StoreError> {
    let id = require_id(session.id, "session")?;
    let mode = session_mode_text(session.mode);
    let status = session_status_text(session.status);
    let strictness = strictness_text(session.strictness);
    let aborted_with = session.aborted_with.map(abort_reason_text);
    let cycle = i64::from(session.cycle);
    let total_cycles = session.total_cycles.map(i64::from);
    let rating = session.rating.map(i64::from);
    let rating_scale = session.rating_scale.map(|scale| i64::from(scale.value()));
    let params: &[(&str, &dyn ToSql)] = &[
        (":id", &id),
        (":activity_id", &session.activity_id),
        (":mode", &mode),
        (":target_seconds", &session.target_seconds),
        (":elapsed_seconds", &session.elapsed_seconds),
        (":credited_seconds", &session.credited_seconds),
        (":status", &status),
        (":cycle", &cycle),
        (":total_cycles", &total_cycles),
        (":strictness", &strictness),
        (":started_at", &session.started_at),
        (":ended_at", &session.ended_at),
        (":aborted_with", &aborted_with),
        (":accumulated_gap_seconds", &session.accumulated_gap_seconds),
        (":gap_ratio", &session.gap_ratio),
        (":rating", &rating),
        (":rating_scale", &rating_scale),
        (":reflection", &session.reflection),
    ];

    let updated = connection
        .execute(UPDATE_SESSION, params)
        .map_err(sql_backend)?;
    if updated == 0 {
        connection
            .execute(INSERT_SESSION, params)
            .map_err(sql_backend)?;
    }
    Ok(())
}

fn session_from_row(row: &Row<'_>) -> rusqlite::Result<Session> {
    let mode: String = row.get(2)?;
    let status: String = row.get(6)?;
    let strictness: String = row.get(9)?;
    let aborted_with: Option<String> = row.get(13)?;
    let total_cycles: Option<i64> = row.get(8)?;
    let rating: Option<i64> = row.get(16)?;
    let rating_scale: Option<i64> = row.get(17)?;

    Ok(Session {
        id: Some(row.get(0)?),
        activity_id: row.get(1)?,
        mode: parse_session_mode(&mode).map_err(conversion)?,
        target_seconds: row.get(3)?,
        elapsed_seconds: row.get(4)?,
        credited_seconds: row.get(5)?,
        status: parse_session_status(&status).map_err(conversion)?,
        cycle: u32::try_from(row.get::<_, i64>(7)?)
            .map_err(|error| conversion(format!("sessions.cycle: {error}")))?,
        total_cycles: total_cycles
            .map(|value| {
                u32::try_from(value)
                    .map_err(|error| conversion(format!("sessions.total_cycles: {error}")))
            })
            .transpose()?,
        strictness: parse_strictness(&strictness).map_err(conversion)?,
        started_at: row.get(10)?,
        ended_at: optional_timestamp(row, 11, 12, "sessions.ended_at")?,
        aborted_with: match aborted_with {
            Some(text) => Some(parse_abort_reason(&text).map_err(conversion)?),
            None => None,
        },
        accumulated_gap_seconds: row.get(14)?,
        gap_ratio: row.get(15)?,
        rating: rating
            .map(|value| {
                u8::try_from(value).map_err(|error| conversion(format!("sessions.rating: {error}")))
            })
            .transpose()?,
        rating_scale: rating_scale
            .map(|value| parse_rating_scale(value).map_err(conversion))
            .transpose()?,
        reflection: row.get(18)?,
    })
}

pub(super) fn load_session(
    connection: &Connection,
    id: i64,
) -> Result<Option<Session>, StoreError> {
    let sql = format!("SELECT {SESSION_COLUMNS} FROM sessions WHERE id = ?1");
    query_optional(connection, &sql, &[&id], session_from_row)
}

// ---------------------------------------------------------------------------
// Session gaps
// ---------------------------------------------------------------------------

const UPDATE_GAP: &str = "UPDATE session_gaps SET session_id = :session_id, kind = :kind, \
seconds = :seconds, deducted_seconds = :deducted_seconds, qualified = :qualified, \
strictness = :strictness, at = datetime(:at, 'unixepoch') WHERE id = :id";

const INSERT_GAP: &str = "INSERT INTO session_gaps (id, session_id, kind, seconds, \
deducted_seconds, qualified, strictness, at) VALUES (:id, :session_id, :kind, :seconds, \
:deducted_seconds, :qualified, :strictness, datetime(:at, 'unixepoch'))";

pub(super) fn upsert_gap(connection: &Connection, gap: &SessionGap) -> Result<(), StoreError> {
    let id = require_id(gap.id, "session gap")?;
    let kind = gap_kind_text(gap.kind);
    let strictness = strictness_text(gap.strictness);
    let params: &[(&str, &dyn ToSql)] = &[
        (":id", &id),
        (":session_id", &gap.session_id),
        (":kind", &kind),
        (":seconds", &gap.seconds),
        (":deducted_seconds", &gap.deducted_seconds),
        (":qualified", &gap.qualified),
        (":strictness", &strictness),
        (":at", &gap.at),
    ];

    let updated = connection
        .execute(UPDATE_GAP, params)
        .map_err(sql_backend)?;
    if updated == 0 {
        connection
            .execute(INSERT_GAP, params)
            .map_err(sql_backend)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Debts
// ---------------------------------------------------------------------------

const DEBT_COLUMNS: &str = "id, activity_id, level, base_seconds, growth_per_day_seconds, \
cap_seconds, CAST(strftime('%s', created_at) AS INTEGER), \
CAST(strftime('%s', due_at) AS INTEGER), paid_seconds, closed_at, \
CAST(strftime('%s', closed_at) AS INTEGER), closed_reason";

const UPDATE_DEBT: &str = "UPDATE debts SET activity_id = :activity_id, level = :level, \
base_seconds = :base_seconds, growth_per_day_seconds = :growth_per_day_seconds, \
cap_seconds = :cap_seconds, created_at = datetime(:created_at, 'unixepoch'), \
due_at = datetime(:due_at, 'unixepoch'), paid_seconds = :paid_seconds, \
closed_at = CASE WHEN :closed_at IS NULL THEN NULL ELSE datetime(:closed_at, 'unixepoch') END, \
closed_reason = :closed_reason WHERE id = :id";

const INSERT_DEBT: &str = "INSERT INTO debts (id, activity_id, level, base_seconds, \
growth_per_day_seconds, cap_seconds, created_at, due_at, paid_seconds, closed_at, closed_reason) \
VALUES (:id, :activity_id, :level, :base_seconds, :growth_per_day_seconds, :cap_seconds, \
datetime(:created_at, 'unixepoch'), datetime(:due_at, 'unixepoch'), :paid_seconds, \
CASE WHEN :closed_at IS NULL THEN NULL ELSE datetime(:closed_at, 'unixepoch') END, \
:closed_reason)";

pub(super) fn upsert_debt(connection: &Connection, debt: &Debt) -> Result<(), StoreError> {
    let id = require_id(debt.id, "debt")?;
    let level = debt_level_text(debt.level);
    let closed_reason = debt.closed_reason.map(debt_close_reason_text);
    let params: &[(&str, &dyn ToSql)] = &[
        (":id", &id),
        (":activity_id", &debt.activity_id),
        (":level", &level),
        (":base_seconds", &debt.base_seconds),
        (":growth_per_day_seconds", &debt.growth_per_day_seconds),
        (":cap_seconds", &debt.cap_seconds),
        (":created_at", &debt.created_at),
        (":due_at", &debt.due_at),
        (":paid_seconds", &debt.paid_seconds),
        (":closed_at", &debt.closed_at),
        (":closed_reason", &closed_reason),
    ];

    let updated = connection
        .execute(UPDATE_DEBT, params)
        .map_err(sql_backend)?;
    if updated == 0 {
        connection
            .execute(INSERT_DEBT, params)
            .map_err(sql_backend)?;
    }
    Ok(())
}

fn debt_from_row(row: &Row<'_>) -> rusqlite::Result<Debt> {
    let level: String = row.get(2)?;
    let closed_reason: Option<String> = row.get(11)?;

    Ok(Debt {
        id: Some(row.get(0)?),
        activity_id: row.get(1)?,
        level: parse_debt_level(&level).map_err(conversion)?,
        base_seconds: row.get(3)?,
        growth_per_day_seconds: row.get(4)?,
        cap_seconds: row.get(5)?,
        created_at: row.get(6)?,
        due_at: row.get(7)?,
        paid_seconds: row.get(8)?,
        closed_at: optional_timestamp(row, 9, 10, "debts.closed_at")?,
        closed_reason: match closed_reason {
            Some(text) => Some(parse_debt_close_reason(&text).map_err(conversion)?),
            None => None,
        },
    })
}

pub(super) fn list_open_debts(
    connection: &Connection,
    activity_id: i64,
) -> Result<Vec<Debt>, StoreError> {
    let sql = format!(
        "SELECT {DEBT_COLUMNS} FROM debts WHERE activity_id = ?1 AND closed_at IS NULL ORDER BY id"
    );
    query_all(connection, &sql, &[&activity_id], debt_from_row)
}

// ---------------------------------------------------------------------------
// Commitment contracts
// ---------------------------------------------------------------------------

const CONTRACT_COLUMNS: &str = "id, level, term_seconds, \
CAST(strftime('%s', started_at) AS INTEGER), CAST(strftime('%s', ends_at) AS INTEGER), \
params_snapshot, checksum, closed_at, CAST(strftime('%s', closed_at) AS INTEGER)";

const UPDATE_CONTRACT: &str = "UPDATE commitment_contracts SET level = :level, \
term_seconds = :term_seconds, started_at = datetime(:started_at, 'unixepoch'), \
ends_at = datetime(:ends_at, 'unixepoch'), params_snapshot = :params_snapshot, \
checksum = :checksum, closed_at = CASE WHEN :closed_at IS NULL THEN NULL ELSE \
datetime(:closed_at, 'unixepoch') END WHERE id = :id";

const INSERT_CONTRACT: &str = "INSERT INTO commitment_contracts (id, level, term_seconds, \
started_at, ends_at, params_snapshot, checksum, closed_at) VALUES (:id, :level, :term_seconds, \
datetime(:started_at, 'unixepoch'), datetime(:ends_at, 'unixepoch'), :params_snapshot, \
:checksum, CASE WHEN :closed_at IS NULL THEN NULL ELSE datetime(:closed_at, 'unixepoch') END)";

pub(super) fn upsert_contract(
    connection: &Connection,
    contract: &CommitmentContract,
) -> Result<(), StoreError> {
    let id = require_id(contract.id, "commitment contract")?;
    let level = strictness_text(contract.level);
    let params: &[(&str, &dyn ToSql)] = &[
        (":id", &id),
        (":level", &level),
        (":term_seconds", &contract.term_seconds),
        (":started_at", &contract.started_at),
        (":ends_at", &contract.ends_at),
        (":params_snapshot", &contract.params_snapshot),
        (":checksum", &contract.checksum),
        (":closed_at", &contract.closed_at),
    ];

    let updated = connection
        .execute(UPDATE_CONTRACT, params)
        .map_err(sql_backend)?;
    if updated == 0 {
        connection
            .execute(INSERT_CONTRACT, params)
            .map_err(sql_backend)?;
    }
    Ok(())
}

fn contract_from_row(row: &Row<'_>) -> rusqlite::Result<CommitmentContract> {
    let level: String = row.get(1)?;

    Ok(CommitmentContract {
        id: Some(row.get(0)?),
        level: parse_strictness(&level).map_err(conversion)?,
        term_seconds: row.get(2)?,
        started_at: row.get(3)?,
        ends_at: row.get(4)?,
        params_snapshot: row.get(5)?,
        checksum: row.get(6)?,
        closed_at: optional_timestamp(row, 7, 8, "commitment_contracts.closed_at")?,
    })
}

pub(super) fn load_active_contract(
    connection: &Connection,
) -> Result<Option<CommitmentContract>, StoreError> {
    let sql = format!(
        "SELECT {CONTRACT_COLUMNS} FROM commitment_contracts WHERE closed_at IS NULL \
         ORDER BY started_at DESC, id DESC LIMIT 1"
    );
    query_optional(connection, &sql, &[], contract_from_row)
}
