//! Node roles reported under `/smd/hsm/v2/service/values/role`.
//!
//! The live-fetch method (`ShastaClient::hsm_roles_get`) is now defined
//! in `crate::hsm::wrapper::service_values` against the
//! progenitor-generated client.
//!
//! Submodules:
//!
//! - [`hardcoded_values`] — built-in fallback list for offline use.
//! - [`types`] — re-export of the generated wire-format type.

pub mod hardcoded_values;
pub mod types;
