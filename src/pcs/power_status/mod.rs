//! PCS power status — query the current power state of components.
//! Wraps `/power-control/v1/power-status`.

/// `ShastaClient` methods for `/power-control/v1/power-status`.
pub mod http_client;
/// Request / response types for the PCS power-status endpoints.
pub mod types;
