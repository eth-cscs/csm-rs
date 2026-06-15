//! Hardware State Manager (HSM) bindings.
//!
//! HSM is the source of truth for what hardware exists in a Shasta
//! system, how it's grouped, and what its current operational state is.
//! Almost every node-targeted operation in CSM (CFS, BOS, PCS, …) starts
//! by resolving identifiers through HSM.
//!
//! Submodules:
//!
//! - [`component`] — individual hardware components (nodes, BMCs, …) and
//!   their identifiers/state.
//! - [`component_status`] — runtime status snapshots for components.
//! - [`group`] — HSM groups (named, possibly nested collections of
//!   components used by CFS/BOS as targets).
//! - [`memberships`] — the membership relation between components and
//!   groups.
//! - [`hw_inventory`] — detailed inventory: HW components, Redfish
//!   endpoints, ethernet interfaces.
//! - [`service`] — service-discovery values (e.g. node roles) exposed
//!   by HSM.
//!
//! ## How this module is built
//!
//! Wire-format types and the underlying HTTP client surface are
//! generated from `src/hsm/csm_api_docs.yaml`:
//!
//! 1. **Developer step:** `make convert-spec` converts the Swagger 2.0
//!    YAML to OpenAPI 3.0 (`src/hsm/csm_api_docs.openapi3.json`) using
//!    `swagger2openapi`. Re-run whenever the YAML changes; the JSON is
//!    committed.
//! 2. `build.rs` runs `progenitor` on the JSON and writes the generated
//!    client to `$OUT_DIR/hsm_generated.rs`.
//! 3. `src/hsm/generated.rs` `include!`s the file as a `pub(crate)`
//!    module — only the wrapper layer and `types.rs` re-export aliases
//!    are allowed to touch it.
//! 4. `src/hsm/wrapper/` glues the generated client (and where the
//!    spec/contract drift, raw `reqwest` calls) to the public
//!    `ShastaClient::hsm_*` API. Per-method routing decisions
//!    (progenitor vs raw `reqwest`) are documented in each per-resource
//!    file's module docstring.
//!
//! Per-resource `types.rs` files are either pure re-exports of
//! generated types, or hand-rolled wire types where a full swap to
//! generated types would cascade through `dispatcher_conv` bridges
//! (`hw_inventory/hw_component`, `hw_inventory/redfish_endpoint`,
//! `hw_inventory/ethernet_interfaces`). Projection types like
//! `NodeSummary` live in the wrapper module
//! (`src/hsm/wrapper/hw_component_types.rs`) and are re-exported
//! through the existing public path.

pub mod component;
pub mod component_status;
pub mod group;
pub mod hw_inventory;
pub mod memberships;
pub mod service;
pub(crate) mod generated;
mod wrapper;
/// Shared HSM response types (`HsmActionResponse`, `ResourceURI`) used
/// across the submodules above.
pub mod types;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// HSM mirrors. Gated behind the `manta-dispatcher` Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;
