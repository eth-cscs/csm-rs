//! CFS configuration HTTP bindings — wraps `/cfs/v2/configurations` and
//! `/cfs/v3/configurations`. Prefer v3 on releases that expose it.

pub mod v2;
pub mod v3;
