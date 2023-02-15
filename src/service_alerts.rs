use std::collections::HashMap;
use std::hash::Hash;

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
// use serde;

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

    // For reasons to do with `kube` relying on JsonSchema, I cannot use serde's
    // 'with' attribute here, so you need to manually disambiguate both the
    // serialisation and deserialisation.
    //
    // See: https://github.com/GREsau/schemars/issues/89
    #[serde(
        serialize_with = "serde_yaml::with::singleton_map_recursive::serialize",
        deserialize_with = "serde_yaml::with::singleton_map_recursive::deserialize"
    )]
    pub alerts: HashMap<AlertType, Vec<AlertConfig>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceSelector {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum AlertType {
    ReplicaCount,
    ErrorPercent,
    TrafficRate,
    LatencyMilliseconds(Percentile),
}

impl std::str::FromStr for AlertType {

    type Err = ();

    fn from_str(input: &str) -> Result<AlertType, Self::Err> {
        match input {
            "replicaCount"  => Ok(AlertType::ReplicaCount),
            "errorPercent"  => Ok(AlertType::ErrorPercent),
            "trafficRate"  => Ok(AlertType::TrafficRate),
            "LatencyMillisecondsP50" => Ok(AlertType::LatencyMilliseconds(Percentile::P50)),
            "LatencyMillisecondsP90" => Ok(AlertType::LatencyMilliseconds(Percentile::P90)),
            "LatencyMillisecondsP95" => Ok(AlertType::LatencyMilliseconds(Percentile::P95)),
            "LatencyMillisecondsP99" => Ok(AlertType::LatencyMilliseconds(Percentile::P99)),
            _      => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Percentile {
    // I'm lazy, need to generate P1-P99
    P50,
    P90,
    P95,
    P99,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlertConfig {
    operation: Operation,
    value: f32,
    #[serde(rename = "for")]
    for_: String, // want to be able to specify like 3m 4s
    alert_with_labels: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    EqualTo,
    LessThan,
    MoreThan,
}

// impl std::str::FromStr for Operation {
//
//     type Err = ();
//
//     fn from_str(input: &str) -> Result<Operation, Self::Err> {
//         match input {
//             "LessThan"  => Ok(Operation::LessThan),
//             "MoreThan"  => Ok(Operation::MoreThan),
//             "EqualTo"  => Ok(Operation::EqualTo),
//             _      => Err(()),
//         }
//     }
// }

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
  - operation: lessThan
    value: 3
    for: 3m
    alertWithLabels:
      severity: warning
  - operation: equalTo
    value: 0
    for: 0m
    alertWithLabels:
      severity: critical
  latencyMillisecondsP99:
  - operation: moreThan
    value: 20
    for: 5m
    alertWithLabels:
      severity: warning
  - operation: moreThan
    value: 50
    for: 2m
    alertWithLabels:
      severity: critical
  latencyMillisecondsP50:
  - operation: moreThan
    value: 20
    alertWithLabels:
      severity: critical
"#;

    #[test]
    fn test_serialisation_happy_path() -> color_eyre::Result<()> {
        let rust_repr_alerts: HashMap<AlertType, Vec<AlertConfig>> = HashMap::from([
            (AlertType::ReplicaCount, vec![
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
                },
            ]),
            (AlertType::LatencyMilliseconds(Percentile::P99), vec![
                AlertConfig {
                    operation: Operation::MoreThan,
                    value: 20 as f32,
                    for_: String::from("5m"),
                    alert_with_labels: HashMap::from([
                        (String::from("severity"),  String::from("warning")),
                    ])
                },
                AlertConfig {
                    operation: Operation::MoreThan,
                    value: 50 as f32,
                    for_: String::from("2m"),
                    alert_with_labels: HashMap::from([
                        (String::from("severity"),  String::from("critical")),
                    ])
                },
            ]),
            (AlertType::LatencyMilliseconds(Percentile::P50), vec![
                AlertConfig {
                    operation: Operation::LessThan,
                    value: 20 as f32,
                    for_: String::from("0m"),
                    alert_with_labels: HashMap::from([
                        (String::from("severity"),  String::from("critical")),
                    ])
                },
            ])
        ]);

        let rust_repr = ServiceAlerterSpec {
            common_labels: HashMap::from([
                ("origin".into(), "cloud".into()),
                ("owner".into(), "bar".into()),
            ]),
            deployment_name: String::from("best-service-eu"),
            alerts: rust_repr_alerts,
        };

        let yaml_repr: ServiceAlerterSpec = serde_yaml::from_str(SERIALIZED_YAML_SPEC)?;

        assert_eq!(yaml_repr, rust_repr);
        Ok(())
    }
}
