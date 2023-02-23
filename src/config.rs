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
//! Note that the RUST_LOG environment variable is provided by
//! [`tracing_subscriber::EnvFilter`] in the [`crate::logging`] module.
//!
//! ```bash
//! HTTP_ADDRESS=127.0.0.1 \
//! HTTP_PORT=80 \
//! RUST_LOG=info \
//! cargo run --bin controller
//! ```
//!
//! ### Config file: `cactuar.toml`
//!
//! ```toml
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
    pub http: HTTP,
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
