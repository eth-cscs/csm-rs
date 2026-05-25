//! BOS session templates — reusable definitions ("boot this image with
//! this CFS configuration against these nodes").
//!
//! Submodules:
//!
//! - [`http_client`] — `ShastaClient` methods for v1 and v2.
//! - [`utils`] — helpers built on top of the raw client.

pub mod http_client;
pub mod utils;
