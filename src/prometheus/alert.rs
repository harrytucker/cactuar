use std::collections::{BTreeMap, HashMap};

use color_eyre::{eyre::eyre, Result};
use serde::{Deserialize, Serialize};

use crate::{
    prometheus::replica_alerts::produce_replica_alerts,
    service_alerts::{Alerts, ServiceAlertSpec},
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PromAlerts {
    pub groups: Vec<AlertGroup>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AlertGroup {
    pub name: String,
    pub rules: Vec<AlertRules>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AlertRules {
    pub alert: String,
    pub expr: String,
    #[serde(rename = "for")]
    pub for_: String,
    pub labels: Labels,
    pub annotations: Annotations,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PrometheusSeverity {
    Warning,
    Critical,
    Page,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Labels {
    pub severity: PrometheusSeverity,
    pub source: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Annotations {
    pub summary: String,
    pub description: String,
}

impl TryFrom<PromAlerts> for BTreeMap<String, String> {
    type Error = color_eyre::Report;

    fn try_from(value: PromAlerts) -> Result<Self, Self::Error> {
        // owner should be a unique identifier, so at least for now we can use
        // it as the key for our `BTreeMap`
        let identifier = match value.groups.first() {
            Some(group) => match group.rules.first() {
                Some(rule) => rule.labels.owner.clone(),
                None => return Err(eyre!("No rules defined in alert group.")),
            },
            None => return Err(eyre!("No alert rule groups defined.")),
        };

        let yaml_string = serde_yaml::to_string(&value)?;

        Ok(BTreeMap::from([(identifier, yaml_string)]))
    }
}

/// FIXME: This should be replaced with a generated/converted when possible.
/// Once this is marked as DEAD_CODE then we are good to go!
pub const PLACEHOLDER_VALUE: &str = "PLACEHOLDER";

impl TryFrom<ServiceAlertSpec> for PromAlerts {
    type Error = color_eyre::Report;

    fn try_from(value: ServiceAlertSpec) -> Result<Self, Self::Error> {
        todo!()
        // use crate::prometheus::alert::*;

        // let mut alerts = PromAlerts {
        //     groups: Vec::with_capacity(value.alerts.len()),
        // };

        // if let Some(replica_alerts) = value.alerts.misc.unwrap().get(&crate::service_alerts::MiscAlerts::AllReplicasDown::ReplicaCount) {
        //     alerts
        //         .groups
        //         .push(produce_replica_alerts(replica_alerts, &value));
        // }

        // Ok(alerts)
    }
}

/// CRD Severities are currently part of a HashMap, so we need to grab them from
/// that structure. Since we don't need to modify or consume the HashMap, I
/// borrow it here.
impl From<&HashMap<String, String>> for PrometheusSeverity {
    fn from(value: &HashMap<String, String>) -> Self {
        match value.get("severity").unwrap().as_str() {
            "warning" => PrometheusSeverity::Warning,
            "critical" => PrometheusSeverity::Critical,
            "page" => PrometheusSeverity::Page,
            _ => PrometheusSeverity::Warning,
        }
    }
}

#[cfg(test)]
mod test {
    use color_eyre::Result;
    use pretty_assertions::assert_eq;

    use super::*;

    const SERIALIZED_PROM_ALERT: &str = r#"
groups:
- name: example
  rules:
  - alert: HighRequestLatency
    expr: job:request_latency_seconds:mean5m{job="myjob"} > 0.5
    for: 10m
    labels:
      severity: page
      source: cloud
      owner: service
    annotations:
      summary: High request latency
      description: Request latency over 9000"#;

    #[test]
    fn test_serialisation_happy_path() -> Result<()> {
        let rust_repr = PromAlerts {
            groups: vec![AlertGroup {
                name: "example".into(),
                rules: vec![AlertRules {
                    alert: "HighRequestLatency".into(),
                    expr: r#"job:request_latency_seconds:mean5m{job="myjob"} > 0.5"#.into(),
                    for_: "10m".into(),
                    labels: Labels {
                        severity: PrometheusSeverity::Page,
                        source: "cloud".into(),
                        owner: "service".into(),
                    },
                    annotations: Annotations {
                        summary: "High request latency".into(),
                        description: "Request latency over 9000".into(),
                    },
                }],
            }],
        };

        let yaml_repr: PromAlerts = serde_yaml::from_str(SERIALIZED_PROM_ALERT)?;
        assert_eq!(yaml_repr, rust_repr);

        Ok(())
    }
}
