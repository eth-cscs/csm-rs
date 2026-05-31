//! Bidirectional `From` impls between csm-rs's CFS v2 component types and
//! the dispatcher's mirrors. Gated behind the `manta-dispatcher` Cargo
//! feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::cfs::component::{
  Component as FrontEndComponent, State as FrontEndState,
};

use super::types::{Component, State};

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
