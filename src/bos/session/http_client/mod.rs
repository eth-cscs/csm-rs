//! BOS session HTTP bindings — wraps `/bos/v1/session` and
//! `/bos/v2/sessions`. Prefer v2 on releases that expose it.

pub mod v1;
pub mod v2;
