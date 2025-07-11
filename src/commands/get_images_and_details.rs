use crate::{
  error::Error,
  ims::image::{self, http_client::types::Image},
};

/// Returns a tuple like(Image sruct, cfs configuration name, list of target - either hsm group name
/// or xnames, bool - indicates if image is used to boot a node or not)
/// This method tries to filter by HSM group which means it will make use of:
///  - CFS sessions to find which image id was created against which HSM group
///  - BOS sessiontemplates to find the HSM group related to nodes being rebooted in the past
///  - Image ids in boot params for nodes in HSM groups we are looking for (This is needed to not miss
/// images currenly used which name may not have HSM group we are looking for included not CFS
/// session nor BOS sessiontemplate)
///  - Image names with HSM group name included (This is a bad practice because this is a free text
/// prone to human errors)
pub async fn get_images_and_details(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hsm_group_name_vec: &[String],
  id_opt: Option<&String>,
  limit_number: Option<&u8>,
) -> Result<Vec<(Image, String, String, bool)>, Error> {
  let mut image_vec: Vec<Image> = image::http_client::get(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    id_opt.map(|elem| elem.as_str()),
  )
  .await
  .unwrap();

  let image_detail_vec_rslt: Result<Vec<(Image, String, String, bool)>, Error> =
    image::utils::get_image_cfs_config_name_hsm_group_name(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &mut image_vec,
      hsm_group_name_vec,
      limit_number,
    )
    .await
    .map_err(|e| {
      Error::Message(format!("ERROR - Failed to get image details: {}", e))
    });

  image_detail_vec_rslt
}
