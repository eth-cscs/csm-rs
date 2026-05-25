//! Node roles reported under `/smd/hsm/v2/service/values/role`.
//!
//! Submodules:
//!
//! - [`http_client`] — `ShastaClient` method to fetch the live list.
//! - [`hardcoded_values`] — built-in fallback list for offline use.
//! - [`types`] — response shape.

pub mod hardcoded_values;
pub mod http_client;
pub mod types;
