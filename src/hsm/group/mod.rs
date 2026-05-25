//! HSM groups — named collections of components used by CFS, BOS, and
//! related services as targets.
//!
//! Submodules:
//!
//! - [`http_client`] — `ShastaClient` methods for `/smd/hsm/v2/groups`.
//! - [`types`] — request/response shapes.
//! - [`utils`] — composed helpers (membership unions, substring lookup).
//! - [`hacks`] — workarounds for CSM behaviour that doesn't fit cleanly
//!   into the rest of the surface.

pub mod hacks;
pub mod http_client;
#[cfg(test)]
pub mod tests;
pub mod types;
pub mod utils;
