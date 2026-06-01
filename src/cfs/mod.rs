//! Configuration Framework Service (CFS) bindings.
//!
//! CFS is the Ansible-driven configuration engine for Shasta: it owns
//! the *configurations* (an ordered list of git-backed Ansible layers),
//! the *components* (per-node desired/observed state of those layers),
//! and the *sessions* that actually run Ansible against nodes.
//!
//! Submodules:
//!
//! - [`configuration`] — CFS configurations (v2 and v3 endpoints).
//! - [`component`] — per-node component records (v2 and v3 endpoints).
//! - [`session`] — CFS sessions (v2 and v3 endpoints).
//! - [`common`] — shared helpers used across the CFS resources.
//! - [`health`] — liveness/readiness checks for the CFS service itself.
//!
//! The v3 endpoints are preferred on CSM releases that expose them; the
//! v2 endpoints are kept for sites still on older CSM.

pub mod common;
pub mod component;
pub mod configuration;
pub mod health;
pub mod session;
#[cfg(test)]
pub mod tests;

// Domain-root canonical names. CFS exposes both v2 (legacy) and v3
// (current) endpoints on `ShastaClient`; these re-exports pin **v3** as
// the canonical wire-format for external consumers. v2 types remain
// reachable to internal callers via the deep `*::http_client::v2::types`
// paths but are `pub(crate)`-only since Phase 5.2.
//
// v2 and v3 are structurally distinct (different field renames,
// different optional fields, an additional pagination `Next` / `logs`
// in v3, etc.) — picking a canonical here is a load-bearing decision,
// not cosmetic.
pub use component::http_client::v3::types::Component;
pub use configuration::http_client::v3::types::cfs_configuration_request::CfsConfigurationRequest;
pub use configuration::http_client::v3::types::cfs_configuration_response::CfsConfigurationResponse;
pub use session::http_client::v3::types::{
  CfsSessionGetResponse, CfsSessionPostRequest,
};
