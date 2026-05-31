//! HSM components — individual hardware components (nodes, BMCs, …) and
//! their identifiers/state.
//!
//! Submodules:
//!
//! - [`http_client`] — `ShastaClient` methods for `/smd/hsm/v2/State/Components`.
//! - [`types`] — request/response shapes.

pub mod http_client;
pub mod types;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// HSM component mirror types. Gated behind the `manta-dispatcher`
/// Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;
