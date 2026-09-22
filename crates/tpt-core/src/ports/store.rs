//! Storage port.

use crate::domain::{Activity, CommitmentContract, Debt, Session, SessionGap, TimeEntry};
use crate::error::StoreError;

/// Persistence boundary for every domain record.
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
}
