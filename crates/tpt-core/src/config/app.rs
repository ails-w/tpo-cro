//! Application configuration model (Feature 1.1).

use serde::{Deserialize, Serialize};

use super::preset::TimerPreset;
use crate::domain::StrictnessLevel;
use crate::error::ConfigError;

/// Root of the TOML configuration document.
///
/// Every section and field falls back to its default when absent, so a partial
/// (or even empty) document always parses. A document with no `[[timer_presets]]`
/// still parses but does not pass [`AppConfig::validate`]: presets are required.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub timers: TimersConfig,
    pub timer_presets: Vec<TimerPreset>,
    pub presence: PresenceConfig,
    pub commitment: CommitmentConfig,
    pub notifications: NotificationsConfig,
}

impl AppConfig {
    /// Parses a TOML document, applying defaults for missing sections/fields.
    pub fn from_toml_str(source: &str) -> Result<Self, ConfigError> {
        Ok(toml::from_str(source)?)
    }

    /// Validates business rules. Returns the first violation found.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.timer_presets.is_empty() {
            return Err(ConfigError::NoPresets);
        }

        if !self
            .timer_presets
            .iter()
            .any(|preset| preset.name == self.timers.default_preset)
        {
            return Err(ConfigError::UnknownDefaultPreset(
                self.timers.default_preset.clone(),
            ));
        }

        for preset in &self.timer_presets {
            preset.validate(
                self.timers.max_short_break_minutes,
                self.timers.max_long_break_minutes,
            )?;
        }

        if !(self.presence.warn_after_seconds < self.presence.strong_warn_after_seconds
            && self.presence.strong_warn_after_seconds < self.presence.min_gap_seconds)
        {
            return Err(ConfigError::PresenceThresholdOrder);
        }

        Ok(())
    }
}

/// Paths and retention policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub db_path: String,
    pub socket_path: String,
    pub notes_retention_days: u32,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            db_path: "~/.local/share/tpt/metrics.db".to_string(),
            socket_path: "$XDG_RUNTIME_DIR/tpt.sock".to_string(),
            notes_retention_days: 365,
        }
    }
}

/// Timer engine parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TimersConfig {
    pub default_preset: String,
    pub max_short_break_minutes: u32,
    pub max_long_break_minutes: u32,
    pub max_warnings_per_session: u32,
    pub pomodoro_cycles: u32,
    pub require_break_confirmation: bool,
    pub awaiting_expiry_minutes: u32,
    pub focus_view: FocusView,
}

impl Default for TimersConfig {
    fn default() -> Self {
        Self {
            default_preset: "preset-25-5".to_string(),
            max_short_break_minutes: 15,
            max_long_break_minutes: 20,
            max_warnings_per_session: 3,
            pomodoro_cycles: 4,
            require_break_confirmation: true,
            awaiting_expiry_minutes: 45,
            focus_view: FocusView::default(),
        }
    }
}

/// Focus view layer (ADR-004).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FocusView {
    /// Full screen plus beacon and confirmation on exit.
    #[default]
    Soft,
    /// Blocks TUI navigation for the duration of the session.
    Locked,
}

/// Presence measurement parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PresenceConfig {
    pub level: StrictnessLevel,
    pub min_gap_seconds: u32,
    pub gap_deduction: GapDeduction,
    pub suspend_min_seconds: u32,
    pub warn_after_seconds: u32,
    pub strong_warn_after_seconds: u32,
    pub require_heartbeat: bool,
    pub heartbeat_max_age_seconds: u32,
    pub abort_counter_window: AbortCounterWindow,
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self {
            level: StrictnessLevel::default(),
            min_gap_seconds: 420,
            gap_deduction: GapDeduction::default(),
            suspend_min_seconds: 60,
            warn_after_seconds: 180,
            strong_warn_after_seconds: 300,
            require_heartbeat: true,
            heartbeat_max_age_seconds: 60,
            abort_counter_window: AbortCounterWindow::default(),
        }
    }
}

/// How a qualifying gap is deducted from credited time.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GapDeduction {
    /// Deduct the whole gap.
    #[default]
    Full,
    /// Deduct only the seconds beyond the qualifying threshold.
    Excess,
}

/// Window over which abort counters are accumulated.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AbortCounterWindow {
    /// The currently active contract.
    #[default]
    Contract,
    /// The current calendar day.
    Day,
    /// The current session.
    Session,
}

/// Commitment contract parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CommitmentConfig {
    pub min_term_days: u32,
}

impl Default for CommitmentConfig {
    fn default() -> Self {
        Self { min_term_days: 1 }
    }
}

