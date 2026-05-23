pub mod types;

use serde_json::Value;

use types::{Image, PatchImage};

use crate::{common::http, error::Error};

pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  image_id_opt: Option<&str>,
) -> Result<Vec<Image>, Error> {
  log::info!(
    "Get IMS images '{}'",
    image_id_opt.unwrap_or("all available")
  );

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;

  let api_url = if let Some(image_id) = image_id_opt {
    format!("{}/ims/v3/images/{}", shasta_base_url, image_id)
  } else {
    format!("{}/ims/v3/images", shasta_base_url)
  };

  let response_rslt = client
    .get(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?
    .error_for_status()
    .map_err(|e| match e.status() {
      Some(reqwest::StatusCode::NOT_FOUND) => {
        Error::ImageNotFound(image_id_opt.map(str::to_string).unwrap())
      }
      Some(_) => Error::NetError(e),
      None => Error::Message(format!(
        "ERROR - Http response with no status code?.\nReason:\n{}",
        e
      )),
    });

  let image_vec: Vec<Image> = match response_rslt {
    Ok(response) => {
      if image_id_opt.is_none() {
        response.json::<Vec<Image>>().await.unwrap()
      } else {
        vec![response.json::<Image>().await.unwrap()]
      }
    }
    Err(error) => return Err(error),
  };

  Ok(image_vec)
}

pub async fn get_all(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Vec<Image>, Error> {
  get(shasta_token, shasta_base_url, shasta_root_cert, socks5_proxy, None).await
}

/// Register a new image in IMS --> https://github.com/Cray-HPE/docs-csm/blob/release/1.5/api/ims.md#post_v2_image
pub async fn post(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  ims_image: &Image,
) -> Result<Value, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!("{}/ims/v3/images", shasta_base_url);

  client
    .post(api_url)
    .bearer_auth(shasta_token)
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

// Delete IMS image using CSM API. First does a "soft delete", then a "permanent deletion"
// soft delete --> https://csm12-apidocs.svc.cscs.ch/paas/ims/operation/delete_v3_image/
// permanent deletion --> https://csm12-apidocs.svc.cscs.ch/paas/ims/operation/delete_v3_deleted_image/
pub async fn delete(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  image_id: &str,
) -> Result<(), Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;

  // Map a NOT_FOUND status to the dedicated ImageNotFound variant so callers
  // can distinguish "the image is already gone" from other failures.
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
  let api_url = format!("{}/ims/v3/images/{}", shasta_base_url, image_id);
  client
    .delete(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?
    .error_for_status()
    .map(|_| ())
    .map_err(map_delete_err)?;

  // PERMANENT DELETION
  let api_url =
    format!("{}/ims/v3/deleted/images/{}", shasta_base_url, image_id);
  client
    .delete(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?
    .error_for_status()
    .map(|_| ())
    .map_err(map_delete_err)
}

/// update an IMS image record --> https://github.com/Cray-HPE/docs-csm/blob/release/1.5/api/ims.md#post_v2_image
pub async fn patch(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  ims_image_id: &str,
  ims_link: &PatchImage,
) -> Result<(), Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!("{}/ims/v3/images/{}", shasta_base_url, ims_image_id);

  client
    .patch(api_url)
    .bearer_auth(shasta_token)
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
