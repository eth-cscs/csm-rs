//! HSM groups — named collections of components used by CFS, BOS, and
//! related services as targets.
//!
//! Submodules:
//!
//! - [`http_client`] — `ShastaClient` methods for `/smd/hsm/v2/groups`.
//! - [`types`] — request/response shapes.
//! - [`utils`] — composed helpers (membership unions, substring lookup).
//! - [`hacks`] — workarounds for CSM behaviour that doesn't fit cleanly
//!   into the rest of the surface.

/// Workarounds for CSM HSM behaviour that does not fit cleanly into
/// the rest of the surface.
pub mod hacks;
/// `ShastaClient` methods for `/hsm/v2/groups`.
pub mod http_client;
/// Integration-style tests for the HSM group namespace.
#[cfg(test)]
pub mod tests;
pub mod types;
pub mod utils;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// HSM group mirror types. Gated behind the `manta-dispatcher`
/// Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;
