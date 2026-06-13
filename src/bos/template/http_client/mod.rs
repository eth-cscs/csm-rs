//! BOS session-template HTTP bindings — wraps `/bos/v1/sessiontemplate`
//! and `/bos/v2/sessiontemplates`. Prefer v2 on releases that expose it.
//!
//! The v1 `impl ShastaClient` block has moved to
//! `crate::bos::wrapper::v1::template`; only the hand-written
//! wire-format types remain mounted here at the v1 path.

/// BOS v1 wire-format types. The `impl ShastaClient` block previously
/// hosted here now lives in `crate::bos::wrapper::v1::template`
/// (Task 3 of the BOS progenitor migration). v1 has no spec coverage
/// and no `dispatcher_conv.rs`; this module survives only to host
/// `types`.
pub(crate) mod v1 {
  pub(crate) mod types;
}
pub mod v2;
