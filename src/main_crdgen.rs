mod service_alerts;

use kube::CustomResourceExt;
use service_alerts::ServiceAlerter;

fn main() {
    print!("{}", serde_yaml::to_string(&ServiceAlerter::crd()).unwrap())
}