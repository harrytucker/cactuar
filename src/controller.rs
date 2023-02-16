use std::sync::Arc;

use chrono::Utc;
use futures::future::BoxFuture;
use futures::FutureExt;
use futures::StreamExt;
use kube::{
    api::{Api, ListParams, Patch, PatchParams, ResourceExt},
    client::Client,
    runtime::{
        controller::Action,
        events::{Event, EventType, Recorder, Reporter},
        finalizer::{finalizer, Event as Finalizer},
    },
    Resource,
};
use serde_json::json;
use std::result::Result;
use thiserror::Error;
use tokio::time::Duration;
use uuid::Uuid;

use crate::service_alerts::{
    ServiceAlerts, ServiceAlertsStatus, API_GROUP, API_VERSION, FINALIZER_NAME, KIND,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Finalizer Error: {0}")]
    FinalizerError(#[source] kube::runtime::finalizer::Error<kube::Error>),

    #[error("SerializationError: {0}")]
    SerializationError(#[source] serde_json::Error),
}

// Context for our reconciler
#[derive(Clone)]
struct Context {
    /// Kubernetes client
    client: Client,
    reporter: Reporter,
}

async fn reconcile(obj: Arc<ServiceAlerts>, ctx: Arc<Context>) -> Result<Action, Error> {
    tracing::info!(object=?obj, "received request");

    let client = ctx.client.clone();
    let ns = obj.namespace().unwrap();
    let docs: Api<ServiceAlerts> = Api::namespaced(client, &ns);

    let action = finalizer(&docs, FINALIZER_NAME, obj, |event| async {
        match event {
            Finalizer::Apply(doc) => doc.reconcile(ctx.clone()).await,
            Finalizer::Cleanup(doc) => doc.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(Error::FinalizerError);

    action
}

fn error_policy(service_alert: Arc<ServiceAlerts>, error: &Error, _ctx: Arc<Context>) -> Action {
    let requeue_interval = 5*60;
    tracing::info!(
        %error,
        ?service_alert,
        requeue_interval,
        "reconciliation failed, re-queueing"
    );
    Action::requeue(Duration::from_secs(requeue_interval))
}

impl ServiceAlerts {
    // Reconcile (for non-finalizer related changes)
    async fn reconcile(&self, ctx: Arc<Context>) -> Result<Action, kube::Error> {
        tracing::info!(
            service_alert=?self,
            "reconciling request"
        );

        let client = ctx.client.clone();
        let name = self.name_any();
        let ns = self.namespace().unwrap();
        let docs: Api<ServiceAlerts> = Api::namespaced(client, &ns);

        let new_status = Patch::Apply(json!({
            "apiVersion": format!("{}/{}", API_GROUP, API_VERSION),
            "kind": KIND,
            "status": ServiceAlertsStatus{
                last_reconciled_at: Some(Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string()),
                reconciliation_expires_at: Some((Utc::now() + chrono::Duration::seconds(30)).format("%Y-%m-%dT%H:%M:%S").to_string()),
            }
        }));
        let ps = PatchParams::apply("cntrlr").force();
        let _o = docs.patch_status(&name, &ps, &new_status).await?;

        // If no events were received, check back every 5 minutes
        Ok(Action::requeue(Duration::from_secs(5*60)))
    }

    // Reconcile with finalize cleanup (the object was deleted)
    async fn cleanup(&self, ctx: Arc<Context>) -> Result<Action, kube::Error> {
        tracing::info!(
            service_alert=?self,
            "deleting resource",
        );

        let recorder = Recorder::new(
            ctx.client.clone(),
            ctx.reporter.clone(),
            self.object_ref(&()),
        );

        recorder
            .publish(Event {
                type_: EventType::Normal,
                reason: "Delete".into(),
                note: Some(format!("Delete `{}`", self.name_any())),
                action: "Reconciling".into(),
                secondary: None,
            })
            .await?;

        Ok(Action::await_change())
    }
}

#[derive(Clone)]
pub struct CactuarController {}

/// Example Controller that owns a Controller for ServiceAlerts
impl CactuarController {
    /// Lifecycle initialization interface for app
    ///
    /// This returns a `Controller` that drives a `Controller` + a future to be awaited
    /// It is up to `main` to wait for the controller stream.
    pub async fn new() -> (Self, BoxFuture<'static, ()>) {
        let client = Client::try_default().await.expect("create client");
        let context = Arc::new(Context {
            client: client.clone(),
            reporter: Reporter {
                controller: FINALIZER_NAME.into(),
                instance: Some(Uuid::new_v4().to_string()),
            },
        });

        let service_alerter = Api::<ServiceAlerts>::all(client);

        // Ensure CRD is installed before loop-watching
        let _ = service_alerter
            .list(&ListParams::default().limit(1))
            .await
            .expect(
                "is the crd installed? please run: `cargo run --bin crdgen | kubectl apply -f -`",
            );

        // All good. Start controller and return its future.

        let controller =
            kube::runtime::controller::Controller::new(service_alerter, ListParams::default())
                .run(reconcile, error_policy, context)
                .filter_map(|x| async move { std::result::Result::ok(x) })
                .for_each(|_| futures::future::ready(()))
                .boxed();

        (Self {}, controller)
    }
}
