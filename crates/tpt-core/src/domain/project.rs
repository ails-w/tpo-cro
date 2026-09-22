//! Projects (Feature 1.3).

use serde::{Deserialize, Serialize};

/// A grouping of activities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub color_hex: Option<String>,
    pub created_at: i64,
}
