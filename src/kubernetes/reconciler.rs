use kube::{Api, ResourceExt};
use std::{sync::Arc, time::Duration};

use kube::{
    runtime::{controller::Action, events::Reporter, finalizer::Event},
    Client,
};
use thiserror::Error;

use super::operations::OperationError;
use crate::crd::{ServiceAlert, FINALIZER_NAME};

const FAIL_REQUEUE_DURATION: u64 = 10;

// Context for our reconciler
#[derive(Clone)]
pub struct Context {
    /// Kubernetes client
    pub client: Client,
    pub reporter: Reporter,
}

#[derive(Debug, Error)]
pub enum ReconcilerError {
    #[error("Finalizer Error: {0}")]
    Finalizer(#[source] kube::runtime::finalizer::Error<OperationError>),
    #[error("MissingObjectKey: {0}")]
    MissingObjectKey(&'static str),
}

#[tracing::instrument(skip(ctx, crd), fields(crd.metadata.name, crd.metadata.namespace))]
pub async fn reconcile(
    crd: Arc<ServiceAlert>,
    ctx: Arc<Context>,
) -> Result<Action, ReconcilerError> {
    tracing::info!("Beginning reconciliation");

    let ns = crd
        .namespace()
        .ok_or_else(|| ReconcilerError::MissingObjectKey("namespace"))?;
    let api: Api<ServiceAlert> = Api::namespaced(ctx.client.clone(), &ns);

    kube::runtime::finalizer(&api, FINALIZER_NAME, crd, |event| async {
        match event {
            Event::Apply(alert) => alert.update(ctx.clone()).await,
            Event::Cleanup(alert) => alert.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(ReconcilerError::Finalizer)
}

#[tracing::instrument(skip(_crd, _ctx), fields(crd.metadata.name, crd.metadata.namespace))]
pub fn error_policy(
    _crd: Arc<ServiceAlert>,
    error: &ReconcilerError,
    _ctx: Arc<Context>,
) -> Action {
    // All of our owned resources are entirely contained within Kubernetes, so
    // if we encounter an error, we can just requeue reconciliation.
    tracing::error!(FAIL_REQUEUE_DURATION, "Reconciliation failed, re-queueing");
    Action::requeue(Duration::from_secs(FAIL_REQUEUE_DURATION))
}
