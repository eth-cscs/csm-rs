//! Hardware State Manager (HSM) bindings.
//!
//! HSM is the source of truth for what hardware exists in a Shasta
//! system, how it's grouped, and what its current operational state is.
//! Almost every node-targeted operation in CSM (CFS, BOS, PCS, …) starts
//! by resolving identifiers through HSM.
//!
//! Submodules:
//!
//! - [`component`] — individual hardware components (nodes, BMCs, …) and
//!   their identifiers/state.
//! - [`component_status`] — runtime status snapshots for components.
//! - [`group`] — HSM groups (named, possibly nested collections of
//!   components used by CFS/BOS as targets).
//! - [`memberships`] — the membership relation between components and
//!   groups.
//! - [`hw_inventory`] — detailed inventory: HW components, Redfish
//!   endpoints, ethernet interfaces.
//! - [`service`] — service-discovery values (e.g. node roles) exposed
//!   by HSM.

pub mod component;
pub mod component_status;
pub mod group;
pub mod hw_inventory;
pub mod memberships;
pub mod service;
