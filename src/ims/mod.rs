//! Image Management Service (IMS) bindings.
//!
//! IMS owns the OS images, recipes, and build jobs that BOS later boots
//! nodes from. It also stores the SSH public keys IMS jobs need.
//!
//! Submodules:
//!
//! - [`image`] — IMS images (the immutable, bootable artifacts).
//! - [`recipe`] — IMS recipes (the inputs from which an image is built).
//! - [`job`] — IMS jobs (the build that turns a recipe into an image).
//! - [`public_keys`] — SSH public keys registered with IMS.
//! - [`s3_client`] — low-level S3 client used to upload/download IMS
//!   artifacts directly from the CSM-backing S3 store.

pub mod image;
pub mod job;
/// IMS public-key endpoints — register and look up user SSH keys.
pub mod public_keys;
/// IMS recipe endpoints — base images that get customised into final
/// images via CFS sessions.
pub mod recipe;
/// Low-level S3 client used to upload/download IMS artifacts directly
/// from the CSM-backing S3 store.
pub mod s3_client;
