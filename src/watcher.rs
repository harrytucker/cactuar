//! The watcher module is intended to be a basic proof-of-concept that can be
//! run as a Tokio task. It will report any events relating to ServiceAlerter
//! resources.
use crate::ServiceAlerter;

use color_eyre::Result;
use futures::{pin_mut, TryStreamExt};
use kube::{api::ListParams, runtime::watcher, runtime::WatchStreamExt, Api};

pub async fn watch_for_events(api: Api<ServiceAlerter>) -> Result<()> {
    let lp = ListParams::default();
    let watcher = watcher(api, lp).applied_objects();

    pin_mut!(watcher);
    while let Some(event) = watcher.try_next().await? {
        record_event(event);
    }
    Ok(())
}

fn record_event(e: ServiceAlerter) {
    tracing::info!(
        name = e.metadata.name.unwrap(),
        alerts = ?e.spec.alerts,
        "Captured a ServiceAlerter event.",
    )
}
