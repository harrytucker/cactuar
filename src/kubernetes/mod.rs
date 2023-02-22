//! # Kubernetes
//!
//! This module defines the Kubernetes controller and reconciler that Cactuar
//! uses to maintain the correct state in a Kubernetes cluster.

/// Responsible for the Kubernetes controller, this is the top-level structure
/// that is ran by Cactuar in the background.
pub mod controller;

/// Receives events from the controller, responsible for reconciling the current
/// state of Kubernetes with the state defined by our ServiceAlert resources.
///
/// Dispatches events to the correct operation for reconciliation.
pub mod reconciler;

/// Contains the actual reconciliation operations.
pub mod operations;
