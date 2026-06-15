//! Detailed HSM hardware inventory.
//!
//! Submodules:
//!
//! - [`hw_component`] — fine-grained hardware-component records.
//! - [`ethernet_interfaces`] — node ethernet interfaces.
//! - [`redfish_endpoint`] — Redfish endpoints registered with HSM.

pub mod ethernet_interfaces;
pub mod hw_component;
pub mod redfish_endpoint;
