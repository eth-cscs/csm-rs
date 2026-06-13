//! Boot Script Service (BSS) bindings.
//!
//! BSS stores per-node boot parameters (kernel, initrd, command line)
//! that iPXE fetches at boot time. This module wraps the BSS REST API,
//! exposing it as methods on [`crate::ShastaClient`].
//!
//! Submodules:
//!
//! - `wrapper` (private) — `ShastaClient` methods that issue BSS HTTP
//!   calls. Replaces the historic `http_client` submodule.
//! - [`types`] — request/response shapes for the BSS API.
//! - [`utils`] — convenience helpers built on top of the raw client.

pub(crate) mod generated;
/// Integration-style tests for the BSS namespace.
#[cfg(test)]
pub mod tests;
pub mod types;
pub mod utils;
mod wrapper;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// BSS mirror types. Gated behind the `manta-dispatcher` Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;

// Canonical names: callers should prefer these over the deeper
// `types::*` paths so the internal layout can evolve without rippling
// through every command.
pub use types::BootParameters;
