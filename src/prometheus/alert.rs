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
    pub email_to: String,
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
      source: cloud
      owner: service
    annotations:
      summary: High request latency
      description: Request latency over 9000
      email_to: mail@mail.com"#;

    #[test]
    fn test_serialisation_happy_path() -> color_eyre::Result<()> {
        let rust_repr = Alerts {
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
                        email_to: "mail@mail.com".into(),
                    },
                }],
            }],
        };

        let yaml_repr: Alerts = serde_yaml::from_str(SERIALIZED_PROM_ALERT)?;
        assert_eq!(yaml_repr, rust_repr);

        Ok(())
    }
}
