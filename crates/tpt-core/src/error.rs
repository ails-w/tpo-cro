//! Crate error types.

use crate::domain::TimeEntrySource;

/// Errors produced by domain invariants.
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("a {source:?} time entry cannot carry negative seconds ({seconds})")]
    NegativeSeconds {
        source: TimeEntrySource,
        seconds: i64,
    },
}

/// Errors produced while loading or validating the application configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to parse configuration: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("preset '{preset}': short break of {value} min exceeds the maximum of {max} min")]
    ShortBreakTooLong {
        preset: String,
        value: u32,
        max: u32,
    },
    #[error("preset '{preset}': long break of {value} min exceeds the maximum of {max} min")]
    LongBreakTooLong {
        preset: String,
        value: u32,
        max: u32,
    },
    #[error("default preset '{0}' does not exist in timer_presets")]
    UnknownDefaultPreset(String),
    #[error("at least one timer preset is required")]
    NoPresets,
    #[error(
        "presence thresholds must satisfy warn_after_seconds < strong_warn_after_seconds < min_gap_seconds"
    )]
    PresenceThresholdOrder,
}

/// Errors produced by the storage port.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("storage backend unavailable: {message}")]
    Backend { message: String },
    #[error("record not found: {kind} #{id}")]
    NotFound { kind: &'static str, id: i64 },
}

/// Errors produced by the notifier port.
#[derive(Debug, thiserror::Error)]
#[error("notification failed: {0}")]
pub struct NotifyError(pub String);

/// Errors produced by the IPC transport port.
#[derive(Debug, thiserror::Error)]
#[error("ipc transport failed: {0}")]
pub struct IpcError(pub String);
