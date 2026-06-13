//! `manta`-facing CFS v2 wrapper methods. Per-resource sub-modules
//! (`component`, `configuration`, `session`) attach
//! `impl ShastaClient { pub async fn cfs_<resource>_v2_*() }` blocks
//! to the public client. Each sub-module's docstring records the
//! per-method routing decision (generated client vs raw reqwest).
//!
//! See `crate::cfs::wrapper` for the shared `gen_client` / `map_err`
//! / `run` helpers — they're version-agnostic and serve both v2 and v3.

// Per-resource modules are added by Tasks 3, 5, 7:
//   mod component;
//   mod configuration;
//   mod session;
