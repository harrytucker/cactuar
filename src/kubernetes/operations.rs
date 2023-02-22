use std::{collections::BTreeMap, sync::Arc};

use chrono::Utc;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{
    api::{Api, Patch, PatchParams, ResourceExt},
    runtime::{
        controller::Action,
        events::{Event, EventType, Recorder},
    },
    Resource,
};
use thiserror::Error;
use tokio::time::Duration;

use crate::prometheus::alert::PromAlerts;
use crate::service_alerts::{ServiceAlert, ServiceAlertStatus, API_GROUP, FINALIZER_NAME};

use super::reconciler::Context;

#[derive(Debug, Error)]
pub enum OperationError {
    #[error("Failed to create ConfigMap: {0}")]
    ConfigMapCreationFailed(#[source] kube::Error),
    #[error("MissingObjectKey: {0}")]
    MissingObjectKey(&'static str),
    #[error(transparent)]
    Kube(#[from] kube::Error),
    #[error(transparent)]
    Other(#[from] color_eyre::Report),
}

const SUCCESSFUL_REQUEUE_DURATION: u64 = 5 * 60;

impl ServiceAlert {
    // Reconcile (for non-finalizer related changes)
    #[tracing::instrument(skip_all, fields(self.metadata.name, self.metadata.namespace))]
    pub async fn update(&self, ctx: Arc<Context>) -> Result<Action, OperationError> {
        let name = self.name_any();
        let namespace = self
            .namespace()
            .ok_or_else(|| OperationError::MissingObjectKey("namespace"))?;
        let owner_references = self
            .controller_owner_ref(&())
            .ok_or_else(|| OperationError::MissingObjectKey("owner_references"))?;

        let service_alert_api: Api<ServiceAlert> = Api::namespaced(ctx.client.clone(), &namespace);
        let config_map_api: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &namespace);

        let prom_alert = PromAlerts::try_from(self.spec.clone())?;

        tracing::debug!("building configmap");
        let cm = ConfigMap {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                namespace: Some(namespace),
                // This label is what allows prometheus to pick up the configMap
                labels: Some(BTreeMap::from([("rules".into(), "prom-rule".into())])),
                owner_references: Some(vec![owner_references]),
                ..ObjectMeta::default()
            },
            data: Some(BTreeMap::try_from(prom_alert)?),
            ..Default::default()
        };

        tracing::debug!("patching configmap");
        config_map_api
            .patch(
                &name,
                &PatchParams::apply(FINALIZER_NAME),
                &Patch::Apply(&cm),
            )
            .await?;

        tracing::debug!("getting ServiceAlert patch status");
        let ps = PatchParams::apply(API_GROUP).force();
        let _o = service_alert_api
            .patch_status(&name, &ps, &self.generate_status_patch())
            .await?;

        // If no events were received, check back every 5 minutes
        Ok(Action::requeue(Duration::from_secs(
            SUCCESSFUL_REQUEUE_DURATION,
        )))
    }

    // Reconcile with finalize cleanup (the object was deleted)
    #[tracing::instrument(skip_all, fields(self.metadata.name, self.metadata.namespace))]
    pub async fn cleanup(&self, ctx: Arc<Context>) -> Result<Action, OperationError> {
        tracing::debug!("deleting resource");

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

    #[tracing::instrument]
    pub fn generate_status_patch(&self) -> Patch<ServiceAlertStatus> {
        tracing::debug!("building ServiceAlert status");
        Patch::Apply(ServiceAlertStatus {
            last_reconciled_at: Some(Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string()),
            reconciliation_expires_at: Some(
                (Utc::now() + chrono::Duration::seconds(SUCCESSFUL_REQUEUE_DURATION as i64))
                    .format("%Y-%m-%dT%H:%M:%S")
                    .to_string(),
            ),
        })
    }
}
