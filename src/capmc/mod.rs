//! Cray Advanced Platform Monitoring and Control (CAPMC) bindings.
//!
//! CAPMC is the legacy power-control API for Cray/HPE systems — it
//! handles node power on/off/reset and basic power-status queries. On
//! newer CSM releases its responsibilities are being taken over by PCS
//! (see [`crate::pcs`]); both are wrapped here because both are still in
//! use depending on site and CSM version.
//!
//! Submodules:
//!
//! - [`http_client`] — `ShastaClient` methods that issue CAPMC HTTP calls.
//! - [`types`] — request/response shapes.
//! - [`utils`] — helpers built on top of the raw client.

pub mod http_client;
pub mod types;
pub mod utils;

// Domain-root canonical names for the most commonly used CAPMC
// response types. Callers should prefer these over the deeper
// `types::*` paths, matching the convention established for BOS / IMS
// in Phase 4.
pub use types::{XnameError, XnamePowerActionResponse, XnameStatusResponse};
