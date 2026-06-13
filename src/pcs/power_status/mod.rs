//! PCS power status — query the current power state of components.
//! Wraps `/power-control/v1/power-status`.

/// Request / response types for the PCS power-status endpoints.
pub mod types;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// PCS power-status types. Gated behind the `manta-dispatcher` Cargo
/// feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;

// Canonical names: callers should prefer these over the deeper
// `types::*` paths so the internal layout can evolve without rippling
// through every command.
pub use types::{ManagementState, PowerState, PowerStatus, PowerStatusAll};
