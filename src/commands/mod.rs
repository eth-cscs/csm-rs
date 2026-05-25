//! High-level "command" workflows that compose multiple CSM API calls.
//!
//! Each submodule implements one administrator-facing operation by
//! orchestrating several lower-level calls across [`crate::cfs`],
//! [`crate::ims`], [`crate::bos`], [`crate::hsm`], etc.
//!
//! Submodules:
//!
//! - [`apply_hw_cluster_pin`] — apply a hardware pattern to (re)compose
//!   an HSM group from a parent group.
//! - [`apply_session`] — run a CFS session against a set of nodes.
//! - [`delete_and_cancel_session`] — cancel an in-flight CFS session and
//!   clean up its derived resources.
//! - [`delete_configurations_and_data_related`] — remove a CFS
//!   configuration along with its dependent images and session templates.
//! - [`get_images_and_details`] — fetch IMS images plus the CFS
//!   configurations and BOS templates that reference them.
//! - [`i_apply_sat_file`] — apply a SAT (System Admin Toolkit) YAML file.
//! - [`migrate_backup`] / [`migrate_restore`] — export or import the
//!   CSM-side artifacts required to move a cluster between systems.

pub mod apply_hw_cluster_pin;
pub mod apply_session;
pub mod delete_and_cancel_session;
pub mod delete_configurations_and_data_related;
pub mod get_images_and_details;
pub mod i_apply_sat_file;
pub mod migrate_backup;
pub mod migrate_restore;
