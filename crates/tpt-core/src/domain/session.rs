//! Session models (extended in Feature 1.4).

use serde::{Deserialize, Serialize};

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
