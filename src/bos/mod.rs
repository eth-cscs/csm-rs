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

pub mod health_check;
pub mod session;
pub mod template;
#[cfg(test)]
pub mod tests;
