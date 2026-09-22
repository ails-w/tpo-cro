//! Timer presets (Features 1.1 and 1.2).

use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

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

impl TimerPreset {
    /// Enforces the hard rules from ADR-004 §2: short break <= max_short, long
    /// break <= max_long.
    pub fn validate(
        &self,
        max_short_break_minutes: u32,
        max_long_break_minutes: u32,
    ) -> Result<(), ConfigError> {
        if self.short_break_minutes > max_short_break_minutes {
            return Err(ConfigError::ShortBreakTooLong {
                preset: self.name.clone(),
                value: self.short_break_minutes,
                max: max_short_break_minutes,
            });
        }

        if self.long_break_minutes > max_long_break_minutes {
            return Err(ConfigError::LongBreakTooLong {
                preset: self.name.clone(),
                value: self.long_break_minutes,
                max: max_long_break_minutes,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::TimerPreset;
    use crate::error::ConfigError;

    #[test]
    fn timer_preset_short_break_over_15_rejected() {
        let preset = TimerPreset {
            name: "too-long-short-break".to_string(),
            work_minutes: 25,
            short_break_minutes: 16,
            long_break_minutes: 20,
            cycles_before_long: 4,
        };

        let error = preset
            .validate(15, 20)
            .expect_err("a 16 minute short break must be rejected when the max is 15");

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
    fn timer_preset_long_break_over_20_rejected() {
        let preset = TimerPreset {
            name: "too-long-long-break".to_string(),
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 21,
            cycles_before_long: 4,
        };

        let error = preset
            .validate(15, 20)
            .expect_err("a 21 minute long break must be rejected when the max is 20");

        assert!(matches!(
            error,
            ConfigError::LongBreakTooLong {
                value: 21,
                max: 20,
                ..
            }
        ));
    }

    #[test]
    fn timer_preset_within_limits_is_accepted() {
        let preset = TimerPreset {
            name: "at-the-limit".to_string(),
            work_minutes: 90,
            short_break_minutes: 15,
            long_break_minutes: 20,
            cycles_before_long: 1,
        };

        preset
            .validate(15, 20)
            .expect("breaks exactly at the maximum must be accepted");
    }
}
