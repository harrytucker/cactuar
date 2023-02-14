use std::{
    fmt::Debug,
    net::{IpAddr, SocketAddr},
};

use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CactuarConfig {
    pub http: Option<HTTP>,

    pub log: Log,
}

#[derive(Debug, Deserialize)]
pub struct HTTP {
    #[serde(default = "default_serve_address")]
    pub address: IpAddr,
    pub port: u16,
}

impl HTTP {
    pub fn serve_addr(&self) -> SocketAddr {
        SocketAddr::new(self.address, self.port)
    }
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

/// Reads application configuration from either a `config.toml` file, or from
/// environment variables.
pub fn get_configuration() -> Result<CactuarConfig, config::ConfigError> {
    let builder = Config::builder()
        .add_source(config::Environment::default())
        .add_source(config::File::with_name("config"))
        .build()?;

    builder.try_deserialize()
}

fn default_serve_address() -> IpAddr {
    "0.0.0.0"
        .parse()
        .expect("failed to parse default grpc address")
}
