//! CFS configuration HTTP bindings — wraps `/cfs/v2/configurations` and
//! `/cfs/v3/configurations`. Prefer v3 on releases that expose it.
//!
//! The v2 and v3 `impl ShastaClient` blocks have moved to
//! `crate::cfs::wrapper::v{2,3}::configuration`; only the wire-format
//! types and the dispatcher conversion impls remain mounted here.

/// CFS v2 wire-format types and dispatcher conversion shims. The
/// `impl ShastaClient` block previously hosted here now lives in
/// `crate::cfs::wrapper::v2::configuration` (Task 5 of the CFS progenitor
/// migration). This module survives only to host `types` (and, under
/// the `manta-dispatcher` feature, `dispatcher_conv`).
pub(crate) mod v2 {
  pub(crate) mod types;
}

/// CFS v3 wire-format types and dispatcher conversion shims. The
/// `impl ShastaClient` block previously hosted here now lives in
/// `crate::cfs::wrapper::v3::configuration` (Task 6 of the CFS progenitor
/// migration). This module survives only to host `types` (whose
/// `mod.rs` already gates the `dispatcher_conv` submodule behind the
/// `manta-dispatcher` feature).
pub(crate) mod v3 {
  pub(crate) mod types;
}
