//! Boot Script Service (BSS) bindings.
//!
//! BSS stores per-node boot parameters (kernel, initrd, command line)
//! that iPXE fetches at boot time. This module wraps the BSS REST API,
//! exposing it as methods on [`crate::ShastaClient`].
//!
//! Submodules:
//!
//! - [`http_client`] — `ShastaClient` methods that issue BSS HTTP calls.
//! - [`types`] — request/response shapes for the BSS API.
//! - [`utils`] — convenience helpers built on top of the raw client.

pub mod http_client;
#[cfg(test)]
pub mod tests;
pub mod types;
pub mod utils;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// BSS mirror types. Gated behind the `manta-dispatcher` Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;
