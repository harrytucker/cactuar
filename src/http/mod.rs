//! # HTTP
//!
//! This module contains [`axum`] HTTP handlers, as well as a router that
//! exposes a readiness check for Kubernetes and Prometheus metrics.

mod metrics;
mod readiness;
mod router;

// HTTP Router
pub use router::router;

// HTTP Handlers
pub use metrics::prometheus_handler;
pub use readiness::readiness_handler;
