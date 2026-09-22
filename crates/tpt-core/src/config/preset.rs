//! Timer presets (Features 1.1 and 1.2).

use serde::{Deserialize, Serialize};

/// A configurable work/break cycle preset from `config.toml`.
///
/// Every field is required in TOML: a preset without an explicit break length
/// is a configuration mistake, not a default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimerPreset {
    pub name: String,
    pub work_minutes: u32,
    pub short_break_minutes: u32,
    pub long_break_minutes: u32,
    pub cycles_before_long: u32,
}
