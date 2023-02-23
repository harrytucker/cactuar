use std::sync::Arc;

use futures::future::BoxFuture;
use futures::FutureExt;
use futures::StreamExt;
use k8s_openapi::api::core::v1::ConfigMap;

use kube::runtime::controller::Controller;
use kube::{
    api::{Api, ListParams},
    client::Client,
    runtime::events::Reporter,
};

use uuid::Uuid;

use crate::service_alerts::{ServiceAlert, FINALIZER_NAME};

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

    // If the CRD isn't installed, there isn't much our Controller can do.
    let _ = service_alerter_api
        .list(&ListParams::default().limit(1))
        .await
        .expect("is the crd installed? please run: `cargo run --bin crdgen | kubectl apply -f -`");

    // All good. Box the future for the client to `.await`
    Controller::new(service_alerter_api, ListParams::default())
        .owns(config_map_api, ListParams::default())
        .run(reconciler::reconcile, reconciler::error_policy, context)
        .for_each(|_| futures::future::ready(()))
        .boxed()
}
