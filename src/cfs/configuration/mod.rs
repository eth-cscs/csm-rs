//! CFS configurations — the ordered list of git-backed Ansible layers
//! applied to a node.
//!
//! Submodules:
//!
//! - [`http_client`] — `ShastaClient` methods for the v2 and v3 endpoints.
//! - [`utils`] — helpers built on top of the raw client.

pub mod http_client;
pub mod utils;
