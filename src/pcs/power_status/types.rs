//! Wire-format types — mirror the upstream CSM OpenAPI schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

use manta_backend_dispatcher::types::pcs::power_status::types::{
  ManagementState as FrontEndManagementState, PowerState as FrontEndPowerState,
  PowerStatus as FrontEndPowerStatus, PowerStatusAll as FrontEndPowerStatusAll,
};

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
impl From<FrontEndPowerState> for PowerState {
  fn from(value: FrontEndPowerState) -> Self {
    match value {
      FrontEndPowerState::On => PowerState::On,
      FrontEndPowerState::Off => PowerState::Off,
      FrontEndPowerState::Undefined => PowerState::Undefined,
    }
  }
}
impl From<PowerState> for FrontEndPowerState {
  fn from(val: PowerState) -> Self {
    match val {
      PowerState::On => FrontEndPowerState::On,
      PowerState::Off => FrontEndPowerState::Off,
      PowerState::Undefined => FrontEndPowerState::Undefined,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ManagementState {
  #[serde(rename = "unavailable")]
  Unavailable,
  #[serde(rename = "available")]
  Available,
}
impl From<FrontEndManagementState> for ManagementState {
  fn from(value: FrontEndManagementState) -> Self {
    match value {
      FrontEndManagementState::Unavailable => ManagementState::Unavailable,
      FrontEndManagementState::Available => ManagementState::Available,
    }
  }
}
impl From<ManagementState> for FrontEndManagementState {
  fn from(val: ManagementState) -> Self {
    match val {
      ManagementState::Unavailable => FrontEndManagementState::Unavailable,
      ManagementState::Available => FrontEndManagementState::Available,
    }
  }
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
  error: Option<String>,
  #[serde(rename = "supportedPowerTransitions")]
  pub supported_power_transitions: Vec<Operation>,
  #[serde(rename = "lastUpdated")]
  pub last_updated: String,
}

impl From<FrontEndPowerStatus> for PowerStatus {
  fn from(value: FrontEndPowerStatus) -> Self {
    PowerStatus {
      xname: value.xname,
      //power_state_filter: value.power_state_filter.map( |v| PowerState::from(v)),
      power_state: value.power_state.map(PowerState::from),
      management_state: value.management_state.map(ManagementState::from),
      //management_state_filter: value.management_state_filter.map(|v| ManagementState::from(v)),
      error: value.error,
      supported_power_transitions: value
        .supported_power_transitions
        .into_iter()
        .map(Operation::from)
        .collect(),
      last_updated: value.last_updated,
    }
  }
}

impl From<PowerStatus> for FrontEndPowerStatus {
  fn from(val: PowerStatus) -> Self {
    FrontEndPowerStatus {
      xname: val.xname,
      //power_state_filter: self.power_state_filter.map(|v| v.into()),
      power_state: val.power_state.map(|v| v.into()),
      management_state: val.management_state.map(|v| v.into()),
      //management_state_filter: self.management_state_filter.map( |v| v.into()),
      error: val.error,
      supported_power_transitions: val
        .supported_power_transitions
        .into_iter()
        .map(|v| v.into())
        .collect(),
      last_updated: val.last_updated,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PowerStatusAll {
  pub status: Vec<PowerStatus>,
}

impl From<FrontEndPowerStatusAll> for PowerStatusAll {
  fn from(value: FrontEndPowerStatusAll) -> Self {
    PowerStatusAll {
      status: value.status.into_iter().map(PowerStatus::from).collect(),
    }
  }
}

impl From<PowerStatusAll> for FrontEndPowerStatusAll {
  fn from(val: PowerStatusAll) -> Self {
    FrontEndPowerStatusAll {
      status: val.status.into_iter().map(Into::into).collect(),
    }
  }
}
