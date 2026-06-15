//! Wire-format types — mirror the upstream CSM `OpenAPI` schema; field names and
//! shapes are dictated by the API.
//!
//! BEHAVIOUR FIX (Task 11): the previous hand-rolled types here either
//! lacked `#[serde(rename = "...")]` on snake_case fields (the spec uses
//! PascalCase — `MACAddress`, `IPAddresses`, `ComponentID`, …) or used
//! `ip_address: Option<String>` instead of the spec's
//! `IPAddresses: array<IPAddressMapping>`. Both meant the structs did
//! not (de)serialize the actual CSM wire format. Renames added below and
//! the `EthernetInterface` IP-addresses field reshaped to match the
//! spec; the bug fix is intentional but does change the JSON shape
//! produced/accepted by csm-rs callers — documented in the migration
//! commit as a BEHAVIOUR DELTA.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct IpAddressMapping {
  #[serde(rename = "IPAddress")]
  pub ip_address: String,
  #[serde(rename = "Network")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub network: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ComponentEthernetInterface {
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename = "IPAddresses")]
  pub ip_addresses: Vec<IpAddressMapping>,
  #[serde(rename = "ComponentID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub component_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ComponentType {
  CDU,
  CabinetCDU,
  CabinetPDU,
  CabinetPDUOutlet,
  CabinetPDUPowerConnector,
  CabinetPDUController,
  r#Cabinet,
  Chassis,
  ChassisBMC,
  CMMRectifier,
  CMMFpga,
  CEC,
  ComputeModule,
  RouterModule,
  NodeBMC,
  NodeEnclosure,
  NodeEnclosurePowerSupply,
  HSNBoard,
  Node,
  Processor,
  Drive,
  StorageGroup,
  NodeNIC,
  Memory,
  NodeAccel,
  NodeAccelRiser,
  NodeFpga,
  HSNAsic,
  RouterFpga,
  RouterBMC,
  HSNLink,
  HSNConnector,
  INVALID,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct EthernetInterface {
  #[serde(rename = "ID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename = "MACAddress")]
  pub mac_address: String,
  // BEHAVIOUR FIX: the prior hand-rolled struct had
  // `ip_address: Option<String>`. The spec defines
  // `IPAddresses: array<IPAddressMapping>`. Reshaped to match.
  #[serde(rename = "IPAddresses")]
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub ip_addresses: Vec<IpAddressMapping>,
  #[serde(rename = "LastUpdate")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub last_update: Option<String>,
  #[serde(rename = "ComponentID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub component_id: Option<String>,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<ComponentType>,
}
