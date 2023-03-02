//! # CRD Generator
//!
//! This binary simply generates and prints the Kubernetes Custom Resource
//! Definition for Cactuar.
//!
//! Run `cargo make helm-crd` to run this binary and export the CRD
//! specification to the Helm chart directory.

use kube::CustomResourceExt;

use cactuar::crd::ServiceAlert;

fn main() {
    print!("{}", serde_yaml::to_string(&ServiceAlert::crd()).unwrap())
}
