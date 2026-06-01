//! `ShastaClient` methods for `/smd/hsm/v2/Inventory/Hardware`.

use serde_json::Value;

use crate::{
  ShastaClient, common::http, error::Error, hsm::types::HsmActionResponse,
};

use super::types::{HWInventoryByLocationList, NodeSummary};

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
      Some(node_value) => Ok(NodeSummary::from_csm_value(node_value.clone())),
      None => Err(Error::Message(format!(
        "ERROR - json section '/Node' missing in json response API for node '{}'",
        xname
      ))),
    }
  }

  /// `GET /hsm/v2/Inventory/Hardware/Query/{xname}` — raw HSM hardware
  /// inventory query for one xname.
  ///
  /// FIXME: kept as `Value` because csm-rs's
  /// [`super::types::HWInventory`] uses serialize-only serde renames
  /// (`#[serde(rename(serialize = "X"))]`), which prevents direct
  /// deserialization from CSM's PascalCase JSON. The
  /// `backend_connector` impl deserializes into the dispatcher's
  /// bidirectionally-renamed `HWInventory` via `serde_json::from_value`.
  /// Typing this method cleanly requires either fixing the csm-rs
  /// renames or introducing a `#[serde(rename_all = "PascalCase")]`-
  /// based view type.
  pub async fn hsm_hw_inventory_get_query(
    &self,
    token: &str,
    xname: &str,
  ) -> Result<Value, Error> {
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
    http::handle_json_response(response).await
  }
}
