//! # Cactuar!
//!
//! ```text
//!              BG
//!         BB    PB
//!          BP&   P#
//!     BG#   #PB&#PY5555PGB&
//!      &BG#  GJ??J???JYJ??JYG&
//!         &GJ77J?7JYYJJ55Y7?YJG
//!         #!!7!?YY??Y55YY5PY?YY5&
//!         J~!?7~?Y5YJJ5P5Y5P5?J5YG
//!         ?~~7J7!7YPPYJ5PPPBBGY?YJ5&
//!         5^~~7JJ!7J5P5YY5&   GPJJ5YB              &BGPPPB
//!          J~!!7JY?7Y55PYJ5GBG5PG5JYY5&        &#GPYYYYYYYYB
//!           P77!!?5# &PYGPYY5P5Y5GGJJ5YB    &BP55YY55PP5555Y5&
//!            #J!7!?G&&5?5PP55PPP5YPG5JY5P##G5YY555PPGGPP5PP55YB
//!              G7777JYY??Y5B  BPGP55GGYYP55555PPPGGBB#BP555PP5YP&
//!               &Y777?Y5J?J5B  #PPP55PGP5GGGGGGBB#&    &G555PPPP5G
//!                 GJ???J5Y??YP&  BPGGPPGBBBBB#&&        &BP555PPP5P&
//!  #GPPGB&         &PJJ?JYYJ?J5B&#5PPGGGPBB####&          #G555PPGG#
//! 5!?55PPG#          BYJJ?JYY??YY5G5Y5PBBPGB##BG#          &BPPPGB#&
//! 7~7J5GGGGB&         &PYYJJY5J?J5PP5Y5PBBGGB##BGG&          &&&&
//! &J77?5PGGGG&      #G55YYYJ?JYYJJY5GPYYPGBBGGBBGGPB
//!   G?7?J5PGGG#&&#G5YJY5555YJJ?Y5J?J5PG5Y5PBBGGGBBGGG#
//!    &577?YPPPGP5YYY55PPPPPGYJJ?JYY??YPGPYYPGBBGPB#BGGB
//!      B?7?J5P55YY5PPPPGG#& &P????YYJ7J5PP5Y5PB#GGB##BPP&
//!       &57??JY55PPGGB#&      #J???JYJ7?Y5PPY5GB#BGB##GGPB               &&#BB#&
//!         #5JJ5PPGB#&           P?7??JY??Y5PPPPGB##BB#BGGGP#          &BGP555YJYG
//!           &B#&&                #J???JYYJ5GGBGGB####BGGGPGPB      &BG55YYYY5PGBBB
//!                                 57?JJY5PPPBBBBBB#&#PPPPPPGGB#  #G5555YYY55PGBB##
//!                              #G5YYYYJY5PGGGB###&##GGPP5PGGGGPGGP555555PPGGBBGB&
//!                           &BP5YYYY5555PGBBBB#&&  &#BGPPPPGBBGP55555PGGGBBBB#&
//!                         #P5YYY5555PPPGGGGB#&       &BGPPPPPBGPPPPPGGGGBB#&
//!                       GYYYYYY5PPPGGGGGBB&            #BGPP5PGGGGGGGGB#&
//!                      P~?YJY5PGBBGGGB#&                &#GP5PPGGGGB#&
//!                      G~7J?5PGB#GG#&                     &#BBB###&
//!                       G7J?J5PGBBGBB#
//!                        #Y???YPPGBGPGB&
//!                          G???J5PGGGGGGB
//!                           &5?J?Y5PPGPGPG&
//!                             GJJJJY5PGGGGG&
//!                              &5J??Y5PGGG##
//!                                GYJJYPPG##
//!                                 &GPPGB#&
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
