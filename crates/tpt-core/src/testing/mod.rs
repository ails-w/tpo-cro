//! In-memory fakes for the core ports.
//!
//! Compiled only for tests or when the `testing` feature is enabled, so the
//! daemon can reuse them in its own test suite.

use std::cell::Cell;
use std::collections::VecDeque;

use crate::config::AppConfig;
use crate::domain::{Activity, CommitmentContract, Debt, Session, SessionGap, TimeEntry};
use crate::error::{ConfigError, IpcError, NotifyError, StoreError};
use crate::ports::{
    Clock, ConfigSource, IpcTransport, Notification, Notifier, PresenceEvent, PresenceSource, Store,
};

/// A deterministic clock with independent monotonic and wall readings.
#[derive(Debug)]
pub struct FakeClock {
    monotonic: Cell<i64>,
    wall: Cell<i64>,
}

impl FakeClock {
    /// Creates a clock whose monotonic and wall readings both start at `start_wall`.
    #[must_use]
    pub fn new(start_wall: i64) -> Self {
        Self {
            monotonic: Cell::new(start_wall),
            wall: Cell::new(start_wall),
        }
    }

    /// Advances both clocks.
    pub fn advance(&self, seconds: i64) {
        self.monotonic.set(self.monotonic.get() + seconds);
        self.wall.set(self.wall.get() + seconds);
    }

    /// Simulates a suspend: the wall clock moves forward while the monotonic
    /// clock stays frozen, which is exactly the gap ADR-008 detects.
    pub fn advance_monotonic_only(&self, seconds: i64) {
        self.wall.set(self.wall.get() + seconds);
    }
}

impl Clock for FakeClock {
    fn now_monotonic(&self) -> i64 {
        self.monotonic.get()
    }

    fn now_wall(&self) -> i64 {
        self.wall.get()
    }
}

/// A presence source that replays a fixed queue of events.
#[derive(Debug, Default)]
pub struct FakePresence {
    events: VecDeque<PresenceEvent>,
}

impl FakePresence {
    /// Creates a source that will report `events` in order.
    #[must_use]
    pub fn new(events: Vec<PresenceEvent>) -> Self {
        Self {
            events: events.into(),
        }
    }

    /// Appends an event to the queue.
    pub fn push(&mut self, event: PresenceEvent) {
        self.events.push_back(event);
    }
}

impl PresenceSource for FakePresence {
    fn poll(&mut self) -> Option<PresenceEvent> {
        self.events.pop_front()
    }
}

/// Assigns `id` to `record`, replacing the stored record with the same id.
fn upsert<T>(
    records: &mut Vec<T>,
    record: T,
    id: i64,
    id_of: impl Fn(&T) -> Option<i64>,
) -> Result<i64, StoreError> {
    if let Some(existing) = records.iter_mut().find(|item| id_of(item) == Some(id)) {
        *existing = record;
    } else {
        records.push(record);
    }
    Ok(id)
}

/// A correct in-memory implementation of [`Store`] for tests.
#[derive(Debug, Default)]
pub struct InMemoryStore {
    activities: Vec<Activity>,
    time_entries: Vec<TimeEntry>,
    sessions: Vec<Session>,
    gaps: Vec<SessionGap>,
    debts: Vec<Debt>,
    contracts: Vec<CommitmentContract>,
    next_id: i64,
}

impl InMemoryStore {
    fn allocate_id(&mut self) -> i64 {
        self.next_id += 1;
        self.next_id
    }
}

impl Store for InMemoryStore {
    fn save_activity(&mut self, activity: &Activity) -> Result<i64, StoreError> {
        let mut record = activity.clone();
        let id = match record.id {
            Some(id) => id,
            None => {
                let id = self.allocate_id();
                record.id = Some(id);
                id
            }
        };
        upsert(&mut self.activities, record, id, |item| item.id)
    }

    fn load_activity(&self, id: i64) -> Result<Option<Activity>, StoreError> {
        Ok(self
            .activities
            .iter()
            .find(|item| item.id == Some(id))
            .cloned())
    }

    fn list_activities(&self, include_archived: bool) -> Result<Vec<Activity>, StoreError> {
        Ok(self
            .activities
            .iter()
            .filter(|item| include_archived || !item.archived)
            .cloned()
            .collect())
    }

