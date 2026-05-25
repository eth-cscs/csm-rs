//! Wire-format types — mirror the upstream CSM OpenAPI schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

use manta_backend_dispatcher::types::hsm::inventory::{
  DiscoveryInfo as FrontEndDiscoveryInfo,
  RedfishEndpoint as FrontEndRedfishEndpoint,
  RedfishEndpointArray as FrontEndRedfishEndpointArray,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscoveryInfo {
  #[serde(rename = "LastAttempt")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub last_attempt: Option<String>,
  #[serde(rename = "LastStatus")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub last_status: Option<String>,
  #[serde(rename = "RedfishVersion")]
  #[serde(skip_serializing_if = "Option::is_none")]
  redfish_version: Option<String>,
}

impl From<FrontEndDiscoveryInfo> for DiscoveryInfo {
  fn from(info: FrontEndDiscoveryInfo) -> Self {
    DiscoveryInfo {
      last_attempt: info.last_attempt,
      last_status: info.last_status,
      redfish_version: info.redfish_version,
    }
  }
}

impl From<DiscoveryInfo> for FrontEndDiscoveryInfo {
  fn from(val: DiscoveryInfo) -> Self {
    FrontEndDiscoveryInfo {
      last_attempt: val.last_attempt,
      last_status: val.last_status,
      redfish_version: val.redfish_version,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishEndpoint {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "Hostname")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hostname: Option<String>,
  #[serde(rename = "Domain")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub domain: Option<String>,
  #[serde(rename = "FQDN")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fqdn: Option<String>,
  #[serde(rename = "Enabled")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub enabled: Option<bool>,
  #[serde(rename = "UUID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub uuid: Option<String>,
  #[serde(rename = "User")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user: Option<String>,
  #[serde(rename = "Password")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub password: Option<String>,
  #[serde(rename = "UseSSDP")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub use_ssdp: Option<bool>,
  #[serde(rename = "MacRequired")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mac_required: Option<bool>,
  #[serde(rename = "MACAddr")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mac_addr: Option<String>,
  #[serde(rename = "IPAddress")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ip_address: Option<String>,
  #[serde(rename = "RediscoveryOnUpdate")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rediscover_on_update: Option<bool>,
  #[serde(rename = "TemplateID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub template_id: Option<String>,
  #[serde(rename = "DiscoveryInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub discovery_info: Option<DiscoveryInfo>,
}

impl From<FrontEndRedfishEndpoint> for RedfishEndpoint {
  fn from(endpoint: FrontEndRedfishEndpoint) -> Self {
    RedfishEndpoint {
      id: endpoint.id,
      r#type: endpoint.r#type,
      name: endpoint.name,
      hostname: endpoint.hostname,
      domain: endpoint.domain,
      fqdn: endpoint.fqdn,
      enabled: endpoint.enabled,
      uuid: endpoint.uuid,
      user: endpoint.user,
      password: endpoint.password,
      use_ssdp: endpoint.use_ssdp,
      mac_required: endpoint.mac_required,
      mac_addr: endpoint.mac_addr,
      ip_address: endpoint.ip_address,
      rediscover_on_update: endpoint.rediscover_on_update,
      template_id: endpoint.template_id,
      discovery_info: endpoint.discovery_info.map(|info| info.into()),
    }
  }
}

impl From<RedfishEndpoint> for FrontEndRedfishEndpoint {
  fn from(val: RedfishEndpoint) -> Self {
    FrontEndRedfishEndpoint {
      id: val.id,
      r#type: val.r#type,
      name: val.name,
      hostname: val.hostname,
      domain: val.domain,
      fqdn: val.fqdn,
      enabled: val.enabled,
      uuid: val.uuid,
      user: val.user,
      password: val.password,
      use_ssdp: val.use_ssdp,
      mac_required: val.mac_required,
      mac_addr: val.mac_addr,
      ip_address: val.ip_address,
      rediscover_on_update: val.rediscover_on_update,
      template_id: val.template_id,
      discovery_info: val.discovery_info.map(|info| info.into()),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishEndpointArray {
  #[serde(rename = "RedfishEndpoints")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub redfish_endpoints: Option<Vec<RedfishEndpoint>>,
}

impl From<FrontEndRedfishEndpointArray> for RedfishEndpointArray {
  fn from(array: FrontEndRedfishEndpointArray) -> Self {
    RedfishEndpointArray {
      redfish_endpoints: array.redfish_endpoints.map(|endpoints| {
        endpoints.into_iter().map(RedfishEndpoint::from).collect()
      }),
    }
  }
}

impl From<RedfishEndpointArray> for FrontEndRedfishEndpointArray {
  fn from(val: RedfishEndpointArray) -> Self {
    FrontEndRedfishEndpointArray {
      redfish_endpoints: val.redfish_endpoints.map(|endpoints| {
        endpoints
          .into_iter()
          .map(|endpoint| endpoint.into())
          .collect()
      }),
    }
  }
}
