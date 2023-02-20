use std::collections::HashMap;
use std::hash::Hash;

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const API_GROUP: &str = "cactuar.rs";
pub const API_VERSION: &str = "v1alpha1";
pub const KIND: &str = "ServiceAlert";
pub const FINALIZER_NAME: &str = "servicealert.cactuar.rs";

#[derive(CustomResource, Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[kube(
    group = "cactuar.rs",
    version = "v1",
    kind = "ServiceAlert",
    namespaced
)]
pub struct ServiceAlertSpec {
    pub common_labels: HashMap<String, String>,
    pub deployment_name: String,
    pub alerts: HashMap<Alerts, Vec<AlertConfig>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Alerts {
    ReplicaCount,
    ErrorPercent,
    TrafficPerSecond,
    LatencyMillisecondsP50,
    LatencyMillisecondsP90,
    LatencyMillisecondsP95,
    LatencyMillisecondsP99,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlertConfig {
    pub operation: Operation,
    pub value: f32,
    #[serde(rename = "for")]
    pub for_: String, // want to be able to specify like 3m 4s
    pub alert_with_labels: HashMap<String, String>,
}

// Kubernetes enums start with an upper case letter
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Eq, PartialEq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum Operation {
    EqualTo,
    LessThan,
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
pub struct ServiceAlertStatus {
    pub last_reconciled_at: Option<String>,
    pub reconciliation_expires_at: Option<String>,
}

// impl TryFrom<ServiceAlertSpec> for alert::Alerts {
//     type Error = String;

//     fn try_from(value: ServiceAlertSpec) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

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
  latencyMillisecondsP99:
  - operation: MoreThan
    value: 20
    for: 5m
    alertWithLabels:
      severity: warning
  - operation: MoreThan
    value: 50
    for: 2m
    alertWithLabels:
      severity: critical
  latencyMillisecondsP50:
  - operation: MoreThan
    value: 20
    for: 0m
    alertWithLabels:
      severity: critical
"#;

    #[test]
    fn test_serialisation_happy_path() -> color_eyre::Result<()> {
        let rust_repr = ServiceAlertSpec {
            common_labels: HashMap::from([
                ("origin".into(), "cloud".into()),
                ("owner".into(), "bar".into()),
            ]),
            deployment_name: String::from("best-service-eu"),
            alerts: HashMap::from([
                (
                    Alerts::ReplicaCount,
                    vec![
                        AlertConfig {
                            operation: Operation::LessThan,
                            value: 3 as f32,
                            for_: String::from("3m"),
                            alert_with_labels: HashMap::from([(
                                String::from("severity"),
                                String::from("warning"),
                            )]),
                        },
                        AlertConfig {
                            operation: Operation::EqualTo,
                            value: 0 as f32,
                            for_: String::from("0m"),
                            alert_with_labels: HashMap::from([(
                                String::from("severity"),
                                String::from("critical"),
                            )]),
                        },
                    ],
                ),
                (
                    Alerts::LatencyMillisecondsP99,
                    vec![
                        AlertConfig {
                            operation: Operation::MoreThan,
                            value: 20 as f32,
                            for_: String::from("5m"),
                            alert_with_labels: HashMap::from([(
                                String::from("severity"),
                                String::from("warning"),
                            )]),
                        },
                        AlertConfig {
                            operation: Operation::MoreThan,
                            value: 50 as f32,
                            for_: String::from("2m"),
                            alert_with_labels: HashMap::from([(
                                String::from("severity"),
                                String::from("critical"),
                            )]),
                        },
                    ],
                ),
                (
                    Alerts::LatencyMillisecondsP50,
                    vec![AlertConfig {
                        operation: Operation::MoreThan,
                        value: 20 as f32,
                        for_: String::from("0m"),
                        alert_with_labels: HashMap::from([(
                            String::from("severity"),
                            String::from("critical"),
                        )]),
                    }],
                ),
            ]),
        };

        let yaml_repr: ServiceAlertSpec = serde_yaml::from_str(SERIALIZED_YAML_SPEC)?;
        assert_eq!(yaml_repr, rust_repr);
        Ok(())
    }
}
