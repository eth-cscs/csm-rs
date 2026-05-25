//! Hardware component records under `/smd/hsm/v2/Inventory/Hardware`.
//!
//! Submodules:
//!
//! - [`http_client`] — `ShastaClient` methods.
//! - [`types`] — request/response shapes.
//! - [`utils`] — helpers built on top of the raw client.

pub mod http_client;
pub mod types;
pub mod utils;
