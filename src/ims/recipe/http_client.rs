use crate::{common::http, error::Error};

use super::types::RecipeGetResponse;

/// Create IMS job ref --> https://csm12-apidocs.svc.cscs.ch/paas/ims/operation/post_v3_job/
pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  recipe_id_opt: Option<&str>,
) -> Result<Vec<RecipeGetResponse>, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;

  let api_url = if let Some(recipe_id) = recipe_id_opt {
    format!("{}/ims/v2/recipes{}", shasta_base_url, recipe_id)
  } else {
    format!("{}/ims/v2/recipes", shasta_base_url)
  };

  let response = client
    .get(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await?
    .error_for_status()?
    .json::<Vec<RecipeGetResponse>>()
    .await?;

  Ok(response)
}
