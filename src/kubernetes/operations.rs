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

impl ServiceAlert {
    // Reconcile (for non-finalizer related changes)
    pub async fn create_or_update(&self, ctx: Arc<Context>) -> Result<Action, kube::Error> {
        tracing::info!(
            name=?&self.metadata.name.as_ref().unwrap(),
            namespace=?&self.metadata.namespace.as_ref().unwrap(),
            "reconciling request"
        );
        let name = self.name_any();
        let namespace = self.namespace().unwrap();

        let service_alert_api: Api<ServiceAlert> = Api::namespaced(ctx.client.clone(), &namespace);
        let config_map_api: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &namespace);

        let prom_alert = prometheus::alert::PromAlerts::try_from(self.spec.clone()).unwrap();

        let mut labels = BTreeMap::new();
        labels.insert("rules".to_string(), "prom-rule".to_string());

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
            data: Some(BTreeMap::try_from(prom_alert).unwrap()),
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
            "apiVersion": format!("{API_GROUP}/{API_VERSION}"),
            "kind": KIND,
            "status": ServiceAlertStatus{
                last_reconciled_at: Some(Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string()),
                reconciliation_expires_at: Some((Utc::now() + chrono::Duration::seconds(requeue_duration as i64)).format("%Y-%m-%dT%H:%M:%S").to_string()),
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
        Ok(Action::requeue(Duration::from_secs(requeue_duration)))
    }

    // Reconcile with finalize cleanup (the object was deleted)
    pub async fn cleanup(&self, ctx: Arc<Context>) -> Result<Action, kube::Error> {
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
