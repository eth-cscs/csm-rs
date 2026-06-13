//! Power Control Service (PCS) bindings.
//!
//! PCS is the newer power-control API for Shasta, replacing parts of
//! CAPMC (see [`crate::capmc`]) on recent CSM releases. It exposes
//! transitions (power on/off/reset), power status queries, and power
//! capping.
//!
//! Submodules:
//!
//! - [`transitions`] — request power transitions and poll them to
//!   completion.
//! - [`power_status`] — query the current power state of components.
//! - [`power_cap`] — read and update power caps on capable hardware.

pub mod power_cap;
pub mod power_status;
pub mod transitions;

pub(crate) mod generated;
mod wrapper;
