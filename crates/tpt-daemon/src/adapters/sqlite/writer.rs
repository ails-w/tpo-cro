//! Dedicated writer thread for the SQLite store.
//!
//! Callers stamp an id and enqueue a command; the thread owns the only write
//! connection and applies the SQL serially. A write failure is recorded as the
//! first deferred error and reported (and cleared) by the next `Flush`; the
//! thread never short-circuits, so one failing write does not discard the
//! commands queued behind it.

use std::sync::mpsc::{Receiver, Sender};
#[cfg(test)]
use std::time::Duration;

use rusqlite::Connection;

use tpt_core::domain::{Activity, CommitmentContract, Debt, Session, SessionGap, TimeEntry};
use tpt_core::error::StoreError;

use super::{backend, mapping, sql_backend};

/// Bounded queue capacity: a full queue applies backpressure instead of
/// growing memory without limit.
pub(super) const QUEUE_CAPACITY: usize = 1024;

/// Upper bound for the test seam, so a forgotten release can never hang the suite.
#[cfg(test)]
const BLOCK_TIMEOUT: Duration = Duration::from_secs(10);

/// Commands processed serially by the writer thread.
pub(super) enum WriteCommand {
    SaveActivity(Activity),
    SaveTimeEntry(TimeEntry),
    SaveSession(Session),
    SaveGap(SessionGap),
    SaveDebt(Debt),
    SaveContract(CommitmentContract),
    Checkpoint(Sender<Result<(), StoreError>>),
    Flush(Sender<Result<(), StoreError>>),
    Shutdown,
    #[cfg(test)]
    BlockUntil(Receiver<()>),
}

/// Runs until `Shutdown` arrives or every sender is dropped.
pub(super) fn run(connection: Connection, receiver: Receiver<WriteCommand>) {
    let mut first_error: Option<StoreError> = None;

    while let Ok(command) = receiver.recv() {
        match command {
            WriteCommand::SaveActivity(activity) => {
                record(
                    &mut first_error,
                    mapping::upsert_activity(&connection, &activity),
                );
            }
            WriteCommand::SaveTimeEntry(entry) => {
                record(
                    &mut first_error,
                    mapping::upsert_time_entry(&connection, &entry),
                );
            }
            WriteCommand::SaveSession(session) => {
                record(
                    &mut first_error,
                    mapping::upsert_session(&connection, &session),
                );
            }
            WriteCommand::SaveGap(gap) => {
                record(&mut first_error, mapping::upsert_gap(&connection, &gap));
            }
            WriteCommand::SaveDebt(debt) => {
                record(&mut first_error, mapping::upsert_debt(&connection, &debt));
            }
            WriteCommand::SaveContract(contract) => {
                record(
                    &mut first_error,
                    mapping::upsert_contract(&connection, &contract),
                );
            }
            WriteCommand::Checkpoint(reply) => {
                let _ = reply.send(checkpoint(&connection));
            }
            WriteCommand::Flush(reply) => {
                let result = match first_error.take() {
                    Some(error) => Err(error),
                    None => Ok(()),
                };
                let _ = reply.send(result);
            }
            WriteCommand::Shutdown => break,
            #[cfg(test)]
            WriteCommand::BlockUntil(release) => {
                let _ = release.recv_timeout(BLOCK_TIMEOUT);
            }
        }
    }
}

/// Keeps the first write error and ignores the following ones.
fn record(first_error: &mut Option<StoreError>, result: Result<(), StoreError>) {
    if first_error.is_none() {
        if let Err(error) = result {
            *first_error = Some(error);
        }
    }
}

/// Truncates the WAL and reclaims the free pages left by deleted rows.
///
/// `wal_checkpoint(TRUNCATE)` reports how many readers blocked it through its
/// `busy` flag; a non-zero value means the truncation was incomplete.
pub(super) fn checkpoint(connection: &Connection) -> Result<(), StoreError> {
    let busy: i64 = connection
        .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))
        .map_err(sql_backend)?;
    if busy != 0 {
        return Err(backend(format!(
            "wal_checkpoint(TRUNCATE) is incomplete (busy = {busy})"
        )));
    }

    connection
        .execute_batch("PRAGMA incremental_vacuum;")
        .map_err(sql_backend)?;
    Ok(())
}
