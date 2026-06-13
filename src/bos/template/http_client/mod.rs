//! BOS session-template HTTP bindings — wraps `/bos/v1/sessiontemplate`
//! and `/bos/v2/sessiontemplates`. Prefer v2 on releases that expose it.
//!
//! Both the v1 and v2 `impl ShastaClient` blocks have moved to
//! `crate::bos::wrapper::v{1,2}::template`; only the hand-written
//! wire-format `types` (and the v2 `dispatcher_conv`) remain mounted
//! here.

/// BOS v1 wire-format types. The `impl ShastaClient` block previously
/// hosted here now lives in `crate::bos::wrapper::v1::template`
/// (Task 3 of the BOS progenitor migration). v1 has no spec coverage
/// and no `dispatcher_conv.rs`; this module survives only to host
/// `types`.
pub(crate) mod v1 {
  pub(crate) mod types;
}

/// BOS v2 wire-format types and dispatcher conversions. The `impl
/// ShastaClient` block previously hosted here now lives in
/// `crate::bos::wrapper::v2::template` (Task 5 of the BOS progenitor
/// migration).
pub(crate) mod v2 {
  pub(crate) mod types;
  #[cfg(feature = "manta-dispatcher")]
  mod dispatcher_conv;
}
