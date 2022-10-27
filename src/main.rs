//! # Cactuar!
//!
//! ```text
//!          \ | /
//!          /¯¯¯\
//!  /¯\    | o o |
//! |   |   |  0  |
//! |   |__|      |____
//!  \____         ___  \
//!        |       |  |  |
//!  /¯¯¯¯        |    \/
//! |   |¯¯¯¯¯|  |
//! |   |     |  |____
//!  \_/       \______)
//! ```
//!
//! Kubernetes operator for creating Prometheus alerts using standard metrics
//! emitted by an Istio sidecar container.
//!
//! # TODO
//! - Implement transformation from CRD (ServiceAlerter) spec into Prometheus
//!   alert rules
//! - Implement reconciler to ensure consistent state between deployed CRDs and
//!   Prometheus alerting rules
//! - Potentially implement component to load alert rules directly into a
//!   Prometheus deployment inside of a Kubernetes cluster
//! - Tests
//! - Cargo Makefile
//! - Project architecture

pub mod logging;
pub mod service_alerts;

use color_eyre::Result;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{Patch, PatchParams},
    Api, Client, CustomResourceExt,
};
use service_alerts::ServiceAlerter;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = logging::new_subscriber(tracing::Level::INFO)?;
    logging::set_global_logger(subscriber)?;

    let client = Client::try_default().await?;
    let crds: Api<CustomResourceDefinition> = Api::all(client.clone());

    // logs a hello world message along with the debug representation of the
    // above K8S resource spec (theoretically)
    tracing::info!("Hello, world!");
    crds.patch(
        "servicealerters.cactuar.rs",
        &PatchParams::apply("cactuar"),
        &Patch::Apply(ServiceAlerter::crd()),
    )
    .await?;

    Ok(())
}
