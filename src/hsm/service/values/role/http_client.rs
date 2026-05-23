use crate::{common::http, error::Error};

use super::types::Role;

/// Get list of Roles
pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Vec<String>, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!("{}/smd/hsm/v2/service/values/role", shasta_base_url);

  let payload = client
    .get(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?
    .json::<Role>()
    .await;

  payload.map(|role| role.role).map_err(Error::NetError)
}
