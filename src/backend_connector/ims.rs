//! `ImsTrait`, `GetImagesAndDetailsTrait` impls for [`Csm`](super::Csm).

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
      .shasta_client()
      .ims_image_get(shasta_token, image_id_opt)
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
      .shasta_client()
      .ims_image_get_all(shasta_token)
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
      .shasta_client()
      .ims_image_patch(shasta_token, image_id, &image.clone().into())
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
      .shasta_client()
      .ims_image_delete(shasta_token, image_id)
      .await
      .map_err(|e| Error::Message(e.to_string()))
  }
}

/// Backend-dispatcher impl of `GetImagesAndDetailsTrait` for [`Csm`].
///
/// Delegates to
/// [`crate::commands::get_images_and_details::get_images_and_details`];
/// see that function for the matching strategy.
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
