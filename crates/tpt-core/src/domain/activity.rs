//! Activity models (Feature 1.3).

use serde::{Deserialize, Serialize};

use super::schedule::ScheduleRule;

/// How an activity accrues time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrackingMode {
    /// Time is credited by the timer engine.
    Timer,
    /// Time is entered by hand.
    Manual,
}

/// A unit of work that time is tracked against.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    pub id: Option<i64>,
    pub name: String,
    pub project_id: Option<i64>,
    pub target_minutes: Option<u32>,
    pub schedule_rule: Option<ScheduleRule>,
    pub tracking_mode: TrackingMode,
    pub pending_extra_ratio: f64,
    pub archived: bool,
    pub created_at: i64,
}

impl Activity {
    /// Creates a new activity owned by nobody and scheduled nowhere.
    pub fn new(name: impl Into<String>, tracking_mode: TrackingMode, created_at: i64) -> Self {
        Self {
            id: None,
            name: name.into(),
            project_id: None,
            target_minutes: None,
            schedule_rule: None,
            tracking_mode,
            pending_extra_ratio: 0.0,
            archived: false,
            created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Activity, TrackingMode};

    #[test]
    fn activity_without_schedule_is_valid() {
        let activity = Activity::new("Read a book", TrackingMode::Manual, 1_700_000_000);
        assert!(activity.schedule_rule.is_none());
        assert_eq!(activity.tracking_mode, TrackingMode::Manual);
        assert_eq!(activity.name, "Read a book");
    }

    #[test]
    fn activity_requires_tracking_mode() {
        let json = r#"{
            "id": null,
            "name": "No mode",
            "project_id": null,
            "target_minutes": null,
            "schedule_rule": null,
            "pending_extra_ratio": 0.0,
            "archived": false,
            "created_at": 1700000000
        }"#;

        let error = serde_json::from_str::<Activity>(json)
            .expect_err("deserializing an activity without tracking_mode must fail");

        assert!(
            error.to_string().contains("tracking_mode"),
            "the error must point at the missing tracking_mode, got: {error}"
        );
    }
}
