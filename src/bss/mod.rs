//! Boot Script Service (BSS) bindings.
//!
//! BSS stores per-node boot parameters (kernel, initrd, command line)
//! that iPXE fetches at boot time. This module wraps the BSS REST API,
//! exposing it as methods on [`crate::ShastaClient`].
//!
//! Submodules:
//!
//! - `wrapper` (private) — `ShastaClient` methods that issue BSS HTTP
//!   calls. Replaces the historic `http_client` submodule.
//! - [`types`] — request/response shapes for the BSS API.
//! - [`utils`] — convenience helpers built on top of the raw client.
//!
//! ## How this module is built
//!
//! Wire-format types and the underlying HTTP client surface are
//! generated from `src/bss/csm_api_docs.yaml`. Mirrors the HSM pipeline
//! documented in [`crate::hsm`]: the spec is Swagger 2.0, so it gets
//! converted to OpenAPI 3.0 via `swagger2openapi` and the converted
//! JSON (`src/bss/csm_api_docs.openapi3.json`) is committed alongside
//! the YAML.
//!
//! 1. **Developer step:** `npx swagger2openapi src/bss/csm_api_docs.yaml
//!    -o src/bss/csm_api_docs.openapi3.json` — re-run whenever the YAML
//!    changes; the JSON is committed.
//! 2. `build.rs` runs `progenitor` on the JSON and writes the generated
//!    client to `$OUT_DIR/bss_generated.rs`.
//! 3. `src/bss/generated.rs` `include!`s the file as a `pub(crate)`
//!    module — only the wrapper layer and `types.rs` re-export aliases
//!    are allowed to touch it.
//! 4. `src/bss/wrapper/mod.rs` glues the generated client (and where
//!    the spec/contract drifts, raw `reqwest` calls) to the public
//!    `ShastaClient::bss_*` API. Per-method routing decisions
//!    (progenitor vs raw `reqwest`) are documented in the file's
//!    module docstring.
//!
//! `src/bss/types.rs` holds the public `BootParameters` type. As of the
//! migration commit train ending at `bss_bootparameters` (commit
//! `77cab21`), the type is hand-written rather than re-aliased to the
//! generated `BootParams`. The reason: the public `BootParameters`
//! carries 9 instance methods (kernel-parameter manipulation helpers,
//! `get_boot_image`, etc.) that depend on non-Option fields. The
//! generated equivalent makes those fields `Option<String>`, which
//! would cascade `unwrap` changes through every caller and force a
//! coordinated `manta-backend-dispatcher` release. The generated client
//! is wired up and ready; the type swap is a follow-up.

pub(crate) mod generated;
/// Integration-style tests for the BSS namespace.
#[cfg(test)]
pub mod tests;
pub mod types;
pub mod utils;
mod wrapper;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// BSS mirror types. Gated behind the `manta-dispatcher` Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;

// Canonical names: callers should prefer these over the deeper
// `types::*` paths so the internal layout can evolve without rippling
// through every command.
pub use types::BootParameters;
