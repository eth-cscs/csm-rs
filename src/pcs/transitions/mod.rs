//! PCS power transitions — request power-state changes and poll for
//! completion. Wraps `/power-control/v1/transitions`.

/// `ShastaClient` methods for `/power-control/v1/transitions`.
pub mod http_client;
/// Request / response types for the PCS transitions endpoints.
pub mod types;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// PCS transitions types. Gated behind the `manta-dispatcher` Cargo
/// feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;

// Canonical names: callers should prefer these over the deeper
// `types::*` paths so the internal layout can evolve without rippling
// through every command.
pub use types::{
  Location, Operation, Task, TaskCounts, Transition, TransitionResponse,
  TransitionResponseList, TransitionStartOutput,
};
