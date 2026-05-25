//! Node-level helpers that don't map 1:1 to a single CSM service.
//!
//! Where [`crate::hsm`] is the authoritative inventory API and
//! [`crate::bos`] / [`crate::pcs`] handle boot and power, this module
//! collects operations that act on individual nodes across those APIs.
//!
//! Submodules:
//!
//! - [`console`] — open and interact with a node's serial console via
//!   the CSM `cray-console-operator` / `cray-console-node` services.
//! - [`types`] — shared node-shaped data structures.
//! - [`utils`] — helper functions composed from the lower-level APIs.

pub mod console;
pub mod types;
pub mod utils;
