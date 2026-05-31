//! Redfish endpoints registered under
//! `/smd/hsm/v2/Inventory/RedfishEndpoints`.

pub mod http_client;
pub mod types;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// HSM Redfish endpoint mirror types. Gated behind the `manta-dispatcher`
/// Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;
