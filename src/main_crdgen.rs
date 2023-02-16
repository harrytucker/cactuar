use kube::CustomResourceExt;

use service_alerts::ServiceAlerts;

mod service_alerts;

fn main() {
    print!("{}", serde_yaml::to_string(&ServiceAlerts::crd()).unwrap())
}
