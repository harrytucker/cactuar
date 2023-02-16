use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Alerts {
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
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

#[cfg(test)]
mod test {
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
    annotations:
      summary: High request latency"#;

    #[test]
    fn test_serialisation_happy_path() -> color_eyre::Result<()> {
        let rust_repr = Alerts {
            groups: vec![AlertGroup {
                name: "example".into(),
                rules: vec![AlertRules {
                    alert: "HighRequestLatency".into(),
                    expr: r#"job:request_latency_seconds:mean5m{job="myjob"} > 0.5"#.into(),
                    for_: "10m".into(),
                    labels: HashMap::from([("severity".into(), "page".into())]),
                    annotations: HashMap::from([("summary".into(), "High request latency".into())]),
                }],
            }],
        };

        // assert_eq!("", serde_yaml::to_string(&rust_repr)?);
        let yaml_repr: Alerts = serde_yaml::from_str(SERIALIZED_PROM_ALERT)?;
        assert_eq!(yaml_repr, rust_repr);
        Ok(())
    }
}
