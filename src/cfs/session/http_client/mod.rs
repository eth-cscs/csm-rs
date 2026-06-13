//! CFS session HTTP bindings — wraps `/cfs/v2/sessions` and
//! `/cfs/v3/sessions`. Prefer v3 on releases that expose it.
//!
//! The v2 `impl ShastaClient` block has moved to
//! `crate::cfs::wrapper::v2::session`; only the wire-format types
//! and the dispatcher conversion impls remain mounted here.

/// CFS v2 wire-format types and dispatcher conversion shims. The
/// `impl ShastaClient` block previously hosted here now lives in
/// `crate::cfs::wrapper::v2::session` (Task 7 of the CFS progenitor
/// migration). This module survives only to host `types` (and, under
/// the `manta-dispatcher` feature, `dispatcher_conv`).
pub(crate) mod v2 {
  pub(crate) mod types;

  /// Bidirectional `From` impls between [`types`] and the dispatcher's
  /// CFS v2 session mirror types. Gated behind the `manta-dispatcher`
  /// Cargo feature.
  #[cfg(feature = "manta-dispatcher")]
  mod dispatcher_conv;
}

pub mod v3;