    fn save_time_entry(&mut self, entry: &TimeEntry) -> Result<i64, StoreError> {
        let mut record = entry.clone();
        let id = match record.id {
            Some(id) => id,
            None => {
                let id = self.allocate_id();
                record.id = Some(id);
                id
            }
        };
        upsert(&mut self.time_entries, record, id, |item| item.id)
    }

    fn list_time_entries(&self, activity_id: i64) -> Result<Vec<TimeEntry>, StoreError> {
        Ok(self
            .time_entries
            .iter()
            .filter(|item| item.activity_id == activity_id)
            .cloned()
            .collect())
    }

    fn save_session(&mut self, session: &Session) -> Result<i64, StoreError> {
        let mut record = session.clone();
        let id = match record.id {
            Some(id) => id,
            None => {
                let id = self.allocate_id();
                record.id = Some(id);
                id
            }
        };
        upsert(&mut self.sessions, record, id, |item| item.id)
    }

    fn load_session(&self, id: i64) -> Result<Option<Session>, StoreError> {
        Ok(self
            .sessions
            .iter()
            .find(|item| item.id == Some(id))
            .cloned())
    }

    fn save_gap(&mut self, gap: &SessionGap) -> Result<i64, StoreError> {
        let mut record = gap.clone();
        let id = match record.id {
            Some(id) => id,
            None => {
                let id = self.allocate_id();
                record.id = Some(id);
                id
            }
        };
        upsert(&mut self.gaps, record, id, |item| item.id)
    }

    fn save_debt(&mut self, debt: &Debt) -> Result<i64, StoreError> {
        let mut record = debt.clone();
        let id = match record.id {
            Some(id) => id,
            None => {
                let id = self.allocate_id();
                record.id = Some(id);
                id
            }
        };
        upsert(&mut self.debts, record, id, |item| item.id)
    }

    fn list_open_debts(&self, activity_id: i64) -> Result<Vec<Debt>, StoreError> {
        Ok(self
            .debts
            .iter()
            .filter(|item| item.activity_id == activity_id && item.closed_at.is_none())
            .cloned()
            .collect())
    }

    fn save_contract(&mut self, contract: &CommitmentContract) -> Result<i64, StoreError> {
        let mut record = contract.clone();
        let id = match record.id {
            Some(id) => id,
            None => {
                let id = self.allocate_id();
                record.id = Some(id);
                id
            }
        };
        upsert(&mut self.contracts, record, id, |item| item.id)
    }

    fn load_active_contract(&self) -> Result<Option<CommitmentContract>, StoreError> {
        Ok(self
            .contracts
            .iter()
            .find(|item| item.closed_at.is_none())
            .cloned())
    }
}

/// A configuration source that always returns the same snapshot.
#[derive(Debug, Clone, Default)]
pub struct StaticConfigSource {
    config: AppConfig,
}

impl StaticConfigSource {
    /// Creates a source that serves `config`.
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}

impl ConfigSource for StaticConfigSource {
    fn load(&self) -> Result<AppConfig, ConfigError> {
        Ok(self.config.clone())
    }
}

/// A notifier that records every notification it receives.
#[derive(Debug, Default)]
pub struct RecordingNotifier {
    notifications: Vec<Notification>,
}

impl RecordingNotifier {
    /// Every notification received so far, in order.
    #[must_use]
    pub fn notifications(&self) -> &[Notification] {
        &self.notifications
    }
}

impl Notifier for RecordingNotifier {
    fn notify(&mut self, notification: &Notification) -> Result<(), NotifyError> {
        self.notifications.push(notification.clone());
        Ok(())
    }
}

/// An IPC transport with in-memory queues.
#[derive(Debug, Default)]
pub struct FakeTransport {
    outbound: Vec<Vec<u8>>,
    inbound: VecDeque<Vec<u8>>,
}

impl FakeTransport {
    /// Creates a transport that will hand out `inbound` payloads in order.
    #[must_use]
    pub fn new(inbound: Vec<Vec<u8>>) -> Self {
        Self {
            outbound: Vec::new(),
            inbound: inbound.into(),
        }
    }

    /// Every payload sent so far, in order.
    #[must_use]
    pub fn outbound(&self) -> &[Vec<u8>] {
        &self.outbound
    }
}

impl IpcTransport for FakeTransport {
    fn send(&mut self, payload: &[u8]) -> Result<(), IpcError> {
        self.outbound.push(payload.to_vec());
        Ok(())
    }

    fn recv(&mut self) -> Result<Option<Vec<u8>>, IpcError> {
        Ok(self.inbound.pop_front())
    }
}
