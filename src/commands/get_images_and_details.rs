//! Fetch IMS images plus the CFS configurations and BOS templates that reference them.
//!
//! Thin facade over [`crate::ims::image::utils::get_with_details`].
//! The orchestration logic lives in the IMS namespace; this module
//! exists so the `commands::get_images_and_details::exec` entry point
//! is reachable for embedders that walk the `commands` surface.

use crate::{error::Error, ims::image::http_client::types::Image};

/// See [`crate::ims::image::utils::get_with_details`] for the full
/// description.
///
/// # Errors
///
/// Returns an [`Error`] variant on CSM, transport, or
/// deserialization failure; see the crate-level `Error` enum
/// for the full set.
pub async fn get_images_and_details(
  client: &crate::ShastaClient,
  shasta_token: &str,
  hsm_group_name_vec: &[String],
  id_opt: Option<&str>,
  limit_number: Option<&u8>,
) -> Result<Vec<(Image, String, String, bool)>, Error> {
  crate::ims::image::utils::get_with_details(
    client,
    shasta_token,
    hsm_group_name_vec,
    id_opt,
    limit_number,
  )
  .await
}
