//! HSM memberships — the relation between components and groups.
//!
//! Wraps `/smd/hsm/v2/memberships`. Submodules:
//!
//! - [`http_client`] — `ShastaClient` methods.
//! - [`types`] — request/response shapes.

pub mod http_client;
pub mod types;
