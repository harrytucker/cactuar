use crate::crd::{AlertConfig, Operation, ServiceAlertSpec};

use super::alert::{AlertGroup, AlertRules, Annotations, Labels, PrometheusSeverity};

/// Generates an [`AlertGroup`] for a list of defined replica alerts. Caller is
/// responsible for only passing in a slice of alerts that are actually replica
/// alerts!
pub fn replica_count_rules(alert_configs: &[AlertConfig], spec: &ServiceAlertSpec) -> AlertGroup {
    // Prometheus Alert Rules in a single file must be uniquely named, but we
    // can't generate a *random* unique identifier, since that would break the
    // idempotency of our reconciliation, booting us into an infinite loop.
    let replica_rules = alert_configs
        .iter()
        .enumerate()
        .map(|(i, conf)| AlertRules {
            alert: format!("ReplicaRule-{0}-{1}", spec.deployment_name, i),
            expr: replicas_promql(conf, spec),
            for_: conf.for_.clone(),
            labels: Labels {
                severity: PrometheusSeverity::from(&conf.with_labels),
                source: spec.common_labels.origin.clone(),
                owner: spec.common_labels.owner.clone(),
            },
            annotations: replicas_annotations(conf),
        })
        .collect();

    AlertGroup {
        name: String::from("Replica Alerts"),
        rules: replica_rules,
    }
}

// Since the metrics are different for different protocols, we must map each Alerts enum
// to a different expression string in prometheus land.
// e.g.
// REST + ErrorPercent uses the istio_requests_total         istio standard metric
// gRPC + ErrorPercent uses the istio_request_messages_total istio standard metric
//
// Example query (all replicas down):
// sum by (app_kubernetes_io_name) (up{app_kubernetes_io_name="best-service-eu-grpc"}) == 0
// struct PromQL {
//     aggr: String,
// }

/// Returns a [`String`] containing the PromQL expression for a given
/// [`AlertConfig`] that alerts based on the number of pod replicas deployed.
///
/// Note that, as [AlertConfigs](AlertConfig) are agnostic to the type of alert,
/// it is the caller's responsibility to *not* call this function on other alert
/// types, like HTTP or gRPC alerts.
fn replicas_promql(alert_config: &AlertConfig, spec: &ServiceAlertSpec) -> String {
    let operation = &alert_config.operation;
    format!(
        r#"sum by (app_kubernetes_io_name) (up{{app_kubernetes_io_name="{0}"}}) {operation} {1}"#,
        spec.deployment_name, alert_config.value,
    )
}

/// Returns the [`Annotations`] struct for a given [`AlertConfig`].
fn replicas_annotations(alert_config: &AlertConfig) -> Annotations {
    // Alert annotations and labels for Prometheus can be templated, using two
    // pairs of braces.
    //
    // Rust uses a single pair of braces for `format!()` macro templating, so
    // you need to use an extra pair of braces for every literal brace you want
    // in the string. This is why you see quadruple brace pairs in descriptions!
    match alert_config.operation {
        Operation::EqualTo => Annotations {
            summary: String::from("Replicas reached alert boundary"),
            description: format!("{0} replicas currently up", alert_config.value),
        },
        Operation::LessThan => Annotations {
            summary: String::from("Replicas less than alert boundary"),
            description: format!(
                "{{{{ $value }}}} replicas currently up, expected at least {0}",
                alert_config.value
            ),
        },
        Operation::MoreThan => Annotations {
            summary: String::from("Replicas more than alert boundary"),
            description: format!(
                "{{{{ $value }}}} replicas currently up, expected less than {0}",
                alert_config.value
            ),
        },
    }
}
