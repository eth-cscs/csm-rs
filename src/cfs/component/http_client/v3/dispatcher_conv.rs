//! Bidirectional `From` impls between csm-rs's CFS v3 component types and
//! the dispatcher's mirrors. Gated behind the `manta-dispatcher` Cargo
//! feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::cfs::component::{
  Component as FrontEndComponent, ComponentVec as FrontEndComponentVec,
  State as FrontEndState,
};

use super::types::{Component, ComponentVec, State};

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

impl From<FrontEndComponent> for Component {
  fn from(component: FrontEndComponent) -> Self {
    Component {
      id: component.id,
      state: component.state.map(|state_vec| {
        state_vec.into_iter().map(std::convert::Into::into).collect()
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
        state_vec.into_iter().map(std::convert::Into::into).collect()
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

impl From<FrontEndComponentVec> for ComponentVec {
  fn from(component_vec: FrontEndComponentVec) -> Self {
    Self {
      components: component_vec
        .components
        .into_iter()
        .map(std::convert::Into::into)
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
        .map(std::convert::Into::into)
        .collect(),
    }
  }
}
