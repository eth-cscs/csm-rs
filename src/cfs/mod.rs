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
//! - [`cleanup`] — cascade-delete a CFS configuration along with the
//!   IMS images, CFS sessions, and BOS templates derived from it.
//! - [`health`] — liveness/readiness checks for the CFS service itself.
//!
//! The v3 endpoints are preferred on CSM releases that expose them; the
//! v2 endpoints are kept for sites still on older CSM.

pub mod cleanup;
pub mod cleanup_session;
pub mod common;
pub mod component;
pub mod configuration;
pub mod health;
pub mod session;
#[cfg(test)]
pub mod tests;

// CFS exposes both v2 (legacy) and v3 (current) endpoints on
// `ShastaClient`. v2 and v3 are structurally distinct on the wire
// (different field renames, different optional fields, pagination
// `Next`/`logs` only in v3), so they are not interchangeable. Each
// `ShastaClient::cfs_*_v2_*` method takes/returns v2 types; each
// `cfs_*_v3_*` method takes/returns v3 types — pick the submodule
// (`cfs::v2::*` or `cfs::v3::*`) that matches the methods you call.
//
// The deep `*::http_client::v{2,3}::types::*` paths stay `pub(crate)`
// from Phase 5.2 so external callers reach the types only through
// these two submodules.

/// Legacy CFS v2 endpoint types — still supported on older CSM
/// installs where v3 is unavailable.
pub mod v2 {
  pub use super::component::http_client::v2::types::{Component, State};
  pub use super::configuration::http_client::v2::types::cfs_configuration_request::CfsConfigurationRequest;
  pub use super::configuration::http_client::v2::types::cfs_configuration_response::{
    CfsConfigurationResponse, Layer,
  };
  pub use super::session::http_client::v2::types::{
    Ansible, Artifact, CfsSessionGetResponse, CfsSessionPostRequest,
    Configuration, Group, Session, Status, Target,
  };
}

/// Current CFS v3 endpoint types.
pub mod v3 {
  pub use super::component::http_client::v3::types::Component;
  pub use super::configuration::http_client::v3::types::cfs_configuration_request::CfsConfigurationRequest;
  pub use super::configuration::http_client::v3::types::cfs_configuration_response::CfsConfigurationResponse;
  pub use super::session::http_client::v3::types::{
    CfsSessionGetResponse, CfsSessionPostRequest, Configuration, Session,
    Status, Target,
  };
}
