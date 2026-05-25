//! `ShastaClient` methods for `/smd/hsm/v2/Inventory/Hardware`.

use serde_json::Value;

use crate::{ShastaClient, common::http, error::Error};

use super::types::{HWInventoryByLocationList, NodeSummary};

impl ShastaClient {
  pub async fn hsm_hw_inventory_get(
    &self,
    xname: &str,
  ) -> Result<NodeSummary, Error> {
    let api_url = format!("{}/smd/hsm/v2/Inventory/Hardware", self.base_url());

    let payload: Value = http::handle_json_or_text_response(
      self
        .http()
        .get(api_url)
        .bearer_auth(self.token())
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

  pub async fn hsm_hw_inventory_get_query(
    &self,
    xname: &str,
  ) -> Result<Value, Error> {
    let api_url = format!(
      "{}/smd/hsm/v2/Inventory/Hardware/Query/{}",
      self.base_url(),
      xname
    );
    http::get_json(self.http(), &api_url, self.token()).await
  }

  pub async fn hsm_hw_inventory_post(
    &self,
    hw_inventory_by_location: HWInventoryByLocationList,
  ) -> Result<Value, Error> {
    let api_url = format!("{}/smd/hsm/v2/Inventory/Hardware", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(self.token())
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
}
