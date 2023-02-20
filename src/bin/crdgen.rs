use kube::CustomResourceExt;

use cactuar::service_alerts::ServiceAlert;

fn main() {
    print!("{}", serde_yaml::to_string(&ServiceAlert::crd()).unwrap())
}
