//! `ShastaClient` methods for `/smd/hsm/v2/Inventory/EthernetInterfaces`.

use crate::{
  ShastaClient, error::Error,
  hsm::hw_inventory::ethernet_interfaces::types::EthernetInterface,
};

use super::types::{ComponentEthernetInterface, IpAddressMapping};

impl ShastaClient {
  /// `POST /hsm/v2/Inventory/EthernetInterfaces` — register a new
  /// ethernet interface for a component.
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
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          let error_payload = response.text().await?;
          return Err(Error::RequestError {
            response: e,
            payload: error_payload,
          });
        }
        _ => {
          let error_payload = response.text().await?;
          dbg!(&error_payload);
          return Err(Error::Message(error_payload));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  /// `POST /hsm/v2/Inventory/EthernetInterfaces/{component_id}/IPAddresses`
  /// — add IP address mappings to an existing component's interface.
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
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          let error_payload = response.text().await?;
          return Err(Error::RequestError {
            response: e,
            payload: error_payload,
          });
        }
        _ => {
          let error_payload = response.text().await?;
          return Err(Error::Message(error_payload));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  /// Get list of network interfaces. Ref: <https://csm12-apidocs.svc.cscs.ch/iaas/hardware-state-manager/operation/doCompEthInterfacesGetV2/>.
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
