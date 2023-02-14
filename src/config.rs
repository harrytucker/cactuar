use std::fmt::Debug;

use config::{Config, ValueKind};
use serde::Deserialize;

const DEFAULT_LOG_LEVEL: &str = "info";

#[derive(Debug, Deserialize)]
pub struct CactuarConfig {
    pub log: Log,
}

#[derive(Debug, Deserialize)]
pub struct Log {
    #[serde(default)]
    pub level: LogLevel,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => Self::ERROR,
            LogLevel::Warn => Self::WARN,
            LogLevel::Info => Self::INFO,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Trace => Self::TRACE,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl CactuarConfig {
    /// Reads application configuration from either a `config.toml` file, or from
    /// environment variables.
    pub fn new() -> Result<CactuarConfig, config::ConfigError> {
        let builder = Config::builder()
            .add_source(config::File::with_name("cactuar").required(false))
            .add_source(config::Environment::default().separator("_"))
            .set_default("log.level", ValueKind::String(DEFAULT_LOG_LEVEL.into()))?
            .build()?;

        builder.try_deserialize()
    }
}
