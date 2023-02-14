use std::collections::HashMap;

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
    pub service_selector: ServiceSelector,

    // For reasons to do with `kube` relying on JsonSchema, I cannot use serde's
    // 'with' attribute here, so you need to manually disambiguate both the
    // serialisation and deserialisation.
    //
    // See: https://github.com/GREsau/schemars/issues/89
    #[serde(
        serialize_with = "serde_yaml::with::singleton_map_recursive::serialize",
        deserialize_with = "serde_yaml::with::singleton_map_recursive::deserialize"
    )]
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceSelector {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Alert {
    Replicas(SeverityBoundary),
    RequestErrorPercent(SeverityBoundary),
}

type SeverityBoundary = HashMap<Severity, Boundary>;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Boundary {
    LessThan(i32),
    EqualTo(i32),
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
    // pub reconciled: bool,
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
serviceSelector:
  name: fubar-service
alerts:
- replicas:
    warning:
      lessThan: 3
    critical:
      lessThan: 1
- requestErrorPercent:
    critical:
      equalTo: 100
"#;

    #[test]
    fn test_serialisation_happy_path() -> color_eyre::Result<()> {
        let rust_repr_alerts: Vec<Alert> = vec![
            Alert::Replicas(SeverityBoundary::from([
                (Severity::Warning, Boundary::LessThan(3)),
                (Severity::Critical, Boundary::LessThan(1)),
            ])),
            Alert::RequestErrorPercent(SeverityBoundary::from([(
                Severity::Critical,
                Boundary::EqualTo(100),
            )])),
        ];
        let rust_repr = ServiceAlerterSpec {
            common_labels: HashMap::from([
                ("origin".into(), "cloud".into()),
                ("owner".into(), "bar".into()),
            ]),
            service_selector: ServiceSelector {
                name: "fubar-service".into(),
            },
            alerts: rust_repr_alerts,
        };

        let yaml_repr: ServiceAlerterSpec = serde_yaml::from_str(SERIALIZED_YAML_SPEC)?;

        assert_eq!(yaml_repr, rust_repr);
        Ok(())
    }
}
