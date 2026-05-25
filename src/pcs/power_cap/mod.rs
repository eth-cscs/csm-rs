//! PCS power caps — read and update power caps on capable hardware.
//! Wraps `/power-control/v1/power-cap`.

/// `ShastaClient` methods for `/power-control/v1/power-cap`.
pub mod http_client;
/// Request / response types for the PCS power-cap endpoints.
pub mod types;
