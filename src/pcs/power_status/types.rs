//! Wire-format types — mirror the upstream CSM OpenAPI schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::pcs::transitions::types::Operation;

#[derive(Debug, Serialize, Deserialize)]
pub enum PowerState {
  #[serde(rename = "on")]
  On,
  #[serde(rename = "off")]
  Off,
  #[serde(rename = "undefined")]
  Undefined,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ManagementState {
  #[serde(rename = "unavailable")]
  Unavailable,
  #[serde(rename = "available")]
  Available,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PowerStatus {
  pub xname: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "powerState")]
  pub power_state: Option<PowerState>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "managementState")]
  pub management_state: Option<ManagementState>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "error")]
  pub(super) error: Option<String>,
  #[serde(rename = "supportedPowerTransitions")]
  pub supported_power_transitions: Vec<Operation>,
  #[serde(rename = "lastUpdated")]
  pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PowerStatusAll {
  pub status: Vec<PowerStatus>,
}
