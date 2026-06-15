//! CFS component HTTP bindings — wraps `/cfs/v2/components` and
//! `/cfs/v3/components`. Prefer v3 on releases that expose it.
//!
//! The v2 and v3 `impl ShastaClient` blocks have moved to
//! `crate::cfs::wrapper::v{2,3}::component`; only the wire-format types
//! and the dispatcher conversion impls remain mounted here.

/// CFS v2 wire-format types and dispatcher conversion shims. The
/// `impl ShastaClient` block previously hosted here now lives in
/// `crate::cfs::wrapper::v2::component` (Task 3 of the CFS progenitor
/// migration). This module survives only to host `types` (and, under
/// the `manta-dispatcher` feature, `dispatcher_conv`).
pub(crate) mod v2 {
  pub(crate) mod types;

  /// Bidirectional `From` impls between [`types`] and the dispatcher's
  /// CFS v2 component mirror types. Gated behind the `manta-dispatcher`
  /// Cargo feature.
  #[cfg(feature = "manta-dispatcher")]
  mod dispatcher_conv;
}

/// CFS v3 wire-format types and dispatcher conversion shims. The
/// `impl ShastaClient` block previously hosted here now lives in
/// `crate::cfs::wrapper::v3::component` (Task 4 of the CFS progenitor
/// migration). This module survives only to host `types` (and, under
/// the `manta-dispatcher` feature, `dispatcher_conv`).
pub(crate) mod v3 {
  pub(crate) mod types;

  /// Bidirectional `From` impls between [`types`] and the dispatcher's
  /// CFS v3 component mirror types. Gated behind the `manta-dispatcher`
  /// Cargo feature.
  #[cfg(feature = "manta-dispatcher")]
  mod dispatcher_conv;
}
