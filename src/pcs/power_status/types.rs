use manta_backend_dispatcher::types::pcs::power_status::types::{
    PowerStatus as FrontEndPowerStatus,
    PowerStatusAll as FrontEndPowerStatusAll,
    PowerState as FrontEndPowerState,
    ManagementState as FrontEndManagementState,
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
impl Into<FrontEndPowerState> for PowerState {
    fn into(self) -> FrontEndPowerState {
        match self {
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
impl Into<FrontEndManagementState> for ManagementState {
    fn into(self) -> FrontEndManagementState {
        match self {
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
            power_state: value.power_state.map(|v| PowerState::from(v)),
            management_state: value.management_state.map(|v| ManagementState::from(v)),
            //management_state_filter: value.management_state_filter.map(|v| ManagementState::from(v)),
            error: value.error,
            supported_power_transitions: value.supported_power_transitions.into_iter().map(|v| Operation::from(v)).collect(),
            last_updated: value.last_updated,
    }
  }
}

impl Into<FrontEndPowerStatus> for PowerStatus {
  fn into(self) -> FrontEndPowerStatus {
    FrontEndPowerStatus {
        xname: self.xname,
        //power_state_filter: self.power_state_filter.map(|v| v.into()),
        power_state: self.power_state.map( |v| v.into()),
        management_state: self.management_state.map( |v| v.into()),
        //management_state_filter: self.management_state_filter.map( |v| v.into()),
        error: self.error,
        supported_power_transitions: self.supported_power_transitions.into_iter().map( |v| v.into()).collect(),
        last_updated: self.last_updated,
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
            status: value.status.into_iter().map(PowerStatus::from).collect()
        }
    }
}

impl Into<FrontEndPowerStatusAll> for PowerStatusAll {
    fn into(self) -> FrontEndPowerStatusAll {
        FrontEndPowerStatusAll {
            status: self.status.into_iter().map(Into::into).collect()
        }
    }
}

