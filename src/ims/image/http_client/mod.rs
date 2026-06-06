//! `ShastaClient` methods for `/ims/v3/images`.

pub(crate) mod types;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// IMS image mirror types. Gated behind the `manta-dispatcher`
/// Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;

use serde_json::Value;

use types::{Image, PatchImage};

use crate::{ShastaClient, error::Error};

impl ShastaClient {
  /// `GET /ims/v3/images` (or `/ims/v3/images/{id}` if `image_id_opt`
  /// is supplied) — list IMS images or fetch one by ID.
  pub async fn ims_image_get(
    &self,
    token: &str,
    image_id_opt: Option<&str>,
  ) -> Result<Vec<Image>, Error> {
    log::debug!(
      "Get IMS images '{}'",
      image_id_opt.unwrap_or("all available")
    );

    let api_url = if let Some(image_id) = image_id_opt {
      format!("{}/ims/v3/images/{}", self.base_url(), image_id)
    } else {
      format!("{}/ims/v3/images", self.base_url())
    };

    let response = self
      .http()
      .get(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?
      .error_for_status()
      .map_err(|e| match e.status() {
        Some(reqwest::StatusCode::NOT_FOUND) => Error::ImageNotFound(
          image_id_opt.map(str::to_string).unwrap_or_default(),
        ),
        Some(_) => Error::NetError(e),
        None => Error::Message(format!(
          "ERROR - Http response with no status code?.\nReason:\n{}",
          e
        )),
      })?;

    let image_vec: Vec<Image> = if image_id_opt.is_none() {
      response
        .json::<Vec<Image>>()
        .await
        .map_err(Error::NetError)?
    } else {
      vec![response.json::<Image>().await.map_err(Error::NetError)?]
    };

    Ok(image_vec)
  }

  /// `GET /ims/v3/images` — every IMS image.
  pub async fn ims_image_get_all(
    &self,
    token: &str,
  ) -> Result<Vec<Image>, Error> {
    self.ims_image_get(token, None).await
  }

  /// Register a new image in IMS.
  pub async fn ims_image_post(
    &self,
    token: &str,
    ims_image: &Image,
  ) -> Result<Value, Error> {
    let api_url = format!("{}/ims/v3/images", self.base_url());

    self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&ims_image)
      .send()
      .await
      .map_err(Error::NetError)?
      .error_for_status()
      .map_err(Error::NetError)?
      .json()
      .await
      .map_err(Error::NetError)
  }

  /// Delete an IMS image (soft delete + permanent deletion in sequence).
  pub async fn ims_image_delete(
    &self,
    token: &str,
    image_id: &str,
  ) -> Result<(), Error> {
    let map_delete_err = |e: reqwest::Error| match e.status() {
      Some(reqwest::StatusCode::NOT_FOUND) => {
        Error::ImageNotFound(image_id.to_string())
      }
      Some(_) => Error::NetError(e),
      None => Error::Message(format!(
        "ERROR - Http response with no status code?.\nReason:\n{}",
        e
      )),
    };

    // SOFT DELETION
    let api_url = format!("{}/ims/v3/images/{}", self.base_url(), image_id);
    self
      .http()
      .delete(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?
      .error_for_status()
      .map(|_| ())
      .map_err(map_delete_err)?;

    // PERMANENT DELETION
    let api_url =
      format!("{}/ims/v3/deleted/images/{}", self.base_url(), image_id);
    self
      .http()
      .delete(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?
      .error_for_status()
      .map(|_| ())
      .map_err(map_delete_err)
  }

  /// Patch an IMS image record (link to S3 manifest, metadata, etc).
  pub async fn ims_image_patch(
    &self,
    token: &str,
    ims_image_id: &str,
    ims_link: &PatchImage,
  ) -> Result<(), Error> {
    let api_url = format!("{}/ims/v3/images/{}", self.base_url(), ims_image_id);

    self
      .http()
      .patch(api_url)
      .bearer_auth(token)
      .json(&ims_link)
      .send()
      .await
      .map_err(Error::NetError)?
      .error_for_status()
      .map_err(Error::NetError)?
      .json::<Value>()
      .await
      .map_err(Error::NetError)?;

    Ok(())
  }
}
