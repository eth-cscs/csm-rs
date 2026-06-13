//! `manta`-facing BOS v2 wrapper methods. Per-resource sub-modules
//! (`session`, `template`) attach
//! `impl ShastaClient { pub async fn bos_<resource>_v2_*() }` blocks
//! to the public client. Each sub-module's docstring records the
//! per-method routing decision (generated client vs raw reqwest).
//!
//! See `crate::bos::wrapper` for the shared `gen_client` / `map_err`
//! / `run` helpers.

// Per-resource modules are added by Tasks 4-5:
//   mod session;
//   mod template;
