//! Debt models (Feature 1.4).
//!
//! Growth and expiry are computed from `created_at` — no maintenance job.

use serde::{Deserialize, Serialize};

/// Severity of a debt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DebtLevel {
    L1,
    L2,
}

/// How a debt stopped being open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DebtCloseReason {
    Paid,
    Refinanced,
    Expired,
}

/// An outstanding obligation created by aborted sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Debt {
    pub id: Option<i64>,
    pub activity_id: i64,
    pub level: DebtLevel,
    pub base_seconds: i64,
    pub growth_per_day_seconds: i64,
    pub cap_seconds: i64,
    pub created_at: i64,
    pub due_at: i64,
    pub paid_seconds: i64,
    pub closed_at: Option<i64>,
    pub closed_reason: Option<DebtCloseReason>,
}
