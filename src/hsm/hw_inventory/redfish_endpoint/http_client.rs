use serde_json::Value;

use crate::{common::http, error::Error};

use super::types::{RedfishEndpoint, RedfishEndpointArray};

pub async fn get_query(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
) -> Result<RedfishEndpointArray, Error> {
  let client = http::build_client(root_cert, socks5_proxy)?;
  let api_url = format!(
    "{}/smd/hsm/v2/Inventory/RedfishEndpoint/Query/{}",
    base_url, xname
  );

  let response = client
    .get(api_url)
    .query(&[xname])
    .bearer_auth(auth_token)
    .send()
    .await?;

  http::handle_json_or_request_error(response).await
}

pub async fn get(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  id: Option<&str>,
  fqdn: Option<&str>,
  r#type: Option<&str>,
  uuid: Option<&str>,
  macaddr: Option<&str>,
  ip_address: Option<&str>,
  last_status: Option<&str>,
) -> Result<RedfishEndpointArray, Error> {
  let client = http::build_client(root_cert, socks5_proxy)?;
  let api_url =
    format!("{}/smd/hsm/v2/Inventory/RedfishEndpoints", base_url);

  let response = client
    .get(api_url)
    .query(&[id, fqdn, r#type, uuid, macaddr, ip_address, last_status])
    .bearer_auth(auth_token)
    .send()
    .await?;

  http::handle_json_or_request_error(response).await
}

pub async fn get_one(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
) -> Result<RedfishEndpoint, Error> {
  let client = http::build_client(root_cert, socks5_proxy)?;
  let api_url =
    format!("{}/smd/hsm/v2/Inventory/RedfishEndpoints/{}", base_url, xname);

  let response = client.get(api_url).bearer_auth(auth_token).send().await?;
  http::handle_json_or_request_error(response).await
}

pub async fn post(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  redfish_endpoint: RedfishEndpoint,
) -> Result<Value, Error> {
  let client = http::build_client(root_cert, socks5_proxy)?;
  let api_url =
    format!("{}/smd/hsm/v2/Inventory/RedfishEndpoints", base_url);

  let response = client
    .post(api_url)
    .bearer_auth(auth_token)
    .json(&redfish_endpoint)
    .send()
    .await?;

  http::handle_json_or_request_error(response).await
}

pub async fn put(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
  redfish_endpoint: RedfishEndpoint,
) -> Result<RedfishEndpoint, Error> {
  let client = http::build_client(root_cert, socks5_proxy)?;
  let api_url =
    format!("{}/smd/hsm/v2/State/Components/{}", base_url, xname);

  let response = client
    .put(api_url)
    .bearer_auth(auth_token)
    .json(&redfish_endpoint)
    .send()
    .await?;

  http::handle_json_or_request_error(response).await
}

pub async fn delete_all(
  base_url: &str,
  auth_token: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Value, Error> {
  let client = http::build_client(root_cert, socks5_proxy)?;
  let api_url =
    format!("{}/smd/hsm/v2/Inventory/RedfishEndpoints", base_url);

  let response = client
    .delete(api_url)
    .bearer_auth(auth_token)
    .send()
    .await?;

  http::handle_json_or_request_error(response).await
}

pub async fn delete_one(
  base_url: &str,
  auth_token: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
) -> Result<Value, Error> {
  let client = http::build_client(root_cert, socks5_proxy)?;
  let api_url =
    format!("{}/smd/hsm/v2/Inventory/RedfishEndpoints/{}", base_url, xname);

  let response = client
    .delete(api_url)
    .bearer_auth(auth_token)
    .send()
    .await?;

  http::handle_json_or_request_error(response).await
}
