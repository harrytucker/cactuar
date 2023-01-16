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

mod logging;
mod service_alerts;
mod watcher;

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
    let discover_alerters = match service_alerters.list(&lp).await {
        Ok(alerters) => alerters,
        Err(error) => {
            tracing::error!(%error, "ServiceAlert discovery failed.");
            explain_kube_err(&error);
            return Err(error.into());
        }
    };

    discover_alerters.iter().for_each(|service_alert| {
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
    tokio::spawn(watcher::watch_for_events(service_alerters)).await??;
    Ok(())
}

fn explain_kube_err(err: &kube::Error) {
    match err {
        kube::Error::Api(_) => todo!(),
        kube::Error::HyperError(_) => {
            tracing::info!("Transport issue detected, am I running in a Kubernetes cluster?")
        }
        kube::Error::Service(_) => todo!(),
        kube::Error::FromUtf8(_) => todo!(),
        kube::Error::LinesCodecMaxLineLengthExceeded => todo!(),
        kube::Error::ReadEvents(_) => todo!(),
        kube::Error::HttpError(_) => todo!(),
        kube::Error::SerdeError(_) => todo!(),
        kube::Error::BuildRequest(_) => todo!(),
        kube::Error::InferConfig(_) => todo!(),
        kube::Error::Discovery(_) => todo!(),
        kube::Error::OpensslTls(_) => todo!(),
        kube::Error::Auth(_) => tracing::info!("Failed to authenticate with the Kubernetes API."),
    }
}
