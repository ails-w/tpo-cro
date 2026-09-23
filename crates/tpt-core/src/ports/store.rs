//! Storage port.

use crate::domain::{Activity, CommitmentContract, Debt, Session, SessionGap, TimeEntry};
use crate::error::StoreError;

/// Persistence boundary for activities, time entries, sessions, gaps, debts and
/// the commitment contract. Projects and tags join in Phase 3.
pub trait Store {
    fn save_activity(&mut self, activity: &Activity) -> Result<i64, StoreError>;
    fn load_activity(&self, id: i64) -> Result<Option<Activity>, StoreError>;
    fn list_activities(&self, include_archived: bool) -> Result<Vec<Activity>, StoreError>;
    fn save_time_entry(&mut self, entry: &TimeEntry) -> Result<i64, StoreError>;
    fn list_time_entries(&self, activity_id: i64) -> Result<Vec<TimeEntry>, StoreError>;
    fn save_session(&mut self, session: &Session) -> Result<i64, StoreError>;
    fn load_session(&self, id: i64) -> Result<Option<Session>, StoreError>;
    fn save_gap(&mut self, gap: &SessionGap) -> Result<i64, StoreError>;
    fn save_debt(&mut self, debt: &Debt) -> Result<i64, StoreError>;
    fn list_open_debts(&self, activity_id: i64) -> Result<Vec<Debt>, StoreError>;
    fn save_contract(&mut self, contract: &CommitmentContract) -> Result<i64, StoreError>;
    fn load_active_contract(&self) -> Result<Option<CommitmentContract>, StoreError>;

    /// Blocks until everything enqueued so far has been applied to the
    /// database, returning (and clearing) the first deferred writer error, if
    /// there was one.
    ///
    /// With `synchronous = NORMAL` this means the writes are committed (for
    /// example, appended to the WAL); it does not force an fsync of the
    /// underlying storage.
    ///
    /// Adapters that write synchronously may implement this as a no-op.
    fn flush(&mut self) -> Result<(), StoreError>;
}
