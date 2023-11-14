use std::collections::{BTreeMap, HashMap};

use color_eyre::{eyre::eyre, Result};
use serde::{Deserialize, Serialize};

use crate::{
    crd::{ReplicaAlert, ServiceAlertSpec},
    prometheus::{http_alerts::http_rules, grpc_alerts::grpc_alert_rules, replica_alerts::replica_count_rules},
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

    fn try_from(spec: ServiceAlertSpec) -> Result<Self, Self::Error> {
        use crate::prometheus::alert::*;

        let mut alerts = PromAlerts { groups: Vec::new() };

        if let Some(replica_alerts) = &spec.alerts.replica {
            replica_alerts.iter().for_each(|(key, val)| match key {
                ReplicaAlert::Count => alerts.groups.push(replica_count_rules(val, &spec)),
            });
        }

        if spec.alerts.rest.is_some() {
            alerts.groups.push(http_rules(&spec))
        }

        if let Some(grpc_alerts) = &spec.alerts.grpc {
            grpc_alerts
                .iter()
                .for_each(|(key, val)| alerts.groups.push(grpc_alert_rules(key, val, &spec)));
        }

        Ok(alerts)
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
