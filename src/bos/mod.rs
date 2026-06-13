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
//!
//! Liveness/readiness probes against the BOS service itself are exposed
//! as the [`ShastaClient::bos_health_check`](crate::ShastaClient::bos_health_check)
//! method, implemented in the internal `wrapper::health_check` module.
//!
//! Both v1 and v2 endpoints are wrapped; new code should generally prefer
//! the v2 variants where available.
//!
//! ## How this module is built
//!
//! Wire-format types and the underlying HTTP client surface are
//! generated from `src/bos/csm_api_docs.yaml` (OpenAPI 3.0.3). Mirrors
//! the HSM pipeline documented in [`crate::hsm`]; the only structural
//! difference is that the BOS spec is OpenAPI 3.x natively (no
//! Swagger 2.0 conversion needed), so there is no `make convert-spec`
//! step for BOS.
//!
//! 1. `build.rs` runs `progenitor` on the YAML and writes the
//!    generated client to `$OUT_DIR/bos_generated.rs`.
//! 2. `src/bos/generated.rs` `include!`s the file as a `pub(crate)`
//!    module — only the wrapper layer and `types.rs` re-export aliases
//!    are allowed to touch it.
//! 3. `src/bos/wrapper/` glues the generated client (and where the
//!    spec/contract drifts, raw `reqwest` calls) to the public
//!    `ShastaClient::bos_*` API. Per-method routing decisions
//!    (progenitor vs raw `reqwest`) are documented in each per-resource
//!    file's module docstring. The wrapper is split into `v1/` (raw
//!    reqwest only — the spec is v2-only) and `v2/` subfolders so the
//!    API-version boundary is visible in the directory tree, plus a
//!    `health_check.rs` for the singleton `/v2/healthz` route.
//!
//! Per-resource `types.rs` files are hand-rolled (not pure re-exports
//! of generated types) where a full swap would cascade through
//! `dispatcher_conv` bridges (`session/http_client/v2/dispatcher_conv.rs`
//! and `template/http_client/v2/dispatcher_conv.rs`). The generated
//! client is wired up and ready, but per-method progenitor routing is
//! deferred for the methods where the cost-of-swap outweighs the
//! benefit (same pattern as the CFS and BSS migrations). As of the
//! migration commit train ending at the health_check task, only
//! `bos_health_check` routes through the generated client via a
//! `serde_json::to_value` boundary conversion; v1/v2 session and
//! template methods stay on raw `reqwest`.

pub(crate) mod generated;
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
