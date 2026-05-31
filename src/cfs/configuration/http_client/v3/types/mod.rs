//! Serde types for the CFS v3 configurations endpoint.
//!
//! These mirror the upstream CSM OpenAPI schema; field names and shapes
//! are dictated by the API.
#![allow(missing_docs)]

pub mod cfs_configuration;
pub mod cfs_configuration_request;
pub mod cfs_configuration_response;

/// Bidirectional `From` impls between the v3 configuration [`types`] and
/// the dispatcher's mirror types. Gated behind the `manta-dispatcher`
/// Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;
