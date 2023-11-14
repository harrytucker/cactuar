use super::alert::{AlertGroup, AlertRules, Annotations, Labels, PrometheusSeverity};
use crate::crd::{AlertConfig, NetworkAlert, Operation, ServiceAlertSpec};

pub fn grpc_alert_rules(
    network_alert: &NetworkAlert,
    alert_configs: &[AlertConfig],
    spec: &ServiceAlertSpec,
) -> AlertGroup {
    let replica_rules = alert_configs
        .iter()
        .enumerate()
        .map(|(i, conf)| AlertRules {
            alert: String::from(format!(
                "{0} {1} {2}",
                network_alert, alert_configs[i].operation, alert_configs[i].value
            )),
            expr: grpc_promql(network_alert, conf, spec),
            for_: conf.for_.clone(),
            labels: Labels {
                severity: PrometheusSeverity::from(&conf.with_labels),
                source: spec.common_labels.origin.clone(),
                owner: spec.common_labels.owner.clone(),
            },
            annotations: Annotations {
                summary: grpc_summary(network_alert, &alert_configs[i]),
                description: grpc_description(network_alert, &alert_configs[i]),
            },
        })
        .collect();

    AlertGroup {
        name: String::from("gRPC Alerts"),
        rules: replica_rules,
    }
}

fn grpc_promql(
    network_alert: &NetworkAlert,
    alert_config: &AlertConfig,
    spec: &ServiceAlertSpec,
) -> String {
    let operation = &alert_config.operation;
    format!(r#"foobar {operation} baz"#)
}

fn grpc_summary(network_alert: &NetworkAlert, alert_config: &AlertConfig) -> String {
    match { network_alert } {
        NetworkAlert::ErrorPercent => String::from(format!(
            "error rate {0} {1}% for {2}",
            alert_config.operation, alert_config.value, alert_config.for_
        )),
        NetworkAlert::TrafficPerSecond => String::from(format!(
            "traffic {0} {1}/sec for {2}",
            alert_config.operation, alert_config.value, alert_config.for_
        )),
        NetworkAlert::LatencyMillisecondsP50 => String::from(format!(
            "latency P(50) {0} {1} ms for {2}",
            alert_config.operation, alert_config.value, alert_config.for_
        )),
        NetworkAlert::LatencyMillisecondsP90 => String::from(format!(
            "latency P(90) {0} {1} ms for {2}",
            alert_config.operation, alert_config.value, alert_config.for_
        )),
        NetworkAlert::LatencyMillisecondsP95 => String::from(format!(
            "latency P(95) {0} {1} ms for {2}",
            alert_config.operation, alert_config.value, alert_config.for_
        )),
        NetworkAlert::LatencyMillisecondsP99 => String::from(format!(
            "latency P(99) {0} {1} ms for {2}",
            alert_config.operation, alert_config.value, alert_config.for_
        )),
    }
}

fn grpc_description(network_alert: &NetworkAlert, alert_config: &AlertConfig) -> String {
    match network_alert {
        NetworkAlert::ErrorPercent => String::from("this is a placeholder description"),
        NetworkAlert::TrafficPerSecond => String::from("this is a placeholder description"),
        NetworkAlert::LatencyMillisecondsP50 => String::from("this is a placeholder description"),
        NetworkAlert::LatencyMillisecondsP90 => String::from("this is a placeholder description"),
        NetworkAlert::LatencyMillisecondsP95 => String::from("this is a placeholder description"),
        NetworkAlert::LatencyMillisecondsP99 => String::from("this is a placeholder description"),
    }
}

// fn grpc_promql(network_alert: NetworkAlert, conf: AlertConfig) -> String {
//
// }

// fn network_alert(network_alert: NetworkAlert) -> str {
//     match network_alert {
//         NetworkAlert::ErrorPercent => {}
//         NetworkAlert::TrafficPerSecond => {}
//         NetworkAlert::LatencyMillisecondsP50 => {}
//         NetworkAlert::LatencyMillisecondsP90 => {}
//         NetworkAlert::LatencyMillisecondsP95 => {}
//         NetworkAlert::LatencyMillisecondsP99 => {}
//     }
// }
