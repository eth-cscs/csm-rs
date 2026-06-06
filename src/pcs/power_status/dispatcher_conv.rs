//! Bidirectional `From` impls between csm-rs's PCS power-status types
//! and the dispatcher's mirrors. Gated behind the `manta-dispatcher`
//! Cargo feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::pcs::power_status::types::{
  ManagementState as FrontEndManagementState, PowerState as FrontEndPowerState,
  PowerStatus as FrontEndPowerStatus, PowerStatusAll as FrontEndPowerStatusAll,
};

use super::types::{ManagementState, PowerState, PowerStatus, PowerStatusAll};

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

impl From<FrontEndPowerStatus> for PowerStatus {
  fn from(value: FrontEndPowerStatus) -> Self {
    PowerStatus {
      xname: value.xname,
      power_state: value.power_state.map(PowerState::from),
      management_state: value.management_state.map(ManagementState::from),
      error: value.error,
      supported_power_transitions: value
        .supported_power_transitions
        .into_iter()
        .map(Into::into)
        .collect(),
      last_updated: value.last_updated,
    }
  }
}

impl From<PowerStatus> for FrontEndPowerStatus {
  fn from(val: PowerStatus) -> Self {
    FrontEndPowerStatus {
      xname: val.xname,
      power_state: val.power_state.map(std::convert::Into::into),
      management_state: val.management_state.map(std::convert::Into::into),
      error: val.error,
      supported_power_transitions: val
        .supported_power_transitions
        .into_iter()
        .map(std::convert::Into::into)
        .collect(),
      last_updated: val.last_updated,
    }
  }
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
