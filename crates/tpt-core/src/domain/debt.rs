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

impl Debt {
    /// Pending amount per ADR-004 section 6: the cap applies to the base, the
    /// daily growth is added on top, and growth stops at the expiry.
    #[must_use]
    pub fn outstanding_seconds(&self, now: i64) -> i64 {
        if self.closed_at.is_some() {
            return 0;
        }
        let until = now.min(self.due_at);
        let days = (until - self.created_at).max(0) / 86_400;
        let base = self.base_seconds.min(self.cap_seconds);
        (base + self.growth_per_day_seconds * days - self.paid_seconds).max(0)
    }

    /// Expiry is computed, with no maintenance job.
    #[must_use]
    pub fn is_expired(&self, now: i64) -> bool {
        self.closed_at.is_none() && now >= self.due_at
    }
}

#[cfg(test)]
mod tests {
    use super::{Debt, DebtCloseReason, DebtLevel};

    fn sample_debt() -> Debt {
        Debt {
            id: None,
            activity_id: 7,
            level: DebtLevel::L1,
            base_seconds: 300,
            growth_per_day_seconds: 300,
            cap_seconds: 1200,
            created_at: 1_700_000_000,
            due_at: 1_700_000_000 + 15 * 86_400,
            paid_seconds: 0,
            closed_at: None,
            closed_reason: None,
        }
    }

    #[test]
    fn debt_amount_is_computed_from_created_at() {
        let debt = sample_debt();
        let now = debt.created_at + 4 * 86_400;

        assert_eq!(debt.outstanding_seconds(now), 1500);
    }

    #[test]
    fn debt_stops_growing_at_due_at() {
        let debt = sample_debt();
        let now = debt.created_at + 30 * 86_400;

        assert_eq!(debt.outstanding_seconds(now), 4800);
    }

    #[test]
    fn debt_base_is_capped() {
        let mut debt = sample_debt();
        debt.base_seconds = 2000;
        let now = debt.created_at;

        assert_eq!(debt.outstanding_seconds(now), 1200);
    }

    #[test]
    fn debt_amount_subtracts_paid_seconds() {
        let mut debt = sample_debt();
        debt.paid_seconds = 1000;
        let now = debt.created_at + 4 * 86_400;

        assert_eq!(debt.outstanding_seconds(now), 500);
    }

    #[test]
    fn debt_amount_never_goes_below_zero() {
        let mut debt = sample_debt();
        debt.paid_seconds = 5000;
        let now = debt.created_at + 4 * 86_400;

        assert_eq!(debt.outstanding_seconds(now), 0);
    }

    #[test]
    fn closed_debt_has_no_outstanding_amount() {
        let mut debt = sample_debt();
        debt.closed_at = Some(debt.created_at + 86_400);
        debt.closed_reason = Some(DebtCloseReason::Paid);
        let now = debt.created_at + 30 * 86_400;

        assert_eq!(debt.outstanding_seconds(now), 0);
        assert!(!debt.is_expired(now));
    }

    #[test]
    fn debt_expires_at_due_at() {
        let debt = sample_debt();

        assert!(!debt.is_expired(debt.due_at - 1));
        assert!(debt.is_expired(debt.due_at));
    }
}
