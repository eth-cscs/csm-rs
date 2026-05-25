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
