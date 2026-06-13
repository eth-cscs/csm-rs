//! Wrapper for `/Inventory/Hardware`. Replaces
//! `src/hsm/hw_inventory/hw_component/http_client.rs`.
//!
//! `hsm_hw_inventory_get` returns a projection ([`NodeSummary`]) built
//! from the `/Nodes/0` slice of the raw response. The projection types
//! ([`NodeSummary`], [`ArtifactSummary`], [`ArtifactType`]) live in
//! [`crate::hsm::wrapper::hw_component_types`] per the design decision
//! that `types.rs` files hold wire-shape mirrors only and hand-rolled
//! projections belong with the wrapper that creates them.
//! The historical public path
//! `csm_rs::hsm::hw_inventory::hw_component::NodeSummary` stays valid
//! via re-exports through `src/hsm/hw_inventory/hw_component/mod.rs`.
//!
//! **All three methods stay on raw `reqwest`.** Concrete reasons per
//! method:
//!
//! - `hsm_hw_inventory_get` reads `/Nodes/0` from the GET
//!   `/Inventory/Hardware` response. The generated
//!   `do_hw_inv_by_location_get_all` returns
//!   `Vec<HwInventory100HwInventoryByLocation>` — a flat list of
//!   *location* records (cabinets, chassis, processors, …), not the
//!   per-component-collection shape (`Nodes`, `Cabinets`, …) the
//!   projection needs. Routing through progenitor would require
//!   reshaping the typed Vec back into the `/Nodes/0` JSON
//!   pointer-walk the projection already handles correctly — a wash
//!   that loses the diagnostic from
//!   `Error::HsmInventoryShape("json section '/Nodes/0' missing ...")`.
//! - `hsm_hw_inventory_get_query` hits
//!   `/Inventory/Hardware/Query/{xname}` and returns the local
//!   `HWInventory` tagged-enum tree. The generated
//!   `do_hw_inv_by_location_query_get` returns
//!   `HwInventory100HwInventory`, a structurally different shape: the
//!   csm-rs `HWInventoryByLocation` is a Rust `#[serde(tag = "...")]`
//!   enum (so a single `HWInventory.nodes` field deserialises against
//!   the variant), while the generated type is a flat collection of
//!   `Option<Vec<HwInvByLocXxx>>` per category. Both deserialise the
//!   same JSON, but the public type returned to callers
//!   (`HWInventory`) is the local hand-written one and the dispatcher
//!   bridge ([`dispatcher_conv`]) is built on that shape; switching
//!   the public return type to the generated struct is out of scope
//!   for this task and would force a parallel rewrite of the
//!   865-line `dispatcher_conv.rs`.
//! - `hsm_hw_inventory_post` POSTs an `HWInventoryByLocationList`
//!   (also the hand-written tagged-enum shape) and returns
//!   `HsmActionResponse` whose `code`/`message` fields are
//!   `#[serde(default)]` to tolerate `{}` bodies from CSM mocks. The
//!   generated `do_hw_inv_by_location_post` takes
//!   `DoHwInvByLocationPostBody` and returns `Response100`, neither of
//!   which round-trip with the existing public/dispatcher shape (see
//!   the `hsm_hw_inventory_get_query` rationale above).
//!
//! [`dispatcher_conv`]: super::super::hw_inventory::hw_component::dispatcher_conv
//! [`NodeSummary`]: super::hw_component_types::NodeSummary
//! [`ArtifactSummary`]: super::hw_component_types::ArtifactSummary
//! [`ArtifactType`]: super::hw_component_types::ArtifactType

use serde_json::Value;

use crate::{
  ShastaClient,
  common::http,
  error::Error,
  hsm::{
    hw_inventory::hw_component::types::{HWInventory, HWInventoryByLocationList},
    types::HsmActionResponse,
    wrapper::hw_component_types::NodeSummary,
  },
};

impl ShastaClient {
  /// `GET /hsm/v2/Inventory/Hardware` — fetch the hardware inventory
  /// for a single node, summarised as a [`NodeSummary`].
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
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
      None => Err(Error::HsmInventoryShape(format!(
        "json section '/Nodes/0' missing in response for xname '{xname}'"
      ))),
    }
  }

  /// `GET /hsm/v2/Inventory/Hardware/Query/{xname}` — typed HSM
  /// hardware inventory query for one xname.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
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
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
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
