use kube::CustomResourceExt;

use service_alerts::ServiceAlert;

mod service_alerts;

fn main() {
    print!("{}", serde_yaml::to_string(&ServiceAlert::crd()).unwrap())
}
