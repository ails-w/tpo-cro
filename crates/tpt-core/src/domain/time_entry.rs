//! Time entries (Feature 1.4).

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// Where a time entry came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeEntrySource {
    /// Entered by hand.
    Manual,
    /// Credited when a session ended.
    Session,
    /// Brought in by an importer.
    Imported,
    /// Debt penalty.
    Penalty,
}

impl std::fmt::Display for TimeEntrySource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Manual => "MANUAL",
            Self::Session => "SESSION",
            Self::Imported => "IMPORTED",
            Self::Penalty => "PENALTY",
        };
        formatter.write_str(label)
    }
}

// `thiserror` treats any field named `source` as the error source, so the enum
// must satisfy the `Error` bound even though it is a plain domain value.
impl std::error::Error for TimeEntrySource {}

/// An amount of time credited to an activity on a calendar day.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: Option<i64>,
    pub activity_id: i64,
    pub source: TimeEntrySource,
    pub seconds: i64,
    pub day: String,
    pub note: Option<String>,
    pub session_id: Option<i64>,
    pub external_id: Option<String>,
    pub created_at: i64,
}

impl TimeEntry {
    /// Only `PENALTY` entries may carry a negative amount (mirrors the ADR-002 CHECK).
    pub fn new(
        activity_id: i64,
        source: TimeEntrySource,
        seconds: i64,
        day: impl Into<String>,
        created_at: i64,
    ) -> Result<Self, DomainError> {
        if seconds < 0 && source != TimeEntrySource::Penalty {
            return Err(DomainError::NegativeSeconds { source, seconds });
        }

        Ok(Self {
            id: None,
            activity_id,
            source,
            seconds,
            day: day.into(),
            note: None,
            session_id: None,
            external_id: None,
            created_at,
        })
    }

    /// Creates a manual entry from a non-negative amount of seconds.
    pub fn manual(
        activity_id: i64,
        seconds: u32,
        day: impl Into<String>,
        created_at: i64,
    ) -> Result<Self, DomainError> {
        Self::new(
            activity_id,
            TimeEntrySource::Manual,
            i64::from(seconds),
            day,
            created_at,
        )
    }
}

/// Total credited seconds for one activity, computed from entries — never stored.
pub fn activity_total_seconds(entries: &[TimeEntry], activity_id: i64) -> i64 {
    entries
        .iter()
        .filter(|entry| entry.activity_id == activity_id)
        .map(|entry| entry.seconds)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{TimeEntry, TimeEntrySource, activity_total_seconds};
    use crate::error::DomainError;

    #[test]
    fn time_entry_rejects_negative_seconds_outside_penalty() {
        let error = TimeEntry::new(7, TimeEntrySource::Manual, -60, "2026-09-21", 1_700_000_000)
            .expect_err("a MANUAL entry cannot carry negative seconds");

        match error {
            DomainError::NegativeSeconds { source, seconds } => {
                assert_eq!(source, TimeEntrySource::Manual);
                assert_eq!(seconds, -60);
            }
        }
    }

    #[test]
    fn penalty_time_entry_accepts_negative_seconds() {
        let entry = TimeEntry::new(
            7,
            TimeEntrySource::Penalty,
            -300,
            "2026-09-21",
            1_700_000_000,
        )
        .expect("a PENALTY entry may carry negative seconds");

        assert_eq!(entry.source, TimeEntrySource::Penalty);
        assert_eq!(entry.seconds, -300);
    }

    #[test]
    fn manual_time_entry_adds_to_activity_total() {
        let day = "2026-09-21";
        let entries = vec![
            TimeEntry::manual(7, 1800, day, 1_700_000_000).expect("valid manual entry"),
            TimeEntry::manual(7, 600, day, 1_700_000_100).expect("valid manual entry"),
            TimeEntry::manual(9, 9999, day, 1_700_000_200).expect("valid manual entry"),
        ];
        assert_eq!(activity_total_seconds(&entries, 7), 2400);
    }
}
