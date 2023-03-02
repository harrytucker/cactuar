//! # Cactuar Controller
//!
//! This is the main binary for the Cactuar project, it implements a Kubernetes
//! controller, and exposes its own Prometheus metrics.
//!
//! The controller is intended to be deployed with its Helm chart into a
//! Kubernetes cluster, but can also be deployed on your local machine, provided
//! you have configured `kubectl` access already.

use color_eyre::Result;
use tokio::signal;

use cactuar::{
    config::CactuarConfig, http::router, kubernetes::controller::controller_future, logging,
};

#[tokio::main]
async fn main() -> Result<()> {
    let config = CactuarConfig::new()?;

    let subscriber = logging::new_subscriber()?;
    logging::install_observability(subscriber)?;

    // Start kubernetes controller
    let control_future = controller_future().await;
    tokio::task::Builder::new()
        .name("K8s Controller")
        .spawn(control_future)?;

    tracing::info!("Cactuar now controlling resources");

    let serve_addr = config.http.serve_addr();
    let http_future = axum::Server::bind(&serve_addr).serve(router()?.into_make_service());
    tokio::task::Builder::new()
        .name("HTTP Server")
        .spawn(http_future)?;

    tracing::info!(%serve_addr, "Cactuar ready to serve requests");

    signal::ctrl_c().await?;
    Ok(())
}
