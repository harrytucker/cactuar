use crate::crd::{AlertConfig, NetworkAlert, Operation, ServiceAlertSpec};

use super::alert::{AlertGroup, AlertRules, Annotations, Labels, PrometheusSeverity};

pub fn http_alert_rules(alert_configs: &[AlertConfig], spec: &ServiceAlertSpec) -> AlertGroup {
    let replica_rules = alert_configs
        .iter()
        .enumerate()
        .map(|(i, conf)| AlertRules {
            alert: format!("HTTPRule-{0}-{1}", spec.deployment_name, i),
            expr: http_error_percent_promql(conf, spec),
            for_: conf.for_.clone(),
            labels: Labels {
                severity: PrometheusSeverity::from(&conf.with_labels),
                source: spec.common_labels.origin.clone(),
                owner: spec.common_labels.owner.clone(),
            },
            annotations: http_annotations(conf),
        })
        .collect();

    AlertGroup {
        name: String::from("HTTP Alerts"),
        rules: replica_rules,
    }
}

fn http_latency_rules(spec: &ServiceAlertSpec) -> AlertGroup {
    let mut rules: Vec<AlertRules> = vec![];

    if let Some(rest_alerts) = &spec.alerts.rest {
        rest_alerts.iter().for_each(|(key, val)| match key {
            NetworkAlert::ErrorPercent => {
                rules.append(&mut error_percent_alerts(&spec.deployment_name, &val))
            }
            NetworkAlert::TrafficPerSecond => todo!(),
            NetworkAlert::LatencyMillisecondsP50 => todo!(),
            NetworkAlert::LatencyMillisecondsP90 => todo!(),
            NetworkAlert::LatencyMillisecondsP95 => todo!(),
            NetworkAlert::LatencyMillisecondsP99 => todo!(),
        })
    }

    AlertGroup {
        name: String::from("HTTP Alerts"),
        rules,
    }
}

fn error_percent_alerts(deployment_name: &str, alert_configs: &[AlertConfig]) -> Vec<AlertRules> {
    alert_configs
        .iter()
        .enumerate()
        .map(|(i, conf)| AlertRules {
            alert: format!("HTTPErrorPercentRule-{0}-{1}", deployment_name, i),
            expr: format!(r#"error percent {} {}"#, conf.operation, i),
            for_: conf.for_.clone(),
            labels: Labels {
                severity: todo!(),
                source: todo!(),
                owner: todo!(),
            },
            annotations: todo!(),
        })
        .collect()
}

fn http_error_percent_promql(alert_config: &AlertConfig, spec: &ServiceAlertSpec) -> String {
    let operation = &alert_config.operation;
    format!(r#"foobar {operation} baz"#)
}

fn http_annotations(alert_config: &AlertConfig) -> Annotations {
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
