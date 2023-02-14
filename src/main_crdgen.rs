use kube::CustomResourceExt;

use service_alerts::ServiceAlerter;

mod service_alerts;

fn main() {
    print!("{}", serde_yaml::to_string(&ServiceAlerter::crd()).unwrap())
}
