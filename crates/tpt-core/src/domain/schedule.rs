//! Scheduling rules (Feature 1.3). Behavior arrives in Phase 3.

use serde::{Deserialize, Serialize};

/// Day of the week for explicit schedules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

/// When an activity is expected to happen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScheduleRule {
    /// Monday through Friday.
    Weekdays,
    /// An explicit set of weekdays.
    SpecificDays(Vec<Weekday>),
    /// Every day of the week.
    EveryDay,
    /// Inclusive ISO-8601 (`YYYY-MM-DD`) date range, validated in Phase 3.
    DateRange { start: String, end: String },
}
