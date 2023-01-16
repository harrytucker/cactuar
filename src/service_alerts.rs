use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use kube::CustomResource;

#[derive(CustomResource, Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[kube(
    group = "cactuar.rs",
    version = "v1",
    kind = "ServiceAlerter",
    namespaced
)]
pub struct ServiceAlerterSpec {
    pub common_labels: HashMap<String, String>,
    pub service_selector: ServiceSelector,
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServiceSelector {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Alert {
    ReplicasEqualTo(HashMap<Severity, i32>),
    ReplicasLessThan(HashMap<Severity, i32>),
    LatencyP99MoreThan(HashMap<Severity, i32>),
    LatencyP95MoreThan(HashMap<Severity, i32>),
    LatencyP50MoreThan(HashMap<Severity, i32>),
    ErrorsMoreThan(HashMap<Severity, i32>),
    TrafficMoreThan(HashMap<Severity, i32>),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Warning,
    Critical,
}

#[cfg(test)]
mod test {
    const SERIALISED_EXAMPLE: &str = r#"
        apiVersion: cactuar.rs/v1
        kind: ServiceAlerter
        metadata:
          name: fubar-alerter
        spec:
          commonLabels:
            origin: cloud
            owner: bar
          serviceSelector:
            name: fubar-service
          alerts:
          - replicasEqualTo:
              critical: 0
          - errorsMoreThan:
              warning: 25
"#;

    #[test]
    fn test_serialisation_happy_path() {}
}
