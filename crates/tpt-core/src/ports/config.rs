//! Configuration port.

use crate::config::AppConfig;
use crate::error::ConfigError;

/// Loads the application configuration from its backing source.
pub trait ConfigSource {
    fn load(&self) -> Result<AppConfig, ConfigError>;
}