/// Notification channels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationsConfig {
    pub desktop: bool,
    pub sound: bool,
    pub sound_file: String,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            desktop: true,
            sound: true,
            sound_file: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AbortCounterWindow, AppConfig, FocusView, GapDeduction};
    use crate::config::TimerPreset;
    use crate::domain::StrictnessLevel;
    use crate::error::ConfigError;

    const SHIPPED_CONFIG: &str = include_str!("../../../../config.toml");

    fn valid_preset(name: &str) -> TimerPreset {
        TimerPreset {
            name: name.to_string(),
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 20,
            cycles_before_long: 4,
        }
    }

    #[test]
    fn app_config_parse_valid_toml_returns_expected() {
        let config = AppConfig::from_toml_str(SHIPPED_CONFIG).expect("shipped config must parse");
        assert_eq!(config.timers.default_preset, "preset-25-5");
        assert_eq!(config.timers.max_short_break_minutes, 15);
        assert_eq!(config.timers.max_long_break_minutes, 20);
        assert_eq!(config.timers.pomodoro_cycles, 4);
        assert!(config.timers.require_break_confirmation);
        assert_eq!(config.timers.awaiting_expiry_minutes, 45);
        assert_eq!(config.timer_presets.len(), 4);
        assert_eq!(config.presence.level, StrictnessLevel::L0);
        assert_eq!(config.presence.min_gap_seconds, 420);
        assert_eq!(config.presence.gap_deduction, GapDeduction::Full);
        assert_eq!(
            config.presence.abort_counter_window,
            AbortCounterWindow::Contract
        );
        assert!(config.presence.require_heartbeat);
        assert_eq!(config.general.notes_retention_days, 365);
        assert_eq!(config.commitment.min_term_days, 1);
        assert!(config.notifications.desktop);
        assert!(config.notifications.sound);
        config
            .validate()
            .expect("shipped config must satisfy its own rules");
    }

    #[test]
    fn app_config_parse_minimal_toml_applies_defaults() {
        let config = AppConfig::from_toml_str("").expect("empty document must use defaults");
        assert_eq!(config.presence.level, StrictnessLevel::L0);
        assert_eq!(config.timers.max_short_break_minutes, 15);
        assert_eq!(config.general.notes_retention_days, 365);
        assert!(config.timer_presets.is_empty());
    }

    #[test]
    fn app_config_parse_reads_values_from_document() {
        let document = r#"
            [general]
            notes_retention_days = 30

            [timers]
            default_preset = "custom"
            pomodoro_cycles = 6
            awaiting_expiry_minutes = 15
            focus_view = "locked"

            [presence]
            level = "L1"
            min_gap_seconds = 600
            strong_warn_after_seconds = 400
            warn_after_seconds = 200
            gap_deduction = "excess"
            abort_counter_window = "day"
            require_heartbeat = false

            [commitment]
            min_term_days = 7

            [notifications]
            desktop = false
            sound_file = "/tmp/bell.ogg"

            [[timer_presets]]
            name = "custom"
            work_minutes = 35
            short_break_minutes = 7
            long_break_minutes = 18
            cycles_before_long = 3
        "#;

        let config = AppConfig::from_toml_str(document).expect("custom document must parse");

        assert_eq!(config.general.notes_retention_days, 30);
        assert_eq!(config.timers.default_preset, "custom");
        assert_eq!(config.timers.pomodoro_cycles, 6);
        assert_eq!(config.timers.awaiting_expiry_minutes, 15);
        assert_eq!(config.timers.focus_view, FocusView::Locked);
        assert_eq!(config.presence.level, StrictnessLevel::L1);
        assert_eq!(config.presence.min_gap_seconds, 600);
        assert_eq!(config.presence.strong_warn_after_seconds, 400);
        assert_eq!(config.presence.warn_after_seconds, 200);
        assert_eq!(config.presence.gap_deduction, GapDeduction::Excess);
        assert_eq!(
            config.presence.abort_counter_window,
            AbortCounterWindow::Day
        );
        assert!(!config.presence.require_heartbeat);
        assert_eq!(config.commitment.min_term_days, 7);
        assert!(!config.notifications.desktop);
        assert_eq!(config.notifications.sound_file, "/tmp/bell.ogg");
        assert_eq!(config.timer_presets.len(), 1);
        assert_eq!(config.timer_presets[0].work_minutes, 35);
        assert_eq!(config.timer_presets[0].cycles_before_long, 3);
        config
            .validate()
            .expect("a document with non-default values can still be valid");
    }

    #[test]
    fn app_config_validate_rejects_no_presets() {
        let config = AppConfig::default();

        let error = config
            .validate()
            .expect_err("a configuration without timer presets must be rejected");

        assert!(matches!(error, ConfigError::NoPresets));
    }

    #[test]
    fn app_config_validate_rejects_unknown_default_preset() {
        let config = AppConfig {
            timer_presets: vec![valid_preset("other-preset")],
            ..AppConfig::default()
        };

        let error = config
            .validate()
            .expect_err("a default preset missing from timer_presets must be rejected");

        assert!(matches!(
            error,
            ConfigError::UnknownDefaultPreset(name) if name == "preset-25-5"
        ));
    }

    #[test]
    fn app_config_validate_rejects_preset_over_break_limit() {
        let config = AppConfig {
            timer_presets: vec![TimerPreset {
                short_break_minutes: 16,
                ..valid_preset("preset-25-5")
            }],
            ..AppConfig::default()
        };

        let error = config
            .validate()
            .expect_err("a preset exceeding the short break limit must be rejected");

        assert!(matches!(
            error,
            ConfigError::ShortBreakTooLong {
                value: 16,
                max: 15,
                ..
            }
        ));
    }

    #[test]
    fn app_config_validate_rejects_presence_thresholds_out_of_order() {
        let config = AppConfig {
            timer_presets: vec![valid_preset("preset-25-5")],
            presence: super::PresenceConfig {
                warn_after_seconds: 300,
                strong_warn_after_seconds: 180,
                ..super::PresenceConfig::default()
            },
            ..AppConfig::default()
        };

        let error = config
            .validate()
            .expect_err("warn thresholds must be strictly increasing");

        assert!(matches!(error, ConfigError::PresenceThresholdOrder));
    }
}
