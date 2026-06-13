//! Hardware component records under `/smd/hsm/v2/Inventory/Hardware`.
//!
//! Submodules:
//!
//! - [`types`] — request/response shapes (plus re-exports of the
//!   projection types [`types::NodeSummary`], [`types::ArtifactSummary`],
//!   [`types::ArtifactType`] which physically live in the wrapper
//!   layer per the design decision in
//!   `docs/superpowers/specs/2026-06-13-progenitor-hsm-codegen-design.md`).
//! - [`utils`] — helpers built on top of the raw client.
//!
//! The `ShastaClient` methods (`hsm_hw_inventory_get`,
//! `hsm_hw_inventory_get_query`, `hsm_hw_inventory_post`) now live in
//! `crate::hsm::wrapper::hw_component` (private module — call them via
//! the inherent `ShastaClient::hsm_hw_inventory_*` methods).

#[macro_use]
pub mod types;
pub mod utils;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// HSM hardware-component mirror types. Gated behind the
/// `manta-dispatcher` Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;
