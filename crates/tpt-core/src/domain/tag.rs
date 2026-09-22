//! Tags (Feature 1.3).

use serde::{Deserialize, Serialize};

/// A free-form label attachable to activities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
}
