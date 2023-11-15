use super::alert::{AlertGroup, AlertRules, Annotations, Labels, PrometheusSeverity};
use crate::crd::{AlertConfig, NetworkAlert, ServiceAlertSpec};

pub fn grpc_alert_rules(
    network_alert: &NetworkAlert,
    alert_configs: &[AlertConfig],
    spec: &ServiceAlertSpec,
) -> AlertGroup {
    let grpc_rules = alert_configs
        .iter()
        .enumerate()
        .map(|(i, conf)| AlertRules {
            alert: format!("{0} {1} {2}", network_alert, conf.operation, conf.value),
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
        rules: grpc_rules,
    }
}

fn grpc_promql(
    network_alert: &NetworkAlert,
    alert_config: &AlertConfig,
    _spec: &ServiceAlertSpec,
) -> String {
    match network_alert {
        NetworkAlert::ErrorPercent => {
            format!("(sum by (destination_workload) (rate(grpc_server_handled_total{{grpc_code=~\"Unknown|ResourceExhausted|Internal|Unavailable|DataLoss|DeadlineExceeded\", destination_workload=\"{0}\"}}[{1}]))/  sum by (destination_workload) (rate(grpc_server_started_total{{destination_workload=\"{0}\"}}[{1}])) * 100 > {2})",
                _spec.deployment_name,
                alert_config.for_,
                alert_config.value
            )
        }
        NetworkAlert::TrafficPerSecond => "this is a placeholder value.".to_string(),
        NetworkAlert::LatencyMillisecondsP50 => {
            format!(
                "histogram_quantile(0.50, istio_request_duration_milliseconds{{destination_workload={0}}}[{1}]) {2} {3}",
                _spec.deployment_name,
                alert_config.for_,
                alert_config.operation,
                alert_config.value
            )
        }
        NetworkAlert::LatencyMillisecondsP90 => {
            format!(
                "histogram_quantile(0.90, istio_request_duration_milliseconds{{destination_workload={0}}}[{1}]) {2} {3}",
                _spec.deployment_name,
                alert_config.for_,
                alert_config.operation,
                alert_config.value
            )
        }
        NetworkAlert::LatencyMillisecondsP95 => {
            format!(
                "histogram_quantile(0.95, istio_request_duration_milliseconds{{destination_workload={0}}}[{1}]) {2} {3}",
                _spec.deployment_name,
                alert_config.for_,
                alert_config.operation,
                alert_config.value
            )
        }
        NetworkAlert::LatencyMillisecondsP99 => {
            format!(
                "histogram_quantile(0.99, istio_request_duration_milliseconds{{destination_workload={0}}}[{1}]) {2} {3}",
                _spec.deployment_name,
                alert_config.for_,
                alert_config.operation,
                alert_config.value
            )
        }
    }
}

fn grpc_summary(network_alert: &NetworkAlert, alert_config: &AlertConfig) -> String {
    match { network_alert } {
        NetworkAlert::ErrorPercent => format!(
            "error rate {0} {1}% for {2}",
            alert_config.operation, alert_config.value, alert_config.for_
        ),
        NetworkAlert::TrafficPerSecond => format!(
            "traffic {0} {1}/sec for {2}",
            alert_config.operation, alert_config.value, alert_config.for_
        ),
        NetworkAlert::LatencyMillisecondsP50 => format!(
            "latency P(50) {0} {1} ms for {2}",
            alert_config.operation, alert_config.value, alert_config.for_
        ),
        NetworkAlert::LatencyMillisecondsP90 => format!(
            "latency P(90) {0} {1} ms for {2}",
            alert_config.operation, alert_config.value, alert_config.for_
        ),
        NetworkAlert::LatencyMillisecondsP95 => format!(
            "latency P(95) {0} {1} ms for {2}",
            alert_config.operation, alert_config.value, alert_config.for_
        ),
        NetworkAlert::LatencyMillisecondsP99 => format!(
            "latency P(99) {0} {1} ms for {2}",
            alert_config.operation, alert_config.value, alert_config.for_
        ),
    }
}

fn grpc_description(network_alert: &NetworkAlert, _alert_config: &AlertConfig) -> String {
    match network_alert {
        NetworkAlert::ErrorPercent => String::from("this is a placeholder description"),
        NetworkAlert::TrafficPerSecond => String::from("this is a placeholder description"),
        NetworkAlert::LatencyMillisecondsP50 => String::from("this is a placeholder description"),
        NetworkAlert::LatencyMillisecondsP90 => String::from("this is a placeholder description"),
        NetworkAlert::LatencyMillisecondsP95 => String::from("this is a placeholder description"),
        NetworkAlert::LatencyMillisecondsP99 => String::from("this is a placeholder description"),
    }
}
