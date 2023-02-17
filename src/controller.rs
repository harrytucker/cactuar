use std::{collections::BTreeMap, sync::Arc};
use std::result::Result;

use chrono::Utc;
use futures::future::BoxFuture;
use futures::FutureExt;
use futures::StreamExt;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{
    api::{Api, ListParams, Patch, PatchParams, ResourceExt},
    client::Client,
    Resource,
    runtime::{
        controller::Action,
        events::{Event, EventType, Recorder, Reporter},
        finalizer::{Event as Finalizer, finalizer},
    },
};
use serde_json::json;
use thiserror::Error;
use tokio::time::Duration;
use uuid::Uuid;

use crate::service_alerts::{
    API_GROUP, API_VERSION, FINALIZER_NAME, KIND, ServiceAlert, ServiceAlertStatus,
};

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

// Context for our reconciler
#[derive(Clone)]
struct Context {
    /// Kubernetes client
    client: Client,
    reporter: Reporter,
}

async fn reconcile(obj: Arc<ServiceAlert>, ctx: Arc<Context>) -> Result<Action, Error> {
    tracing::info!(
        name=?obj.metadata.name.as_ref().unwrap(),
        namespace=?obj.metadata.namespace.as_ref().unwrap(),
        "received reconcile request"
    );

    // let client = ctx.client.clone();
    let ns = obj.namespace().unwrap();
    let api: Api<ServiceAlert> = Api::namespaced(ctx.client.clone(), &ns);

    let action = finalizer(&api, FINALIZER_NAME, obj, |event| async {
        match event {
            Finalizer::Apply(alert) => alert.create_or_update(ctx.clone()).await,
            Finalizer::Cleanup(alert) => alert.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(Error::FinalizerError);

    action
}

fn error_policy(service_alert: Arc<ServiceAlert>, error: &Error, _ctx: Arc<Context>) -> Action {
    let requeue_interval = 60;
    tracing::error!(
        %error,
        name=?service_alert.metadata.name.as_ref().unwrap(),
        namespace=?service_alert.metadata.namespace.as_ref().unwrap(),
        requeue_interval,
        "reconciliation failed, re-queueing"
    );

    Action::requeue(Duration::from_secs(requeue_interval))
}

impl ServiceAlert {
    // Reconcile (for non-finalizer related changes)
    async fn create_or_update(&self, ctx: Arc<Context>) -> Result<Action, kube::Error> {
        tracing::info!(
            name=?&self.metadata.name.as_ref().unwrap(),
            namespace=?&self.metadata.namespace.as_ref().unwrap(),
            "reconciling request"
        );
        let name = self.name_any();
        let namespace = self.namespace().unwrap();

        let service_alert_api: Api<ServiceAlert> = Api::namespaced(ctx.client.clone(), &namespace);
        let config_map_api: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &namespace);

        let mut labels = BTreeMap::new();
        labels.insert("rules".to_string(), "prom-rule".to_string());

        let mut data = BTreeMap::new();
        data.insert("message".to_string(), "hello :)".to_string());

        tracing::info!(
            name=?&self.metadata.name.as_ref().unwrap(),
            namespace=?&self.metadata.namespace.as_ref().unwrap(),
            "building configmap"
        );
        let cm = ConfigMap {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                namespace: Some(namespace),
                // This label is what allows prometheus to pick up the configMap
                labels: Some(labels),

                // finalizers: Some(vec![FINALIZER_NAME]),
                owner_references: Some(vec![self.controller_owner_ref(&()).unwrap()]),
                ..ObjectMeta::default()
            },
            data: Some(data),
            // data: Some(contents),
            ..Default::default()
        };

        tracing::info!(
            name=?&self.metadata.name.as_ref().unwrap(),
            namespace=?&self.metadata.namespace.as_ref().unwrap(),
            "patching configmap"
        );
        config_map_api
            .patch(
                &name,
                &PatchParams::apply(FINALIZER_NAME),
                &Patch::Apply(&cm),
            )
            .await
            .map_err(Error::ConfigMapCreationFailed)
            .unwrap();

        tracing::info!(
            name=?&self.metadata.name.as_ref().unwrap(),
            namespace=?&self.metadata.namespace.as_ref().unwrap(),
            "building ServiceAlert status"
        );

        let requeue_duration: u64 = 5 * 60;
        let new_status = Patch::Apply(json!({
            "apiVersion": format!("{}/{}", API_GROUP, API_VERSION),
            "kind": KIND,
            "status": ServiceAlertStatus{
                last_reconciled_at: Some(Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string()),
                reconciliation_expires_at: Some((Utc::now() + chrono::Duration::seconds(requeue_duration.clone() as i64)).format("%Y-%m-%dT%H:%M:%S").to_string()),
            }
        }));

        tracing::info!(
            name=?&self.metadata.name.as_ref().unwrap(),
            namespace=?&self.metadata.namespace.as_ref().unwrap(),
            "building ServiceAlert status"
        );
        let ps = PatchParams::apply("cntrlr").force();

        tracing::info!(
            name=?&self.metadata.name.as_ref().unwrap(),
            namespace=?&self.metadata.namespace.as_ref().unwrap(),
            "getting ServiceAlert patch status"
        );
        let _o = service_alert_api
            .patch_status(&name, &ps, &new_status)
            .await?;

        // If no events were received, check back every 5 minutes
        Ok(Action::requeue(Duration::from_secs(
            requeue_duration.clone(),
        )))
    }

    // Reconcile with finalize cleanup (the object was deleted)
    async fn cleanup(&self, ctx: Arc<Context>) -> Result<Action, kube::Error> {
        tracing::info!(
            name=?&self.metadata.name.as_ref().unwrap(),
            namespace=?&self.metadata.namespace.as_ref().unwrap(),
            "deleting resource",
        );

        let recorder = Recorder::new(
            ctx.client.clone(),
            ctx.reporter.clone(),
            self.object_ref(&()),
        );

        // TODO: delete the configmap

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

/// Example Controller that owns a Controller for ServiceAlert
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

        let service_alerter_api = Api::<ServiceAlert>::all(client.clone());
        let config_map_api = Api::<ConfigMap>::all(client.clone());

        // Ensure CRD is installed before loop-watching
        let _ = service_alerter_api
            .list(&ListParams::default().limit(1))
            .await
            .expect(
                "is the crd installed? please run: `cargo run --bin crdgen | kubectl apply -f -`",
            );

        // All good. Start controller and return its future.

        let controller =
            kube::runtime::controller::Controller::new(service_alerter_api, ListParams::default())
                .owns(config_map_api, ListParams::default())
                .run(reconcile, error_policy, context)
                .filter_map(|x| async move { std::result::Result::ok(x) })
                .for_each(|_| futures::future::ready(()))
                .boxed();

        (Self {}, controller)
    }
}
