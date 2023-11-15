use crate::crd::{AlertConfig, NetworkAlert, Operation, ServiceAlertSpec};

use super::alert::{AlertGroup, AlertRules, Annotations, Labels, PrometheusSeverity};

pub fn http_rules(spec: &ServiceAlertSpec) -> AlertGroup {
    let mut rules: Vec<AlertRules> = vec![];

    if let Some(rest_alerts) = &spec.alerts.rest {
        rest_alerts.iter().for_each(|(key, val)| match key {
            NetworkAlert::ErrorPercent => rules.append(&mut error_percent_alerts(spec, val)),
            NetworkAlert::TrafficPerSecond => {
                rules.append(&mut traffic_per_second_alerts(spec, val))
            }
            NetworkAlert::LatencyMillisecondsP50 => {
                rules.append(&mut latency_percentile_alerts(spec, 50, val))
            }
            NetworkAlert::LatencyMillisecondsP90 => {
                rules.append(&mut latency_percentile_alerts(spec, 90, val))
            }
            NetworkAlert::LatencyMillisecondsP95 => {
                rules.append(&mut latency_percentile_alerts(spec, 95, val))
            }
            NetworkAlert::LatencyMillisecondsP99 => {
                rules.append(&mut latency_percentile_alerts(spec, 99, val))
            }
        })
    }

    AlertGroup {
        name: String::from("HTTP Alerts"),
        rules,
    }
}

fn error_percent_alerts(spec: &ServiceAlertSpec, alert_configs: &[AlertConfig]) -> Vec<AlertRules> {
    alert_configs
        .iter()
        .enumerate()
        .map(|(i, conf)| AlertRules {
            alert: format!("HTTPErrorPercentRule-{0}-{1}", spec.deployment_name, i),
            expr: format!(r#"error percent {} {}"#, conf.operation, i),
            for_: conf.for_.clone(),
            labels: Labels {
                severity: PrometheusSeverity::from(&conf.with_labels),
                source: spec.common_labels.origin.clone(),
                owner: spec.common_labels.owner.clone(),
            },
            annotations: error_percent_annotations(conf),
        })
        .collect()
}

fn error_percent_annotations(alert_config: &AlertConfig) -> Annotations {
    match alert_config.operation {
        Operation::EqualTo => Annotations {
            summary: String::from("Request errors percentage reached alert boundary"),
            description: format!(
                "Current error percentage is exactly {}%",
                alert_config.value
            ),
        },
        Operation::LessThan => Annotations {
            summary: String::from("Request errors percentage is less than alert boundary"),
            description: format!(
                "Current error percentage is {{{{ $value }}}}%, boundary is {}",
                alert_config.value
            ),
        },
        Operation::MoreThan => Annotations {
            summary: String::from("Request errors percentage is higher than alert boundary"),
            description: format!(
                "Current error percentage is {{{{ $value }}}}%, boundary is {}",
                alert_config.value
            ),
        },
    }
}

fn latency_percentile_alerts(
    spec: &ServiceAlertSpec,
    _percentile: i8,
    alert_configs: &[AlertConfig],
) -> Vec<AlertRules> {
    alert_configs
        .iter()
        .enumerate()
        .map(|(i, conf)| AlertRules {
            alert: format!("HTTPLatencyPercentileRule-{0}-{1}", spec.deployment_name, i),
            expr: format!(
                "histogram_quantile({3}, istio_requests_total{{destination_workload={0}}}[{1}]) {2} {3}",
                spec.deployment_name,
                conf.for_,
                conf.operation,
                conf.value
            ),
            for_: conf.for_.clone(),
            labels: Labels {
                severity: PrometheusSeverity::from(&conf.with_labels),
                source: spec.common_labels.origin.clone(),
                owner: spec.common_labels.owner.clone(),
            },
            annotations: latency_percentile_annotations(conf),
        })
        .collect()
}

fn latency_percentile_annotations(alert_config: &AlertConfig) -> Annotations {
    match alert_config.operation {
        Operation::EqualTo => Annotations {
            summary: String::from("Average request latency reached alert boundary"),
            description: format!(
                "Current request latency is exactly {}ms",
                alert_config.value
            ),
        },
        Operation::LessThan => Annotations {
            summary: String::from("Average request latency is less than alert boundary"),
            description: format!(
                "Current request latency is {{{{ $value }}}}ms, boundary is {}ms",
                alert_config.value
            ),
        },
        Operation::MoreThan => Annotations {
            summary: String::from("Average request latency is higher than alert boundary"),
            description: format!(
                "Current request latency is {{{{ $value }}}}ms, boundary is {}ms",
                alert_config.value
            ),
        },
    }
}

fn traffic_per_second_alerts(
    spec: &ServiceAlertSpec,
    alert_configs: &[AlertConfig],
) -> Vec<AlertRules> {
    alert_configs
        .iter()
        .enumerate()
        .map(|(i, conf)| AlertRules {
            alert: format!("HTTPTrafficPerSecondRule-{0}-{1}", spec.deployment_name, i),
            expr: format!(r#"traffic per second {} {}"#, conf.operation, i),
            for_: conf.for_.clone(),
            labels: Labels {
                severity: PrometheusSeverity::from(&conf.with_labels),
                source: spec.common_labels.origin.clone(),
                owner: spec.common_labels.owner.clone(),
            },
            annotations: traffic_per_second_annotations(conf),
        })
        .collect()
}

fn traffic_per_second_annotations(alert_config: &AlertConfig) -> Annotations {
    match alert_config.operation {
        Operation::EqualTo => Annotations {
            summary: String::from("HTTP requests per second reached alert boundary"),
            description: format!("Requests per second is exactly {}/s", alert_config.value),
        },
        Operation::LessThan => Annotations {
            summary: String::from("HTTP requests per second is less than alert boundary"),
            description: format!(
                "Requests per second is {{{{ $value }}}}/s, boundary is {}/s",
                alert_config.value
            ),
        },
        Operation::MoreThan => Annotations {
            summary: String::from("HTTP requests per second is higher than alert boundary"),
            description: format!(
                "Requests per second is {{{{ $value }}}}/s, boundary is {}/s",
                alert_config.value
            ),
        },
    }
}
