//! CFS configuration HTTP bindings — wraps `/cfs/v2/configurations` and
//! `/cfs/v3/configurations`. Prefer v3 on releases that expose it.
//!
//! The v2 `impl ShastaClient` block has moved to
//! `crate::cfs::wrapper::v2::configuration`; only the wire-format types
//! and the dispatcher conversion impls remain mounted here.

/// CFS v2 wire-format types and dispatcher conversion shims. The
/// `impl ShastaClient` block previously hosted here now lives in
/// `crate::cfs::wrapper::v2::configuration` (Task 5 of the CFS progenitor
/// migration). This module survives only to host `types` (and, under
/// the `manta-dispatcher` feature, `dispatcher_conv`).
pub(crate) mod v2 {
  pub(crate) mod types;
}

pub mod v3;
