//! Notifier port.

use crate::domain::{SessionMode, StrictnessLevel};
use crate::error::NotifyError;

/// A user-facing notification emitted by the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notification {
    IdleWarning { elapsed_seconds: i64 },
    StrongIdleWarning { elapsed_seconds: i64 },
    ScreenOff { elapsed_seconds: i64 },
    BlockEnded { activity_id: i64, mode: SessionMode },
    ContractEscalation { level: StrictnessLevel },
}

/// Delivers notifications through whichever channels are enabled.
pub trait Notifier {
    fn notify(&mut self, notification: &Notification) -> Result<(), NotifyError>;
}
