use color_eyre::Result;
use pretty_assertions::assert_eq;

use crate::prometheus::alert::*;

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
