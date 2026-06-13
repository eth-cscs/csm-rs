//! Wrapper for `/Inventory/EthernetInterfaces`. Replaces
//! `src/hsm/hw_inventory/ethernet_interfaces/http_client.rs`.
//!
//! **All four methods stay on raw `reqwest`.** Routing through the
//! generated client would change either the on-wire URL or the public
//! return type — neither is acceptable without a separate breaking-change
//! PR. Per-method rationale:
//!
//! - `hsm_eth_post` and `hsm_eth_post_ip_addresses` hit
//!   `{base}/hsm/v2/Inventory/EthernetInterfaces[/.../IPAddresses]` —
//!   note the missing `smd` segment. The generated client's basePath in
//!   `wrapper::gen_client` is `{base}/smd/hsm/v2`, so routing through it
//!   would silently rewrite the path to `{base}/smd/hsm/v2/...`. This
//!   `/hsm/v2/` vs `/smd/hsm/v2/` split is a long-standing csm-rs quirk
//!   preserved verbatim across the migration (matching the redfish_endpoint
//!   migration in Task 10).
//! - `hsm_eth_get` and `hsm_eth_patch` both return `reqwest::Response`
//!   (the body is handed back to the caller raw). The generated
//!   `do_comp_eth_interfaces_get_v2` returns `Vec<CompEthInterface100>`
//!   and `do_comp_eth_interface_patch_v2` returns `()`. Routing through
//!   either would change the public return type to a typed payload,
//!   which is a public-API break we are explicitly avoiding here.
//!
//! BEHAVIOUR DELTA (from Task 11): the hand-written `EthernetInterface`
//! and (to a lesser extent) `IpAddressMapping` / `ComponentEthernetInterface`
//! types in `super::super::hw_inventory::ethernet_interfaces::types` did
//! not carry the PascalCase `#[serde(rename = "...")]` annotations the
//! spec requires, and `EthernetInterface` had a singular
//! `ip_address: Option<String>` instead of the spec's
//! `IPAddresses: array<IPAddressMapping>`. Those types were fixed
//! in-place in Task 11; this wrapper accepts/returns the corrected
//! shapes. Callers that previously serialised the broken snake_case
//! variants will see their JSON change to spec-conformant PascalCase.

use crate::{
  ShastaClient, error::Error,
  hsm::hw_inventory::ethernet_interfaces::types::EthernetInterface,
};

use super::super::hw_inventory::ethernet_interfaces::types::{
  ComponentEthernetInterface, IpAddressMapping,
};

impl ShastaClient {
  /// `POST /hsm/v2/Inventory/EthernetInterfaces` — register a new
  /// ethernet interface for a component.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_eth_post(
    &self,
    token: &str,
    eht_interface: ComponentEthernetInterface,
  ) -> Result<(), Error> {
    let api_url =
      format!("{}/hsm/v2/Inventory/EthernetInterfaces", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&eht_interface)
      .send()
      .await?;

    if let Err(e) = response.error_for_status_ref() {
      if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        let url = response.url().to_string();
        let error_payload = response.text().await?;
        return Err(Error::RequestError {
          response: e,
          url,
          payload: error_payload,
        });
      } else {
        let error_payload = response.text().await?;
        return Err(Error::Message(error_payload));
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  /// `POST /hsm/v2/Inventory/EthernetInterfaces/{component_id}/IPAddresses`
  /// — add IP address mappings to an existing component's interface.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_eth_post_ip_addresses(
    &self,
    token: &str,
    eht_interface: ComponentEthernetInterface,
  ) -> Result<EthernetInterface, Error> {
    let component_id =
      eht_interface.component_id.as_ref().ok_or_else(|| {
        Error::Message(
          "ComponentEthernetInterface is missing 'component_id'".to_string(),
        )
      })?;
    let api_url = format!(
      "{}/hsm/v2/Inventory/EthernetInterfaces/{}/IPAddresses",
      self.base_url(),
      component_id
    );

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&eht_interface)
      .send()
      .await?;

    if let Err(e) = response.error_for_status_ref() {
      if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        let url = response.url().to_string();
        let error_payload = response.text().await?;
        return Err(Error::RequestError {
          response: e,
          url,
          payload: error_payload,
        });
      } else {
        let error_payload = response.text().await?;
        return Err(Error::Message(error_payload));
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  /// Get list of network interfaces. Ref: <https://csm12-apidocs.svc.cscs.ch/iaas/hardware-state-manager/operation/doCompEthInterfacesGetV2/>.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  #[allow(clippy::too_many_arguments)]
  pub async fn hsm_eth_get(
    &self,
    token: &str,
    mac_address: &str,
    ip_address: &str,
    network: &str,
    component_id: &str, // Node's xname
    r#type: &str,
    olther_than: &str,
    newer_than: &str,
  ) -> Result<reqwest::Response, Error> {
    let api_url = format!(
      "{}/smd/hsm/v2/Inventory/EthernetInterfaces",
      self.base_url()
    );

    self
      .http()
      .get(api_url)
      .query(&[
        ("MACAddress", mac_address),
        ("IPAddress", ip_address),
        ("Network", network),
        ("ComponentID", component_id),
        ("Type", r#type),
        ("OlderThan", olther_than),
        ("NewerThan", newer_than),
      ])
      .bearer_auth(token)
      .send()
      .await?
      .error_for_status()
      .map_err(Error::NetError)
  }

  /// `PATCH /hsm/v2/Inventory/EthernetInterfaces/{id}` — update the
  /// description, owning component, or IP/network mapping of an
  /// existing ethernet interface.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_eth_patch(
    &self,
    token: &str,
    eth_interface_id: &str,
    description: Option<&str>,
    component_id: &str,
    ip_address_mapping: (&str, &str), // [(<ip address>, <network>), ...]
  ) -> Result<reqwest::Response, Error> {
    let ip_address = ip_address_mapping.0;
    let network = ip_address_mapping.1;
    let cei = ComponentEthernetInterface {
      description: description.map(str::to_string),
      ip_addresses: vec![IpAddressMapping {
        ip_address: ip_address.to_string(),
        network: Some(network.to_string()),
      }],
      component_id: Some(component_id.to_string()),
    };

    let api_url = format!(
      "{}/smd/hsm/v2/Inventory/EthernetInterfaces/{}",
      self.base_url(),
      eth_interface_id
    );

    self
      .http()
      .patch(api_url)
      .query(&[("ethInterfaceID", ip_address), ("ipAddress", ip_address)])
      .bearer_auth(token)
      .json(&cei)
      .send()
      .await
      .map_err(Error::NetError)?
      .error_for_status()
      .map_err(Error::NetError)
  }
}
