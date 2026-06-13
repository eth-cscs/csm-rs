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
//!
//! ## How this module is built
//!
//! Wire-format types and the underlying HTTP client surface are
//! generated from `src/cfs/csm_api_docs.yaml` (OpenAPI 3.0.2). Mirrors
//! the HSM pipeline documented in [`crate::hsm`]; the only structural
//! difference is that the CFS spec is OpenAPI 3.x natively (no
//! Swagger 2.0 conversion needed), so there is no `make convert-spec`
//! step for CFS.
//!
//! 1. `build.rs` runs `progenitor` on the YAML and writes the
//!    generated client to `$OUT_DIR/cfs_generated.rs`.
//! 2. `src/cfs/generated.rs` `include!`s the file as a `pub(crate)`
//!    module — only the wrapper layer and `types.rs` re-export aliases
//!    are allowed to touch it.
//! 3. `src/cfs/wrapper/` glues the generated client (and where the
//!    spec/contract drifts, raw `reqwest` calls) to the public
//!    `ShastaClient::cfs_*` API. Per-method routing decisions
//!    (progenitor vs raw `reqwest`) are documented in each per-resource
//!    file's module docstring. The wrapper is split into `v2/` and
//!    `v3/` subfolders so the API-version boundary is visible in the
//!    directory tree.
//!
//! Per-resource `types.rs` files are either pure re-exports of
//! generated types, or hand-rolled wire types where a full swap to
//! generated types would cascade through `dispatcher_conv` bridges. As
//! of the migration commit train ending at session v3, all per-resource
//! `types.rs` files remain hand-rolled because of the
//! `manta-backend-dispatcher` coupling; the generated client is wired
//! up and ready, but per-method progenitor routing is deferred until
//! the public type swap is coordinated.

pub mod cleanup;
pub mod cleanup_session;
pub mod common;
pub mod component;
pub mod configuration;
pub(crate) mod generated;
pub mod health;
pub mod session;
mod wrapper;
/// Integration-style tests for the CFS namespace.
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
