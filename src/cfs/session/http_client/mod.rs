//! CFS session HTTP bindings — wraps `/cfs/v2/sessions` and
//! `/cfs/v3/sessions`. Prefer v3 on releases that expose it.

pub mod v2;
pub mod v3;
