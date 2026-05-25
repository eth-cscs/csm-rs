//! IMS jobs — builds that turn a recipe into an image.
//!
//! Submodules:
//!
//! - [`http_client`] — `ShastaClient` methods for `/ims/v3/jobs`.
//! - [`types`] — request/response shapes.
//! - [`utils`] — helpers built on top of the raw client.

pub mod http_client;
pub mod types;
pub mod utils;
