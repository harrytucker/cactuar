//! # Cactuar Config
//!
//! This module defines the configuration structure for Cactuar, and implements
//! support for merging a final config from various information sources. In
//! order of priority (i.e. item in list *overrides* the next item in list),
//! config sources are merged as follows:
//!
//! - Environment variables
//! - Config file: `cactuar.toml`
//! - Default values
//!
//! ## Examples
//!
//! ### Environment variables
//!
//! ```bash
//! HTTP_ADDRESS=127.0.0.1 \
//! HTTP_PORT=80 \
//! LOG_LEVEL=debug \
//! cargo run --bin controller
//! ```
//!
//! ### Config file: `cactuar.toml`
//!
//! ```toml
//! [log]
//! level = "info"
//!
//! [http]
//! address = "0.0.0.0"
//! port = 8080
//! ```

use std::{
    fmt::Debug,
    net::{IpAddr, SocketAddr},
};

use config::Config;
use serde::Deserialize;

#[derive(Default, Debug, Deserialize)]
#[serde(default)]
/// Forms the tree structure for CactuarConfig. This implementation relies on
/// using the [`Default`] trait, along with the `#[serde(default)]` annotation
/// macro to provide default values for each config value.
///
/// If you need to add a new config value, make sure you either derive
/// [`Default`] if all your sub-types implement the trait, or provide your own
/// implementation of it.
pub struct CactuarConfig {
    pub log: Log,
    pub http: HTTP,
}

#[derive(Default, Debug, Deserialize)]
/// This struct contains a [`LogLevel`] enum in order to provide the serialised
/// structure we expect.
///
/// It also provides the environment variable naming, i.e. Log { level }
/// becomes: LOG_LEVEL
pub struct Log {
    pub level: LogLevel,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// Our own representation of logging levels to decouple from the
/// [`tracing::Level`] representations.
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

#[derive(Debug, Deserialize)]
pub struct HTTP {
    pub address: IpAddr,
    pub port: u16,
}

impl HTTP {
    pub fn serve_addr(&self) -> SocketAddr {
        SocketAddr::new(self.address, self.port)
    }
}

// By default, Cactuar should expect external clients, and thus should bind to
// `0.0.0.0`. Port `8080` is simply commonly used as a non-standard HTTP port.
impl Default for HTTP {
    fn default() -> Self {
        Self {
            address: [0, 0, 0, 0].into(),
            port: 8080,
        }
    }
}

// We want to be able to convert our `LogLevel` enum to their [`tracing::Level`]
// representation, so we implement the [`From`] trait here.
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

impl CactuarConfig {
    /// Create a new [`CactuarConfig`]. This function merges default config
    /// values, config file values, and environment variables, please refer to
    /// the [`config module documentation`](crate::config) for more information.
    pub fn new() -> Result<CactuarConfig, config::ConfigError> {
        let builder = Config::builder()
            .add_source(config::File::with_name("cactuar").required(false))
            .add_source(config::Environment::default().separator("_"))
            .build()?;

        builder.try_deserialize()
    }
}
