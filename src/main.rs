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
    api::{ListParams, Patch, PatchParams},
    Api, Client, CustomResourceExt,
};
use service_alerts::ServiceAlerter;

/// Identifier that is recorded by the Kubernetes API for the purpose of
/// identifying the application responsible for the given Kubernetes resource.
const MANAGER_STRING: &str = "cactuar";

/// CustomResourceDefinition name for the ServiceAlerter type, the FQDN (Fully
/// Qualified Domain Name) serves as a way to namespace custom resources in
/// Kubernetes.
const CUSTOM_RESOURCE_NAME: &str = "servicealerters.cactuar.rs";

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Configuration support

    let subscriber = logging::new_subscriber(tracing::Level::INFO)?;
    logging::set_global_logger(subscriber)?;

    let client = Client::try_default().await?;
    let service_alerters: Api<ServiceAlerter> = Api::all(client.clone());
    let custom_resources: Api<CustomResourceDefinition> = Api::all(client.clone());

    tracing::info!("Discovering existing ServiceAlerts in cluster.");
    let lp = ListParams::default();
    service_alerters
        .list(&lp)
        .await?
        .iter()
        .for_each(|service_alert| {
            tracing::info!(
                service_alert.metadata.name,
                service_alert.metadata.namespace,
                "Discovered ServiceAlert!"
            )
        });

    tracing::info!("Patching ServiceAlert CustomResourceDefinition.");
    custom_resources
        .patch(
            CUSTOM_RESOURCE_NAME,
            &PatchParams::apply(MANAGER_STRING),
            &Patch::Apply(ServiceAlerter::crd()),
        )
        .await?;

    // TODO: How to handle CRD deployment?

    // TODO: Launch reconciler in background

    Ok(())
}
