//! `ShastaClient` methods for `/smd/hsm/v2/Inventory/Hardware`.

use serde_json::Value;

use crate::{
  ShastaClient, common::http, error::Error, hsm::types::HsmActionResponse,
};

use super::types::{HWInventory, HWInventoryByLocationList, NodeSummary};

impl ShastaClient {
  /// `GET /hsm/v2/Inventory/Hardware` — fetch the hardware inventory
  /// for a single node, summarised as a [`NodeSummary`].
  pub async fn hsm_hw_inventory_get(
    &self,
    token: &str,
    xname: &str,
  ) -> Result<NodeSummary, Error> {
    let api_url = format!("{}/smd/hsm/v2/Inventory/Hardware", self.base_url());

    let payload: Value = http::handle_json_or_text_response(
      self
        .http()
        .get(api_url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(Error::NetError)?,
    )
    .await?;

    match payload.pointer("/Nodes/0") {
      Some(node_value) => NodeSummary::try_from_csm_value(node_value),
      None => Err(Error::Message(format!(
        "ERROR - json section '/Node' missing in json response API for node '{}'",
        xname
      ))),
    }
  }

  /// `GET /hsm/v2/Inventory/Hardware/Query/{xname}` — typed HSM
  /// hardware inventory query for one xname.
  pub async fn hsm_hw_inventory_get_query(
    &self,
    token: &str,
    xname: &str,
  ) -> Result<HWInventory, Error> {
    let api_url = format!(
      "{}/smd/hsm/v2/Inventory/Hardware/Query/{}",
      self.base_url(),
      xname
    );
    http::get_json(self.http(), &api_url, token).await
  }

  /// `POST /hsm/v2/Inventory/Hardware` — submit a hardware inventory
  /// payload (typically used by node discovery agents). Returns the
  /// HSM acknowledgement carrying a count of new/modified items.
  pub async fn hsm_hw_inventory_post(
    &self,
    token: &str,
    hw_inventory_by_location: HWInventoryByLocationList,
  ) -> Result<HsmActionResponse, Error> {
    let api_url = format!("{}/smd/hsm/v2/Inventory/Hardware", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&hw_inventory_by_location)
      .send()
      .await
      .map_err(Error::NetError)?;
    http::handle_json_response(response, "POST").await
  }
}
