use std::{collections::BTreeMap, result::Result, sync::Arc};

use chrono::Utc;
use futures::future::BoxFuture;
use futures::FutureExt;
use futures::StreamExt;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::controller::Controller;
use kube::{
    api::{Api, ListParams, Patch, PatchParams, ResourceExt},
    client::Client,
    runtime::{
        controller::Action,
        events::{Event, EventType, Recorder, Reporter},
    },
    Resource,
};
use serde_json::json;

use tokio::time::Duration;
use uuid::Uuid;

use crate::kubernetes::reconciler::Error;
use crate::prometheus;
use crate::service_alerts::{
    ServiceAlert, ServiceAlertStatus, API_GROUP, API_VERSION, FINALIZER_NAME, KIND,
};

use super::reconciler::{self, Context};

/// Builds a [`Controller`] future that controls `ServiceAlerts` that own
/// `ConfigMaps`. To begin controlling Kubernetes resources, the caller should
/// `.await` the returned future, or spawn it on an executor, such as
/// [`tokio::task`].
pub async fn controller_future() -> BoxFuture<'static, ()> {
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
        .expect("is the crd installed? please run: `cargo run --bin crdgen | kubectl apply -f -`");

    // All good. Start controller and return its future.
    Controller::new(service_alerter_api, ListParams::default())
        .owns(config_map_api, ListParams::default())
        .run(reconciler::reconcile, reconciler::error_policy, context)
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()))
        .boxed()
}
