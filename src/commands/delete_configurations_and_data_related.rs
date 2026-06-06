//! Re-export shim — the implementation lives in
//! [`crate::cfs::cleanup`]. Kept here so embedders walking the
//! `commands::*::exec` surface still find a stable entry point under
//! `csm_rs::commands::delete_configurations_and_data_related::*`.

pub use crate::cfs::cleanup::{delete, get_data_to_delete};
