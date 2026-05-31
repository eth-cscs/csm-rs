//! Bidirectional `From` impls between csm-rs's HSM Redfish endpoint types
//! and the dispatcher's mirrors. Gated behind the `manta-dispatcher`
//! Cargo feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::hsm::inventory::{
  DiscoveryInfo as FrontEndDiscoveryInfo,
  RedfishEndpoint as FrontEndRedfishEndpoint,
  RedfishEndpointArray as FrontEndRedfishEndpointArray,
};

use super::types::{DiscoveryInfo, RedfishEndpoint, RedfishEndpointArray};

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
