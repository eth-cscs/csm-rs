//! BOS session HTTP bindings — wraps `/bos/v1/session` and
//! `/bos/v2/sessions`. Prefer v2 on releases that expose it.
//!
//! The v1 `impl ShastaClient` block has moved to
//! `crate::bos::wrapper::v1::session`; v1 has no `types.rs` so nothing
//! survives here at the v1 path.

pub mod v2;
