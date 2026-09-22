//! Configuration models and validation rules (Features 1.1 and 1.2).

pub mod app;
pub mod preset;

pub use app::{
    AbortCounterWindow, AppConfig, CommitmentConfig, FocusView, GapDeduction, GeneralConfig,
    NotificationsConfig, PresenceConfig, TimersConfig,
};
pub use preset::TimerPreset;
