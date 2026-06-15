//! BOS session HTTP bindings — wraps `/bos/v1/session` and
//! `/bos/v2/sessions`. Prefer v2 on releases that expose it.
//!
//! The v1 `impl ShastaClient` block has moved to
//! `crate::bos::wrapper::v1::session`; v1 has no `types.rs` so nothing
//! survives here at the v1 path. The v2 `impl ShastaClient` block has
//! moved to `crate::bos::wrapper::v2::session`; the wire-format
//! `types.rs` (and its dispatcher conversions) stay here so the
//! domain-root re-exports in `crate::bos` (and the dispatcher trait
//! impls) keep working.

pub(crate) mod v2 {
  pub(crate) mod types;
  #[cfg(feature = "manta-dispatcher")]
  mod dispatcher_conv;
}
