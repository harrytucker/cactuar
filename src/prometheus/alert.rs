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
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    const SERIALIZED_PROM_ALERT: &str = r#"
groups:
  - name: foo-monitoring.rules
    rules:
      - alert : bar-alert
        expr: |
          some_foo
          > 5
        labels:
          severity: warning
          source: cloud
          owner: foo
        annotations:
          SUMMARY: >-
            Bar is returning error or unknown responses to Foo
          DESCRIPTION: >-
            Alerts when bad things happen.
          EMAIL_TO: email@mail.com
"#;

    #[test]
    fn test_serialisation_happy_path() -> color_eyre::Result<()> {
        let rust_repr = Alerts {
            groups: vec![AlertGroup {
                name: "foo-monitoring.rules".into(),
                rules: vec![AlertRules {
                    alert: "bar-alert".into(),
                    expr: "some_foo\n> 5\n".into(),
                    labels: HashMap::from([
                        ("severity".into(), "warning".into()),
                        ("source".into(), "cloud".into()),
                        ("owner".into(), "foo".into()),
                    ]),
                    annotations: HashMap::from([
                        (
                            "SUMMARY".into(),
                            "Bar is returning error or unknown responses to Foo".into(),
                        ),
                        (
                            "DESCRIPTION".into(),
                            "Alerts when bad things happen.".into(),
                        ),
                        ("EMAIL_TO".into(), "email@mail.com".into()),
                    ]),
                }],
            }],
        };

        // assert_eq!("", serde_yaml::to_string(&rust_repr)?);
        let yaml_repr: Alerts = serde_yaml::from_str(SERIALIZED_PROM_ALERT)?;
        assert_eq!(yaml_repr, rust_repr);
        Ok(())
    }
}
