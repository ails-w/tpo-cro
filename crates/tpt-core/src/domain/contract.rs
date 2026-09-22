//! Commitment contract and reflection models (Feature 1.4).

use serde::{Deserialize, Serialize};

use super::session::StrictnessLevel;

/// Scale used for the end-of-session rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RatingScale {
    Five,
    Seven,
    Ten,
}

impl RatingScale {
    /// Largest value of the scale.
    #[must_use]
    pub const fn value(self) -> u8 {
        match self {
            Self::Five => 5,
            Self::Seven => 7,
            Self::Ten => 10,
        }
    }
}

/// Optional end-of-session reflection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reflection {
    pub rating: Option<u8>,
    pub rating_scale: Option<RatingScale>,
    pub note: Option<String>,
}

impl Reflection {
    /// A reflection where nothing was filled in.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            rating: None,
            rating_scale: None,
            note: None,
        }
    }
}

/// A signed commitment to a strictness level for a fixed term.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitmentContract {
    pub id: Option<i64>,
    pub level: StrictnessLevel,
    pub term_seconds: i64,
    pub started_at: i64,
    pub ends_at: i64,
    /// JSON snapshot of every parameter of the level (ADR-004 §10); produced in Phase 5.
    pub params_snapshot: String,
    /// Covers level + term + params_snapshot.
    pub checksum: String,
    pub closed_at: Option<i64>,
}
