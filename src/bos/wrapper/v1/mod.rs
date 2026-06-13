//! `manta`-facing BOS v1 wrapper methods. The upstream BOS spec is
//! v2-only — these are kept on raw `reqwest` and migrate as pure file
//! relocations. Per-resource sub-modules (`session`, `template`) attach
//! `impl ShastaClient { pub async fn bos_<resource>_v1_*() }` blocks
//! to the public client.
//!
//! See `crate::bos::wrapper` for the shared `gen_client` / `map_err`
//! / `run` helpers — they're version-agnostic but v1 methods do not
//! use them (no spec to drive progenitor against).

mod session;
mod template;
