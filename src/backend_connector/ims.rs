use manta_backend_dispatcher::{
  error::Error,
  interfaces::{ims::GetImagesAndDetailsTrait, ims::ImsTrait},
  types::ims::{Image as FrontEndImage, PatchImage},
};

use super::Csm;

impl ImsTrait for Csm {
  async fn get_images(
    &self,
    shasta_token: &str,
    image_id_opt: Option<&str>,
  ) -> Result<Vec<FrontEndImage>, Error> {
    self
      .shasta_client(shasta_token)?
      .ims_image_get(image_id_opt)
      .await
      .map(|v| v.into_iter().map(Into::into).collect())
      .map_err(|e| Error::Message(e.to_string()))
  }

  async fn get_all_images(
    &self,
    shasta_token: &str,
    _shasta_base_url: &str,
    _shasta_root_cert: &[u8],
  ) -> Result<Vec<FrontEndImage>, Error> {
    self
      .shasta_client(shasta_token)?
      .ims_image_get_all()
      .await
      .map(|v| v.into_iter().map(Into::into).collect())
      .map_err(|e| Error::Message(e.to_string()))
  }

  fn filter_images(
    &self,
    image_vec: &mut Vec<FrontEndImage>,
  ) -> Result<(), Error> {
    let mut image_aux_vec: Vec<crate::ims::image::http_client::types::Image> =
      image_vec.iter().map(|image| image.clone().into()).collect();

    crate::ims::image::utils::filter(&mut image_aux_vec);

    Ok(())
  }

  async fn update_image(
    &self,
    shasta_token: &str,
    image_id: &str,
    image: &PatchImage,
  ) -> Result<(), Error> {
    let _ = self
      .shasta_client(shasta_token)?
      .ims_image_patch(image_id, &image.clone().into())
      .await
      .map_err(|e| Error::Message(e.to_string()));

    Ok(())
  }

  async fn delete_image(
    &self,
    shasta_token: &str,
    _shasta_base_url: &str,
    _shasta_root_cert: &[u8],
    image_id: &str,
  ) -> Result<(), Error> {
    self
      .shasta_client(shasta_token)?
      .ims_image_delete(image_id)
      .await
      .map_err(|e| Error::Message(e.to_string()))
  }
}

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
impl GetImagesAndDetailsTrait for Csm {
  async fn get_images_and_details(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec: &[String],
    id_opt: Option<&str>,
    limit_number: Option<&u8>,
  ) -> Result<Vec<(FrontEndImage, String, String, bool)>, Error> {
    crate::commands::get_images_and_details::get_images_and_details(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      self.socks5_proxy.as_deref(),
      hsm_group_name_vec,
      id_opt,
      limit_number,
    )
    .await
    .map(|image_details_vec| {
      image_details_vec
        .into_iter()
        .map(|(image, x, y, z)| (image.into(), x, y, z))
        .collect()
    })
    .map_err(|e| Error::Message(e.to_string()))
  }
}
