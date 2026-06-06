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
//!
//! `node::types` and `node::utils` are crate-internal — their helpers
//! are surfaced through the `ShastaClient` and `commands` layers.

/// Open and interact with a node's serial console via the CSM
/// `cray-console-operator` / `cray-console-node` services. Requires
/// the `k8s-console` Cargo feature (Kubernetes client).
#[cfg(feature = "k8s-console")]
pub mod console;
pub mod types;
pub mod utils;
