//! Wire-format types — mirror the upstream CSM OpenAPI schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

// TODO: Update/Review these structs because:
// - PUT/PATH operations are tricky since some fields are read-only
// - State.lastUpdate field may be missing
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use manta_backend_dispatcher::types::cfs::component::{
  Component as FrontEndComponent, ComponentVec as FrontEndComponentVec,
  State as FrontEndState,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct State {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub clone_url: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub playbook: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub commit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub session_name: Option<String>,
}

impl From<FrontEndState> for State {
  fn from(state: FrontEndState) -> Self {
    State {
      clone_url: state.clone_url,
      playbook: state.playbook,
      commit: state.commit,
      session_name: state.session_name,
    }
  }
}

impl From<State> for FrontEndState {
  fn from(val: State) -> Self {
    FrontEndState {
      clone_url: val.clone_url,
      playbook: val.playbook,
      commit: val.commit,
      session_name: val.session_name,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Component {
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub state: Option<Vec<State>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub desired_config: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error_count: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub retry_policy: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub enabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub configuration_status: Option<String>, //values unconfigured, pending, failed, configured
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tags: Option<HashMap<String, String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub logs: Option<String>,
}

impl From<FrontEndComponent> for Component {
  fn from(component: FrontEndComponent) -> Self {
    Component {
      id: component.id,
      state: component.state.map(|state_vec| {
        state_vec.into_iter().map(|state| state.into()).collect()
      }),
      desired_config: component.desired_config,
      error_count: component.error_count,
      retry_policy: component.retry_policy,
      enabled: component.enabled,
      configuration_status: component.configuration_status,
      tags: component.tags,
      logs: component.logs,
    }
  }
}

impl From<Component> for FrontEndComponent {
  fn from(val: Component) -> Self {
    FrontEndComponent {
      id: val.id,
      state: val.state.map(|state_vec| {
        state_vec.into_iter().map(|state| state.into()).collect()
      }),
      desired_config: val.desired_config,
      error_count: val.error_count,
      retry_policy: val.retry_policy,
      enabled: val.enabled,
      configuration_status: val.configuration_status,
      tags: val.tags,
      logs: val.logs,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentVec {
  pub components: Vec<Component>,
}

impl From<FrontEndComponentVec> for ComponentVec {
  fn from(component_vec: FrontEndComponentVec) -> Self {
    Self {
      components: component_vec
        .components
        .into_iter()
        .map(|component| component.into())
        .collect(),
    }
  }
}

impl From<ComponentVec> for FrontEndComponentVec {
  fn from(val: ComponentVec) -> Self {
    FrontEndComponentVec {
      components: val
        .components
        .into_iter()
        .map(|component| component.into())
        .collect(),
    }
  }
}
