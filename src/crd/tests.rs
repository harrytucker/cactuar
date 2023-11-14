use std::collections::HashMap;

use pretty_assertions::assert_eq;

use crate::crd::service_alert::*;

const SERIALIZED_YAML_SPEC: &str = r#"
commonLabels:
  origin: cloud
  owner: foo
deploymentName: best-service-eu
alerts:
  REST:
    latencyMillisecondsP99:
      - operation: MoreThan
        value: 20
        for: 5m
        withLabels:
          severity: warning
      - operation: MoreThan
        value: 50
        for: 2m
        withLabels:
          severity: critical
  gRPC:
    errorPercent:
      - operation: MoreThan
        value: 10 # %
        for: 3m
        withLabels:
          severity: warning
  replica:
    count:
      - operation: LessThan
        value: 3
        for: 5m
        withLabels:
          severity: warning
      - operation: EqualTo
        value: 0
        for: 1m
        withLabels:
          severity: critical
"#;

#[test]
fn test_serialisation_happy_path() -> color_eyre::Result<()> {
    let rust_repr = ServiceAlertSpec {
        common_labels: CommonLabels {
            owner: String::from("foo"),
            origin: String::from("cloud"),
            extra: HashMap::new(),
        },
        deployment_name: String::from("best-service-eu"),
        alerts: Alerts {
            grpc: Some(HashMap::from([(
                NetworkAlert::ErrorPercent,
                vec![AlertConfig {
                    operation: Operation::MoreThan,
                    value: 10_f32,
                    for_: String::from("3m"),
                    with_labels: HashMap::from([(
                        String::from("severity"),
                        String::from("warning"),
                    )]),
                }],
            )])),
            rest: Some(HashMap::from([(
                NetworkAlert::LatencyMillisecondsP99,
                vec![
                    AlertConfig {
                        operation: Operation::MoreThan,
                        value: 20_f32,
                        for_: String::from("5m"),
                        with_labels: HashMap::from([(
                            String::from("severity"),
                            String::from("warning"),
                        )]),
                    },
                    AlertConfig {
                        operation: Operation::MoreThan,
                        value: 50_f32,
                        for_: String::from("2m"),
                        with_labels: HashMap::from([(
                            String::from("severity"),
                            String::from("critical"),
                        )]),
                    },
                ],
            )])),
            replica: Some(HashMap::from([(
                ReplicaAlert::Count,
                vec![
                    AlertConfig {
                        operation: Operation::LessThan,
                        value: 3_f32,
                        for_: String::from("5m"),
                        with_labels: HashMap::from([(
                            String::from("severity"),
                            String::from("warning"),
                        )]),
                    },
                    AlertConfig {
                        operation: Operation::EqualTo,
                        value: 0 as f32,
                        for_: String::from("1m"),
                        with_labels: HashMap::from([(
                            String::from("severity"),
                            String::from("critical"),
                        )]),
                    },
                ],
            )])),
        },
    };

    let yaml_repr: ServiceAlertSpec = serde_yaml::from_str(SERIALIZED_YAML_SPEC)?;
    assert_eq!(yaml_repr, rust_repr);
    Ok(())
}
