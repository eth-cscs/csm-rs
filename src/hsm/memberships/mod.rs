//! HSM memberships — the relation between components and groups.
//!
//! Wraps `/smd/hsm/v2/memberships`. Submodules:
//!
//! - [`types`] — response shape, re-exported from the progenitor-generated
//!   client.
//!
//! `ShastaClient` methods live in `crate::hsm::wrapper::memberships` and
//! delegate to the generated client.

pub mod types;
