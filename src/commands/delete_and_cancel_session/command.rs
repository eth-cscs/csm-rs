//! Re-export shim — the implementation lives in
//! [`crate::cfs::cleanup_session`]. Kept here so embedders walking the
//! `commands::*::exec` surface still find a stable entry point under
//! `csm_rs::commands::delete_and_cancel_session::command::exec`.

pub use crate::cfs::cleanup_session::exec;
