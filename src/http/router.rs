use axum::routing;
use color_eyre::Result;
use prometheus::{process_collector::ProcessCollector, Registry};

use super::{prometheus_handler, readiness_handler};

// HTTP router paths:
const READINESS_CHECK_PATH: &str = "/ready";
const METRICS_PATH: &str = "/metrics";

/// Produces top level HTTP router that can be exposed by an [`axum::Server`]
pub fn router() -> Result<axum::Router> {
    let process_collector = ProcessCollector::for_self();
    let prometheus_registry = Registry::new();
    prometheus_registry.register(Box::new(process_collector))?;

    Ok(axum::Router::new()
        .route(READINESS_CHECK_PATH, routing::get(readiness_handler))
        .route(METRICS_PATH, routing::get(prometheus_handler))
        .with_state(prometheus_registry))
}
