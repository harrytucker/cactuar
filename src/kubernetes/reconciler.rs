use kube::{Api, ResourceExt};
use std::{sync::Arc, time::Duration};

use kube::{
    runtime::{controller::Action, events::Reporter, finalizer::Event},
    Client,
};
use thiserror::Error;

use crate::service_alerts::{ServiceAlert, FINALIZER_NAME};

// Context for our reconciler
#[derive(Clone)]
pub struct Context {
    /// Kubernetes client
    pub client: Client,
    pub reporter: Reporter,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Finalizer Error: {0}")]
    FinalizerError(#[source] kube::runtime::finalizer::Error<kube::Error>),
    // #[error("SerializationError: {0}")]
    // SerializationError(#[source] serde_json::Error),
    #[error("Failed to create ConfigMap: {0}")]
    ConfigMapCreationFailed(#[source] kube::Error),
    // #[error("MissingObjectKey: {0}")]
    // MissingObjectKey(&'static str),
}

#[tracing::instrument(skip(ctx, crd))]
pub async fn reconcile(crd: Arc<ServiceAlert>, ctx: Arc<Context>) -> Result<Action, Error> {
    tracing::info!(
        name=?crd.metadata.name.as_ref().unwrap(),
        namespace=?crd.metadata.namespace.as_ref().unwrap(),
        "received reconcile request"
    );

    let ns = crd.namespace().unwrap();
    let api: Api<ServiceAlert> = Api::namespaced(ctx.client.clone(), &ns);

    kube::runtime::finalizer(&api, FINALIZER_NAME, crd, |event| async {
        match event {
            Event::Apply(alert) => alert.create_or_update(ctx.clone()).await,
            Event::Cleanup(alert) => alert.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(Error::FinalizerError)
}

#[tracing::instrument(skip(_ctx, crd))]
pub fn error_policy(crd: Arc<ServiceAlert>, error: &Error, _ctx: Arc<Context>) -> Action {
    let requeue_interval = 60;
    tracing::error!(
        %error,
        name=?crd.metadata.name.as_ref().unwrap(),
        namespace=?crd.metadata.namespace.as_ref().unwrap(),
        requeue_interval,
        "reconciliation failed, re-queueing"
    );

    Action::requeue(Duration::from_secs(requeue_interval))
}
