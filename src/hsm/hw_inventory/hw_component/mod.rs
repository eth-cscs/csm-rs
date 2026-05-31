//! Hardware component records under `/smd/hsm/v2/Inventory/Hardware`.
//!
//! Submodules:
//!
//! - [`http_client`] — `ShastaClient` methods.
//! - [`types`] — request/response shapes.
//! - [`utils`] — helpers built on top of the raw client.

pub mod http_client;
#[macro_use]
pub mod types;
pub mod utils;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// HSM hardware-component mirror types. Gated behind the
/// `manta-dispatcher` Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;
