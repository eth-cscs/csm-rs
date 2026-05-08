use serde_json::Value;

use crate::error::Error;

use super::types::{HWInventoryByLocationList, NodeSummary};

pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
) -> Result<NodeSummary, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url = format!("{}/smd/hsm/v2/Inventory/Hardware", shasta_base_url);

  let response = client
    .get(api_url)
    .header("Authorization", format!("Bearer {}", shasta_token))
    .send()
    .await
    .map_err(|error| Error::NetError(error))?;

  if response.status().is_success() {
    let payload = response
      .json::<Value>()
      .await
      .map_err(|error| Error::NetError(error));

    match payload.unwrap().pointer("/Nodes/0") {
      Some(node_value) => Ok(NodeSummary::from_csm_value(node_value.clone())),
      None => Err(Error::Message(format!(
        "ERROR - json section '/Node' missing in json response API for node '{}'",
        xname
      ))),
    }
  } else {
    let e = response
      .text()
      .await
      .map_err(|error| Error::NetError(error))?;

    Err(Error::Message(e.to_string()))
  }
}

pub async fn get_query(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
) -> Result<Value, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url = format!(
    "{}/smd/hsm/v2/Inventory/Hardware/Query/{}",
    shasta_base_url, xname
  );

  let response = client
    .get(api_url)
    .header("Authorization", format!("Bearer {}", shasta_token))
    .send()
    .await
    .map_err(|error| Error::NetError(error))?;

  if response.status().is_success() {
    response
      .json()
      .await
      .map_err(|error| Error::NetError(error))
  } else {
    let payload = response
      .json::<Value>()
      .await
      .map_err(|error| Error::NetError(error))?;

    Err(Error::CsmError(payload))
  }
}

pub async fn post(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  hw_inventory_by_location: HWInventoryByLocationList,
) -> Result<Value, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url: String =
    format!("{}/{}", base_url, "/smd/hsm/v2/Inventory/Hardware");

  let response = client
    .post(api_url)
    .bearer_auth(auth_token)
    .json(&hw_inventory_by_location)
    .send()
    .await?;

  if let Err(e) = response.error_for_status_ref() {
    match response.status() {
      reqwest::StatusCode::UNAUTHORIZED => {
        let error_payload = response.text().await?;
        let error = Error::RequestError {
          response: e,
          payload: error_payload,
        };
        return Err(error);
      }
      _ => {
        let error_payload = response.json::<Value>().await?;
        let error = Error::CsmError(error_payload);
        return Err(error);
      }
    }
  }

  response
    .json()
    .await
    .map_err(|error| Error::NetError(error))
}
