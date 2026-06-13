//! Power Control Service (PCS) bindings.
//!
//! PCS is the newer power-control API for Shasta, replacing parts of
//! CAPMC (see [`crate::capmc`]) on recent CSM releases. It exposes
//! transitions (power on/off/reset), power status queries, and power
//! capping.
//!
//! Submodules:
//!
//! - [`transitions`] — request power transitions and poll them to
//!   completion.
//! - [`power_status`] — query the current power state of components.
//! - [`power_cap`] — read and update power caps on capable hardware.
//!
//! ## How this module is built
//!
//! Wire-format types and the underlying HTTP client surface are
//! generated from `src/pcs/csm_api_docs.yaml` (OpenAPI 3.0.0). Mirrors
//! the HSM pipeline documented in [`crate::hsm`]; the only structural
//! difference is that the PCS spec is OpenAPI 3.x natively (no
//! Swagger 2.0 conversion needed), so there is no `make convert-spec`
//! step for PCS.
//!
//! 1. `build.rs` runs `progenitor` on the YAML and writes the
//!    generated client to `$OUT_DIR/pcs_generated.rs`.
//! 2. `src/pcs/generated.rs` `include!`s the file as a `pub(crate)`
//!    module — only the wrapper layer and `types.rs` re-export aliases
//!    are allowed to touch it.
//! 3. `src/pcs/wrapper/` glues the generated client (and where the
//!    spec/contract drifts, raw `reqwest` calls) to the public
//!    `ShastaClient::pcs_*` API. Per-resource files (`power_cap.rs`,
//!    `power_status.rs`, `transitions.rs`) host the routing decisions,
//!    documented in each file's module docstring. There is no version
//!    split — PCS exposes a single API version.
//!
//! `power_cap` is the only PCS resource where the migration adopted
//! the generated types wholesale via `pub use` aliases (no
//! `dispatcher_conv.rs` coupling blocking the swap). All 4 power_cap
//! methods route through the generated client; the migration also
//! surfaced 4 latent bugs in the hand-written code that the spec swap
//! exposed and fixed (wrong list/single return type, field-name
//! typo `hostsLimit*` → `hostLimit*`, JSON key casing for
//! `powerCapLimits`, and `PATCH /power-cap` rather than `PUT /power-cap/snapshot`).
//!
//! `power_status` and `transitions` keep hand-written types because
//! their `dispatcher_conv.rs` bridges (96 lines and 182 lines
//! respectively) would cascade a full swap into a
//! `manta-backend-dispatcher` rewrite. Methods stay on raw `reqwest`
//! with per-method routing rationale documented in the wrapper file.
//! The generated client is wired up and ready; per-method progenitor
//! routing is deferred until the public type swap is coordinated.

pub mod power_cap;
pub mod power_status;
pub mod transitions;

pub(crate) mod generated;
mod wrapper;
