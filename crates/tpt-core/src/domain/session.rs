//! Session models.

use serde::{Deserialize, Serialize};

use super::contract::RatingScale;

/// Strictness level in force for a session (ADR-004).
///
/// Serializes in `SCREAMING_SNAKE_CASE` so the textual form matches the values
/// stored by the SQLite schema (ADR-002).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StrictnessLevel {
    /// No presence tracking: pure stopwatch.
    Off,
    /// Measures and deducts gaps, without aborts or debt.
    #[default]
    L0,
    /// Strict contract (fixed preset).
    L1,
    /// Hard contract (fixed preset).
    L2,
}

/// Timer mode a session runs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionMode {
    /// Open-ended stopwatch.
    Flowtime,
    /// Work/break cycles.
    Pomodoro,
    /// Single focused block.
    Focus,
}

/// Lifecycle status of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionStatus {
    Running,
    Break,
    Awaiting,
    Suspended,
    Completed,
    Aborted,
    AbortedPenalized,
}

/// Why a session ended early.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AbortReason {
    User,
    Idle,
    ScreenOff,
    Suspended,
    DaemonKilled,
}

/// Kind of presence gap detected during a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GapKind {
    Idle,
    ScreenOff,
    Suspended,
}

/// A tracked work session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: Option<i64>,
    pub activity_id: i64,
    pub mode: SessionMode,
    pub target_seconds: Option<i64>,
    pub elapsed_seconds: i64,
    pub credited_seconds: i64,
    pub status: SessionStatus,
    pub cycle: u32,
    pub total_cycles: Option<u32>,
    pub strictness: StrictnessLevel,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub aborted_with: Option<AbortReason>,
    pub accumulated_gap_seconds: i64,
    pub gap_ratio: f64,
    pub rating: Option<u8>,
    pub rating_scale: Option<RatingScale>,
    pub reflection: Option<String>,
}

impl Session {
    /// A session cannot exist without an activity, and it always starts RUNNING.
    pub fn new(
        activity_id: i64,
        mode: SessionMode,
        strictness: StrictnessLevel,
        target_seconds: Option<i64>,
        total_cycles: Option<u32>,
        started_at: i64,
    ) -> Self {
        Self {
            id: None,
            activity_id,
            mode,
            target_seconds,
            elapsed_seconds: 0,
            credited_seconds: 0,
            status: SessionStatus::Running,
            cycle: 1,
            total_cycles,
            strictness,
            started_at,
            ended_at: None,
            aborted_with: None,
            accumulated_gap_seconds: 0,
            gap_ratio: 0.0,
            rating: None,
            rating_scale: None,
            reflection: None,
        }
    }
}

/// A presence gap recorded against a session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionGap {
    pub id: Option<i64>,
    pub session_id: i64,
    pub kind: GapKind,
    pub seconds: i64,
    pub deducted_seconds: i64,
    pub qualified: bool,
    pub strictness: StrictnessLevel,
    pub at: i64,
}

#[cfg(test)]
mod tests {
    use super::{Session, SessionMode, SessionStatus, StrictnessLevel};

    #[test]
    fn session_starts_running_with_activity() {
        let session = Session::new(
            7,
            SessionMode::Focus,
            StrictnessLevel::L1,
            Some(1500),
            None,
            1_700_000_000,
        );
        assert_eq!(session.activity_id, 7);
        assert_eq!(session.status, SessionStatus::Running);
        assert_eq!(session.cycle, 1);
        assert_eq!(session.elapsed_seconds, 0);
        assert_eq!(session.credited_seconds, 0);
        assert_eq!(session.target_seconds, Some(1500));
    }
}
