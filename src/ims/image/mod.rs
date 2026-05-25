//! IMS images — immutable, bootable OS artifacts.
//!
//! Submodules:
//!
//! - [`http_client`] — `ShastaClient` methods for `/ims/v3/images`.
//! - [`utils`] — helpers built on top of the raw client.

pub mod http_client;
pub mod utils;
