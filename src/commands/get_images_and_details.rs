//! Fetch IMS images plus the CFS configurations and BOS templates that reference them.

use crate::{
  error::Error,
  ims::image::{self, http_client::types::Image},
};

/// Fetch IMS images plus the CFS configurations, BOS session
/// templates, and boot-status they relate to, filtered by HSM group.
///
/// Returns a list of tuples
/// `(Image, cfs_configuration_name, targets, is_boot_image)` where
/// `targets` is either a list of HSM group names or xnames, and
/// `is_boot_image` indicates whether the image is currently used to
/// boot a node.
///
/// Filtering is performed against multiple sources to avoid missing
/// images:
///
/// - CFS sessions — to find which image id was created against which
///   HSM group.
/// - BOS session templates — to find the HSM group related to nodes
///   rebooted in the past.
/// - Image ids in BSS boot parameters for nodes in the target HSM
///   groups — needed to catch images currently used whose name does
///   not contain the HSM group and which aren't referenced by any CFS
///   session or BOS session template.
/// - Image names containing the HSM group name — fragile, but a
///   last-resort match because the name is free-form.
pub async fn get_images_and_details(
  client: &crate::ShastaClient,
  shasta_token: &str,
  hsm_group_name_vec: &[String],
  id_opt: Option<&str>,
  limit_number: Option<&u8>,
) -> Result<Vec<(Image, String, String, bool)>, Error> {
  let mut image_vec: Vec<Image> =
    client.ims_image_get(shasta_token, id_opt).await?;

  image::utils::get_image_cfs_config_name_hsm_group_name(
    shasta_token,
    client.base_url(),
    client.root_cert(),
    client.socks5_proxy(),
    &mut image_vec,
    hsm_group_name_vec,
    limit_number,
  )
  .await
  .map_err(|e| {
    Error::Message(format!("ERROR - Failed to get image details: {}", e))
  })
}
