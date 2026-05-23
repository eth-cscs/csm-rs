use serde_json::Value;

use crate::{common::http, error::Error};

use super::types::{HWInventoryByLocationList, NodeSummary};

pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
) -> Result<NodeSummary, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!("{}/smd/hsm/v2/Inventory/Hardware", shasta_base_url);

  let payload: Value =
    http::handle_json_or_text_response(
      client
        .get(api_url)
        .bearer_auth(shasta_token)
        .send()
        .await
        .map_err(Error::NetError)?,
    )
    .await?;

  match payload.pointer("/Nodes/0") {
    Some(node_value) => Ok(NodeSummary::from_csm_value(node_value.clone())),
    None => Err(Error::Message(format!(
      "ERROR - json section '/Node' missing in json response API for node '{}'",
      xname
    ))),
  }
}

pub async fn get_query(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
) -> Result<Value, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!(
    "{}/smd/hsm/v2/Inventory/Hardware/Query/{}",
    shasta_base_url, xname
  );
  http::get_json(&client, &api_url, shasta_token).await
}

pub async fn post(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  hw_inventory_by_location: HWInventoryByLocationList,
) -> Result<Value, Error> {
  let client = http::build_client(root_cert, socks5_proxy)?;
  let api_url = format!("{}/smd/hsm/v2/Inventory/Hardware", base_url);

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
        return Err(Error::RequestError {
          response: e,
          payload: error_payload,
        });
      }
      _ => {
        let error_payload = response.json::<Value>().await?;
        return Err(Error::CsmError(error_payload));
      }
    }
  }

  response.json().await.map_err(Error::NetError)
}
