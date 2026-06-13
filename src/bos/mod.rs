//! Boot Orchestration Service (BOS) bindings.
//!
//! BOS coordinates booting, configuring, and shutting down compute nodes
//! against a defined session template. This module exposes the v1 and v2
//! BOS REST APIs as methods on [`crate::ShastaClient`].
//!
//! Submodules:
//!
//! - [`template`] — session templates (the reusable definition of "boot
//!   this image, with this CFS configuration, against these nodes").
//! - [`session`] — sessions (a single invocation of a template).
//! - [`health_check`] — helpers for liveness/readiness probes against the
//!   BOS service itself.
//!
//! Both v1 and v2 endpoints are wrapped; new code should generally prefer
//! the v2 variants where available.

pub(crate) mod generated;
pub mod health_check;
pub mod session;
pub mod template;
mod wrapper;
/// Integration-style tests for the BOS namespace.
#[cfg(test)]
pub mod tests;

// Domain-root canonical names for the most commonly used BOS types.
// Callers should prefer these over the deep `*::http_client::v2::types::*`
// paths so an eventual v3 bump only needs to flip these re-exports.
pub use session::http_client::v2::types::{BosSession, Operation};
pub use template::http_client::v2::types::{BootSet, BosSessionTemplate, Cfs};
