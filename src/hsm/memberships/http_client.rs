use crate::{common::http, error::Error};

use super::types::Membership;

pub async fn get_all(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Vec<Membership>, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url = format!("{}/smd/hsm/v2/memberships", shasta_base_url);
  http::get_json(&client, &url, shasta_token).await
}

pub async fn get_xname(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
) -> Result<Membership, Error> {
  log::debug!("Get membership of node '{}'", xname);
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url = format!("{}/smd/hsm/v2/memberships/{}", shasta_base_url, xname);
  http::get_json(&client, &url, shasta_token).await
}
