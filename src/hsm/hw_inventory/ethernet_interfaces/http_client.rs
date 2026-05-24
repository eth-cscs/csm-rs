use crate::{
  common::http,
  error::Error,
  hsm::hw_inventory::ethernet_interfaces::types::EthernetInterface,
};

use super::types::{ComponentEthernetInterface, IpAddressMapping};

pub async fn post(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  eht_interface: ComponentEthernetInterface,
) -> Result<(), Error> {
  let client = http::build_client(root_cert, socks5_proxy)?;
  let api_url = format!("{}/hsm/v2/Inventory/EthernetInterfaces", base_url);

  let response = client
    .post(api_url)
    .bearer_auth(auth_token)
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

pub async fn post_ip_addresses(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  eht_interface: ComponentEthernetInterface,
) -> Result<EthernetInterface, Error> {
  let client = http::build_client(root_cert, socks5_proxy)?;
  let component_id = eht_interface.component_id.as_ref().ok_or_else(|| {
    Error::Message(
      "ComponentEthernetInterface is missing 'component_id'".to_string(),
    )
  })?;
  let api_url = format!(
    "{}/hsm/v2/Inventory/EthernetInterfaces/{}/IPAddresses",
    base_url, component_id
  );

  let response = client
    .post(api_url)
    .bearer_auth(auth_token)
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

// Get list of network interfaces
// ref --> https://csm12-apidocs.svc.cscs.ch/iaas/hardware-state-manager/operation/doCompEthInterfacesGetV2/
pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  mac_address: &str,
  ip_address: &str,
  network: &str,
  component_id: &str, // Node's xname
  r#type: &str,
  olther_than: &str,
  newer_than: &str,
) -> Result<reqwest::Response, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!(
    "{}/smd/hsm/v2/Inventory/EthernetInterfaces",
    shasta_base_url
  );

  client
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
    .bearer_auth(shasta_token)
    .send()
    .await?
    .error_for_status()
    .map_err(Error::NetError)
}

pub async fn patch(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  eth_interface_id: &str,
  description: Option<&str>,
  component_id: &str,
  ip_address_mapping: (&str, &str), // [(<ip address>, <network>), ...], examle
                                    // [("192.168.1.10", "HMN"), ...]
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

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!(
    "{}/smd/hsm/v2/Inventory/EthernetInterfaces/{}",
    shasta_base_url, eth_interface_id
  );

  client
    .patch(api_url)
    .query(&[("ethInterfaceID", ip_address), ("ipAddress", ip_address)])
    .bearer_auth(shasta_token)
    .json(&cei)
    .send()
    .await
    .map_err(Error::NetError)?
    .error_for_status()
    .map_err(Error::NetError)
}
