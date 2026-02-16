use hostlist_parser::parse;
use manta_backend_dispatcher::{
  error::Error,
  interfaces::hsm::{
    component::ComponentTrait,
    component_ethernet_interface::ComponentEthernetInterfaceTrait,
    group::GroupTrait, hardware_inventory::HardwareInventory,
    redfish_endpoint::RedfishEndpointTrait,
  },
  types::{
    Component, ComponentArrayPostArray as FrontEndComponentArrayPostArray,
    HWInventoryByLocationList as FrontEndHWInventoryByLocationList,
    NodeMetadataArray,
    hsm::inventory::{
      ComponentEthernetInterface,
      RedfishEndpointArray as FrontEndRedfishEndpointArray,
    },
  },
};
use regex::Regex;
use serde_json::Value;

use super::Csm;
use crate::hsm::{self, component::types::ComponentArrayPostArray};

impl HardwareInventory for Csm {
  async fn get_inventory_hardware(
    &self,
    auth_token: &str,
    xname: &str,
  ) -> Result<Value, Error> {
    hsm::hw_inventory::hw_component::http_client::get(
      auth_token,
      &self.base_url,
      &self.root_cert,
      xname,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
    .and_then(|hw_inventory| {
      serde_json::to_value(hw_inventory)
        .map_err(|e| Error::Message(e.to_string()))
    })
  }

  async fn get_inventory_hardware_query(
    &self,
    auth_token: &str,
    xname: &str,
    r#_type: Option<&str>,
    _children: Option<bool>,
    _parents: Option<bool>,
    _partition: Option<&str>,
    _format: Option<&str>,
  ) -> Result<Value, Error> {
    hsm::hw_inventory::hw_component::http_client::get_query(
      &auth_token,
      &self.base_url,
      &self.root_cert,
      xname,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn post_inventory_hardware(
    &self,
    auth_token: &str,
    hw_inventory: FrontEndHWInventoryByLocationList,
  ) -> Result<Value, Error> {
    hsm::hw_inventory::hw_component::http_client::post(
      auth_token,
      &self.base_url,
      &self.root_cert,
      hw_inventory.into(),
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }
}

impl ComponentTrait for Csm {
  async fn get_all_nodes(
    &self,
    auth_token: &str,
    nid_only: Option<&str>,
  ) -> Result<NodeMetadataArray, Error> {
    hsm::component::http_client::get(
      &self.base_url,
      &self.root_cert,
      auth_token,
      None,
      Some("Node"),
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      nid_only,
    )
    .await
    .map(|c| c.into())
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn get_node_metadata_available(
    &self,
    auth_token: &str,
  ) -> Result<Vec<Component>, Error> {
    let xname_available_vec: Vec<String> = self
      .get_group_available(auth_token)
      .await
      .map_err(|e| Error::Message(e.to_string()))?
      .iter()
      .flat_map(|group| group.get_members())
      .collect();

    let node_metadata_vec_rslt = self
      .get_all_nodes(auth_token, Some("true"))
      .await
      .unwrap()
      .components
      .unwrap_or_default()
      .iter()
      .filter(|&node_metadata| {
        xname_available_vec.contains(&node_metadata.id.as_ref().unwrap())
      })
      .cloned()
      .collect();

    let node_metadata_vec: Vec<Component> = node_metadata_vec_rslt;

    Ok(node_metadata_vec)
  }

  async fn get(
    &self,
    auth_token: &str,
    id: Option<&str>,
    r#type: Option<&str>,
    state: Option<&str>,
    flag: Option<&str>,
    role: Option<&str>,
    subrole: Option<&str>,
    enabled: Option<&str>,
    software_status: Option<&str>,
    subtype: Option<&str>,
    arch: Option<&str>,
    class: Option<&str>,
    nid: Option<&str>,
    nid_start: Option<&str>,
    nid_end: Option<&str>,
    partition: Option<&str>,
    group: Option<&str>,
    state_only: Option<&str>,
    flag_only: Option<&str>,
    role_only: Option<&str>,
    nid_only: Option<&str>,
  ) -> Result<NodeMetadataArray, Error> {
    hsm::component::http_client::get(
      &self.base_url,
      &self.root_cert,
      auth_token,
      id,
      r#type,
      state,
      flag,
      role,
      subrole,
      enabled,
      software_status,
      subtype,
      arch,
      class,
      nid,
      nid_start,
      nid_end,
      partition,
      group,
      state_only,
      flag_only,
      role_only,
      nid_only,
    )
    .await
    .map(|c| c.into())
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn post_nodes(
    &self,
    auth_token: &str,
    component: FrontEndComponentArrayPostArray,
  ) -> Result<(), Error> {
    let component_backend: ComponentArrayPostArray = component.into();

    hsm::component::http_client::post(
      auth_token,
      &self.base_url,
      &self.root_cert,
      component_backend,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn delete_node(
    &self,
    auth_token: &str,
    id: &str,
  ) -> Result<Value, Error> {
    hsm::component::http_client::delete_one(
      auth_token,
      &self.base_url,
      &self.root_cert,
      id,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }

  /// Get list of xnames from NIDs
  /// The list of NIDs can be:
  ///     - comma separated list of NIDs (eg: nid000001,nid000002,nid000003)
  ///     - regex (eg: nid00000.*)
  ///     - hostlist (eg: nid0000[01-15])
  async fn nid_to_xname(
    &self,
    shasta_token: &str,
    user_input_nid: &str,
    is_regex: bool,
  ) -> Result<Vec<String>, Error> {
    if is_regex {
      log::debug!("Regex found, getting xnames from NIDs");
      // Get list of regex
      let regex_vec: Vec<Regex> = user_input_nid
        .split(",")
        .map(|regex_str| Regex::new(regex_str.trim()))
        .collect::<Result<Vec<Regex>, regex::Error>>()
        .map_err(|e| Error::Message(e.to_string()))?;

      // Get all HSM components (list of xnames + nids)
      let hsm_component_vec = hsm::component::http_client::get_all_nodes(
        &self.base_url,
        &self.root_cert,
        shasta_token,
        Some("true"),
      )
      .await
      .map_err(|e| Error::Message(e.to_string()))?
      .components
      .unwrap_or_default();

      let mut xname_vec: Vec<String> = vec![];

      // Get list of xnames the user is asking for
      for hsm_component in hsm_component_vec {
        let nid_long = format!(
          "nid{:06}",
          &hsm_component
            .nid
            .ok_or_else(|| Error::Message("No NID found".to_string()))?
        );
        for regex in &regex_vec {
          if regex.is_match(&nid_long) {
            log::debug!(
              "Nid '{}' IS included in regex '{}'",
              nid_long,
              regex.as_str()
            );
            xname_vec.push(
              hsm_component
                .id
                .clone()
                .ok_or_else(|| Error::Message("No XName found".to_string()))?,
            );
          }
        }
      }

      return Ok(xname_vec);
    } else {
      log::debug!(
        "No regex found, getting xnames from list of NIDs or NIDs hostlist"
      );
      let nid_hostlist_expanded_vec = parse(user_input_nid).map_err(|e| {
        Error::Message(format!(
          "Could not parse list of nodes as a hostlist. Reason:\n{}Exit",
          e
        ))
      })?;

      log::debug!("hostlist: {}", user_input_nid);
      log::debug!("hostlist expanded: {:?}", nid_hostlist_expanded_vec);

      let mut nid_short_vec = Vec::new();

      for nid_long in nid_hostlist_expanded_vec {
        let nid_short_elem = nid_long
          .strip_prefix("nid")
          .ok_or_else(|| {
            Error::Message(format!(
              "Nid '{}' not valid, 'nid' prefix missing",
              nid_long
            ))
          })?
          .trim_start_matches("0");

        nid_short_vec.push(nid_short_elem.to_string());
      }

      let nid_short = nid_short_vec.join(",");

      log::debug!("short NID list: {}", nid_short);

      let hsm_components = hsm::component::http_client::get(
        &self.base_url,
        &self.root_cert,
        shasta_token,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&nid_short),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("true"),
      )
      .await
      .map_err(|e| Error::Message(e.to_string()))?;

      // Get list of xnames from HSM components
      let xname_vec: Vec<String> = hsm_components
        .components
        .unwrap_or_default()
        .iter()
        .map(|component| component.id.clone().unwrap())
        .collect();

      log::debug!("xname list:\n{:#?}", xname_vec);

      return Ok(xname_vec);
    };
  }
}

impl ComponentEthernetInterfaceTrait for Csm {
  async fn get_all_component_ethernet_interfaces(
    &self,
    _auth_token: &str,
  ) -> Result<Vec<ComponentEthernetInterface>, Error> {
    Err(Error::Message(
      "Get all ethernet interfaces command not implemented for this backend"
        .to_string(),
    ))
  }

  async fn get_component_ethernet_interface(
    &self,
    _auth_token: &str,
    _eth_interface_id: &str,
  ) -> Result<ComponentEthernetInterface, Error> {
    Err(Error::Message(
      "Get ethernet interfaces command not implemented for this backend"
        .to_string(),
    ))
  }

  async fn update_component_ethernet_interface(
    &self,
    _auth_token: &str,
    _eth_interface_id: &str,
    _description: Option<&str>,
    _ip_address_mapping: (&str, &str),
  ) -> Result<Value, Error> {
    Err(Error::Message(
      "Update ethernet interface command not implemented for this backend"
        .to_string(),
    ))
  }

  async fn delete_all_component_ethernet_interfaces(
    &self,
    _auth_token: &str,
  ) -> Result<Value, Error> {
    Err(Error::Message(
      "Delete all ethernet interface command not implemented for this backend"
        .to_string(),
    ))
  }

  async fn delete_component_ethernet_interface(
    &self,
    _auth_token: &str,
    _eth_interface_id: &str,
  ) -> Result<Value, Error> {
    Err(Error::Message(
      "Delete ethernet interface command not implemented for this backend"
        .to_string(),
    ))
  }

  /* async fn get_ip_addresses(
    &self,
    _auth_token: &str,
    _eth_interface_id: &str,
  ) -> Result<Vec<IpAddressMapping>, Error> {
    Err(Error::Message(
      "Get IP addresses command not implemented for this backend".to_string(),
    ))
  }

  async fn delete_ip_address(
    &self,
    _auth_token: &str,
    _group_label: &str,
    _eth_interface_id: &str,
    _ip_address: &str,
  ) -> Result<Value, Error> {
    Err(Error::Message(
      "Delete IP address command not implemented for this backend".to_string(),
    ))
  } */
}

impl RedfishEndpointTrait for Csm {
  async fn get_all_redfish_endpoints(
    &self,
    _auth_token: &str,
  ) -> Result<FrontEndRedfishEndpointArray, Error> {
    Err(Error::Message(
      "Get all redfish endpoint command not implemented for this backend"
        .to_string(),
    ))
  }
  async fn get_redfish_endpoints(
    &self,
    _auth_token: &str,
    _id: Option<&str>,
    _fqdn: Option<&str>,
    _type: Option<&str>,
    _uuid: Option<&str>,
    _macaddr: Option<&str>,
    _ip_address: Option<&str>,
    _last_status: Option<&str>,
  ) -> Result<FrontEndRedfishEndpointArray, Error> {
    Err(Error::Message(
      "Get redfish endpoint command not implemented for this backend"
        .to_string(),
    ))
  }

  async fn add_redfish_endpoint(
    &self,
    _auth_token: &str,
    _redfish_endpoint: &manta_backend_dispatcher::types::hsm::inventory::RedfishEndpointArray,
  ) -> Result<(), Error> {
    Err(Error::Message(
      "Add redfish endpoint command not implemented for this backend"
        .to_string(),
    ))
  }

  async fn update_redfish_endpoint(
    &self,
    _auth_token: &str,
    _redfish_endpoint: &manta_backend_dispatcher::types::hsm::inventory::RedfishEndpoint,
  ) -> Result<(), Error> {
    Err(Error::Message(
      "Update redfish endpoint command not implemented for this backend"
        .to_string(),
    ))
  }

  async fn delete_redfish_endpoint(
    &self,
    _auth_token: &str,
    _id: &str,
  ) -> Result<Value, Error> {
    Err(Error::Message(
      "Delete redfish endpoint command not implemented for this backend"
        .to_string(),
    ))
  }
}
