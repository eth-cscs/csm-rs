use crate::{
  error::Error,
  hsm::hw_inventory::ethernet_interfaces::types::EthernetInterface,
};

use super::types::{ComponentEthernetInterface, IpAddressMapping};

pub async fn post(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  eht_interface: ComponentEthernetInterface,
) -> Result<(), Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(root_cert)?)
    .use_rustls_tls();

  // Build client
  let client = if let Ok(socks5_env) = std::env::var("SOCKS5") {
    // socks5 proxy
    log::debug!("SOCKS5 enabled");
    let socks5proxy = reqwest::Proxy::all(socks5_env)?;

    // rest client to authenticate
    client_builder.proxy(socks5proxy).build()?
  } else {
    client_builder.build()?
  };

  // let api_url: String =
  //   format!("{}/{}", base_url, "/hsm/v2/Inventory/EthernetInterfaces");
  let api_url: String =
    format!("{}/hsm/v2/Inventory/EthernetInterfaces", base_url);

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
        let error = Error::RequestError {
          response: e,
          payload: error_payload,
        };
        return Err(error);
      }
      _ => {
        let error_payload = response.text().await?;
        dbg!(&error_payload);
        let error = Error::Message(error_payload);
        return Err(error);
      }
    }
  }

  response
    .json()
    .await
    .map_err(|error| Error::NetError(error))
}

pub async fn post_ip_addresses(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  eht_interface: ComponentEthernetInterface,
) -> Result<EthernetInterface, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(root_cert)?)
    .use_rustls_tls();

  // Build client
  let client = if let Ok(socks5_env) = std::env::var("SOCKS5") {
    // socks5 proxy
    log::debug!("SOCKS5 enabled");
    let socks5proxy = reqwest::Proxy::all(socks5_env)?;

    // rest client to authenticate
    client_builder.proxy(socks5proxy).build()?
  } else {
    client_builder.build()?
  };

  let api_url: String = format!(
    "{}/{}/{}/IPAddresses",
    base_url,
    "hsm/v2/Inventory/EthernetInterfaces",
    eht_interface.component_id.as_ref().unwrap()
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
        let error = Error::RequestError {
          response: e,
          payload: error_payload,
        };
        return Err(error);
      }
      _ => {
        let error_payload = response.text().await?;
        let error = Error::Message(error_payload);
        return Err(error);
      }
    }
  }

  response
    .json()
    .await
    .map_err(|error| Error::NetError(error))
}

// Get list of network interfaces
// ref --> https://csm12-apidocs.svc.cscs.ch/iaas/hardware-state-manager/operation/doCompEthInterfacesGetV2/
pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  mac_address: &str,
  ip_address: &str,
  network: &str,
  component_id: &str, // Node's xname
  r#type: &str,
  olther_than: &str,
  newer_than: &str,
) -> Result<reqwest::Response, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  // Build client
  let client = if let Ok(socks5_env) = std::env::var("SOCKS5") {
    // socks5 proxy
    log::debug!("SOCKS5 enabled");
    let socks5proxy = reqwest::Proxy::all(socks5_env)?;

    // rest client to authenticate
    client_builder.proxy(socks5proxy).build()?
  } else {
    client_builder.build()?
  };

  let api_url: String =
    shasta_base_url.to_owned() + "/smd/hsm/v2/Inventory/EthernetInterfaces";

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

  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  // Build client
  let client = if let Ok(socks5_env) = std::env::var("SOCKS5") {
    // socks5 proxy
    log::debug!("SOCKS5 enabled");
    let socks5proxy = reqwest::Proxy::all(socks5_env)?;

    // rest client to authenticate
    client_builder.proxy(socks5proxy).build()?
  } else {
    client_builder.build()?
  };

  let api_url: String = format!(
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
