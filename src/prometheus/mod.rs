//! # Prometheus
//!
//! Cactuar needs to be able to convert its
//! [`crate::service_alerts::ServiceAlert`] type to a standard Prometheus
//! representation. This module provides an opinionated structure for a
//! Prometheus alert that Cactuar can produce as a Kubernetes `ConfigMap`.

pub mod alert;
pub mod replica_alerts;
