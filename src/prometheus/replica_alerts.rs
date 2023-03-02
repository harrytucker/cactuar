use crate::service_alerts::{AlertConfig, Operation, ServiceAlertSpec};

use super::alert::{
    AlertGroup, AlertRules, Annotations, Labels, PrometheusSeverity, PLACEHOLDER_VALUE,
};

pub fn replica_count_rules(alert_configs: &[AlertConfig], spec: &ServiceAlertSpec) -> AlertGroup {
    // Prometheus Alert Rules in a single file must be uniquely named, but we
    // can't generate a *random* unique identifier, since that would break the
    // idempotency of our reconciliation, booting us into an infinite loop.
    let replica_rules = alert_configs
        .iter()
        .enumerate()
        .map(|(i, conf)| AlertRules {
            alert: format!("ReplicaCountRules-{0}-{1}", spec.deployment_name, i),
            expr: replicas_promql(conf, spec),
            for_: conf.for_.clone(),
            labels: Labels {
                severity: PrometheusSeverity::from(&conf.with_labels),
                source: spec.common_labels.origin.clone(),
                owner: spec.common_labels.owner.clone(),
            },
            annotations: Annotations {
                summary: PLACEHOLDER_VALUE.into(),
                description: PLACEHOLDER_VALUE.into(),
            },
        })
        .collect();

    AlertGroup {
        name: PLACEHOLDER_VALUE.into(),
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
// sum by (app_kubernetes_io_name) (up{app_kubernetes_io_name="software-catalog-grpc"}) == 0
// struct PromQL {
//     aggr: String,
// }

fn replicas_promql(alert_config: &AlertConfig, spec: &ServiceAlertSpec) -> String {
    match alert_config.operation {
        Operation::EqualTo => {
            format!(
                r#"sum by (app_kubernetes_io_name) (up{{app_kubernetes_io_name="{0}"}}) == {1}"#,
                spec.deployment_name, alert_config.value,
            )
        }
        Operation::LessThan => format!(
            r#"sum by (app_kubernetes_io_name) (up{{app_kubernetes_io_name="{0}"}}) < {1}"#,
            spec.deployment_name, alert_config.value,
        ),
        Operation::MoreThan => format!(
            r#"sum by (app_kubernetes_io_name) (up{{app_kubernetes_io_name="{0}"}}) > {1}"#,
            spec.deployment_name, alert_config.value,
        ),
    }
}
