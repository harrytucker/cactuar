use std::collections::HashMap;
use std::hash::Hash;

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Identifier that is recorded by the Kubernetes API for the purpose of
/// identifying the application responsible for the given Kubernetes resource.
const MANAGER_STRING: &str = "cactuar";

pub const API_GROUP: &str = "cactuar.rs";
pub const API_VERSION: &str = "v1alpha1";
pub const KIND: &str = "ServiceAlerter";
pub const FINALIZER_NAME: &str = "servicealerter.cactuar.rs";

#[derive(CustomResource, Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[kube(
    group = "cactuar.rs",
    version = "v1",
    kind = "ServiceAlerter",
    namespaced
)]
pub struct ServiceAlerterSpec {
    pub common_labels: HashMap<String, String>,
    pub deployment_name: String,
    pub alerts: Alerts,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Alerts {
    replica_count: Option<Vec<AlertConfig>>,
    error_percent: Option<Vec<AlertConfig>>,
    traffic_per_second: Option<Vec<AlertConfig>>,
    latency_histogram_milliseconds: Option<Vec<AlertConfigHistogram>>,

}

// TODO: Add validation for Percentile to be between 0 and 1 non-inclusive
type Percentile = f32;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlertConfig {
    operation: Operation,
    value: f32,
    #[serde(rename = "for")]
    for_: String, // want to be able to specify like 3m 4s
    alert_with_labels: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlertConfigHistogram {
    operation: Operation,
    percentile: Percentile,
    value: f32,
    #[serde(rename = "for")]
    for_: String, // want to be able to specify like 3m 4s
    alert_with_labels: HashMap<String, String>,
}

// Kubernetes enums start with an upper case letter
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    #[serde(rename = "EqualTo")]
    EqualTo,
    #[serde(rename = "LessThan")]
    LessThan,
    #[serde(rename = "MoreThan")]
    MoreThan,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Warning,
    Critical,
}

/// The status object of `StatusAlerter`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServiceAlerterStatus {
    pub last_reconciled_at: Option<String>,
    pub reconciliation_expires_at: Option<String>,
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    const SERIALIZED_YAML_SPEC: &str = r#"
commonLabels:
  origin: cloud
  owner: bar
deploymentName: best-service-eu
alerts:
  replicaCount:
  - operation: LessThan
    value: 3
    for: 3m
    alertWithLabels:
      severity: warning
  - operation: EqualTo
    value: 0
    for: 0m
    alertWithLabels:
      severity: critical
  latencyHistogramMilliseconds:
  - operation: MoreThan
    percentile: 0.99
    value: 20
    for: 5m
    alertWithLabels:
      severity: warning
  - operation: MoreThan
    percentile: 0.99
    value: 50
    for: 2m
    alertWithLabels:
      severity: critical
  - operation: MoreThan
    percentile: 0.5
    value: 20
    for: 0m
    alertWithLabels:
      severity: critical
"#;

    #[test]
    fn test_serialisation_happy_path() -> color_eyre::Result<()> {
        let rust_repr = ServiceAlerterSpec {
            common_labels: HashMap::from([
                ("origin".into(), "cloud".into()),
                ("owner".into(), "bar".into()),
            ]),
            deployment_name: String::from("best-service-eu"),
            alerts: Alerts {
                replica_count: Some(vec![
                    AlertConfig {
                        operation: Operation::LessThan,
                        value: 3 as f32,
                        for_: String::from("3m"),
                        alert_with_labels: HashMap::from([
                            (String::from("severity"), String::from("warning")),
                        ])
                    },
                    AlertConfig {
                        operation: Operation::EqualTo,
                        value: 0 as f32,
                        for_: String::from("0m"),
                        alert_with_labels: HashMap::from([
                            (String::from("severity"),  String::from("critical")),
                        ])
                    }
                ]),

                error_percent: None,
                traffic_per_second: None,
                latency_histogram_milliseconds: Some(vec![
                    AlertConfigHistogram {
                        operation: Operation::MoreThan,
                        percentile: 0.99,
                        value: 20 as f32,
                        for_: String::from("5m"),
                        alert_with_labels: HashMap::from([
                            (String::from("severity"),  String::from("warning")),
                        ])
                    },
                    AlertConfigHistogram {
                        operation: Operation::MoreThan,
                        percentile: 0.99,
                        value: 50 as f32,
                        for_: String::from("2m"),
                        alert_with_labels: HashMap::from([
                            (String::from("severity"),  String::from("critical")),
                        ])
                    },
                    AlertConfigHistogram {
                        operation: Operation::MoreThan,
                        percentile: 0.5,
                        value: 20 as f32,
                        for_: String::from("0m"),
                        alert_with_labels: HashMap::from([
                            (String::from("severity"),  String::from("critical")),
                        ])
                    }
                ]),
            },
        };

        let yaml_repr: ServiceAlerterSpec = serde_yaml::from_str(SERIALIZED_YAML_SPEC)?;

        assert_eq!(yaml_repr, rust_repr);
        Ok(())
    }
}
