//! Note retention: nulls old notes without deleting rows.

use rusqlite::Connection;

use tpt_core::error::StoreError;

use super::sql_backend;

/// Nulls `time_entries.note` and `sessions.reflection` older than `days`,
/// returning how many rows were touched.
///
/// `days == 0` disables retention. Rows are never deleted: the day/time filters
/// only decide which notes expire. `time_entries.day` is a local calendar day,
/// so its cutoff uses localtime; `sessions.started_at` is a UTC datetime, so
/// its cutoff stays in UTC. Free pages are reclaimed with `incremental_vacuum`.
pub(super) fn purge_expired_notes(connection: &Connection, days: u32) -> Result<usize, StoreError> {
    if days == 0 {
        return Ok(0);
    }

    let modifier = format!("-{days} days");
    let entries = connection
        .execute(
            "UPDATE time_entries SET note = NULL WHERE note IS NOT NULL \
             AND day < date('now', 'localtime', ?1)",
            [&modifier],
        )
        .map_err(sql_backend)?;
    let sessions = connection
        .execute(
            "UPDATE sessions SET reflection = NULL WHERE reflection IS NOT NULL \
             AND started_at < datetime('now', ?1)",
            [&modifier],
        )
        .map_err(sql_backend)?;

    connection
        .execute_batch("PRAGMA incremental_vacuum;")
        .map_err(sql_backend)?;

    Ok(entries + sessions)
}
