use manta_backend_dispatcher::types::cfs::component::{
  Component as FrontEndComponent, State as FrontEndState,
};

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct State {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "cloneUrl")]
  pub clone_url: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub playbook: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub commit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "sesisonName")]
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
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub state: Option<Vec<State>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "stateAppend")]
  pub state_append: Option<State>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "desiredConfig")]
  pub desired_config: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "errorCount")]
  pub error_count: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "retryPolicy")]
  pub retry_policy: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub enabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "configurationStatus")]
  pub configuration_status: Option<String>, //values unconfigured, pending, failed, configured
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tags: Option<HashMap<String, String>>,
}

impl From<FrontEndComponent> for Component {
  fn from(component: FrontEndComponent) -> Self {
    let state_vec_opt = if let Some(component_state_vec) = component.state {
      let mut state_vec: Vec<State> = Vec::new();
      for state in component_state_vec {
        let state = State {
          clone_url: state.clone_url,
          playbook: state.playbook,
          commit: state.commit,
          session_name: state.session_name,
        };
        state_vec.push(state);
      }

      Some(state_vec)
    } else {
      None
    };

    Component {
      id: component.id,
      state: state_vec_opt,
      state_append: None,
      desired_config: component.desired_config,
      error_count: component.error_count,
      retry_policy: component.retry_policy,
      enabled: component.enabled,
      tags: component.tags,
      configuration_status: component.configuration_status,
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
      logs: None,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatchComponent {
  patch: Vec<Component>,
  filters: Filter,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Filter {
  #[serde(skip_serializing_if = "Option::is_none")]
  ids: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  status: Option<String>, // TODO: change to enum
  #[serde(skip_serializing_if = "Option::is_none")]
  enabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "configurationName")]
  config_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tags: Option<HashMap<String, String>>,
}
