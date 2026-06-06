//! Bidirectional `From` impls between csm-rs's HSM component types and
//! the dispatcher's mirrors. Gated behind the `manta-dispatcher` Cargo
//! feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::{
  Component as FrontEndComponent,
  ComponentArrayPostArray as FrontEndComponentArrayPostArray,
  ComponentCreate as FrontEndComponentCreate,
  NodeMetadataArray as FrontEndNodeMetadataArray,
};

use super::types::{
  Component, ComponentArray, ComponentArrayPostArray, ComponentCreate,
};

impl From<FrontEndNodeMetadataArray> for ComponentArray {
  fn from(value: FrontEndNodeMetadataArray) -> Self {
    let component_vec_opt: Option<Vec<Component>> =
      if let Some(components) = value.components {
        let mut component_vec: Vec<Component> =
          Vec::with_capacity(components.len());

        for component in components {
            component_vec.push(Component::from(component));
          }

        Some(component_vec)
      } else {
        None
      };

    ComponentArray {
      components: component_vec_opt,
    }
  }
}

impl From<ComponentArray> for FrontEndNodeMetadataArray {
  fn from(val: ComponentArray) -> Self {
    let component_vec_opt: Option<Vec<FrontEndComponent>> =
      if let Some(components) = val.components {
        let mut component_vec: Vec<FrontEndComponent> =
          Vec::with_capacity(components.len());

        for component in components {
          component_vec.push(component.into());
        }

        Some(component_vec)
      } else {
        None
      };

    FrontEndNodeMetadataArray {
      components: component_vec_opt,
    }
  }
}

impl From<FrontEndComponent> for Component {
  fn from(value: FrontEndComponent) -> Self {
    Component {
      id: value.id,
      r#type: value.r#type,
      state: value.state,
      flag: value.flag,
      enabled: value.enabled,
      software_status: value.software_status,
      role: value.role,
      sub_role: value.sub_role,
      nid: value.nid,
      subtype: value.subtype,
      net_type: value.net_type,
      arch: value.arch,
      class: value.class,
      reservation_disabled: value.reservation_disabled,
      locked: value.locked,
    }
  }
}

impl From<Component> for FrontEndComponent {
  fn from(val: Component) -> Self {
    FrontEndComponent {
      id: val.id,
      r#type: val.r#type,
      state: val.state,
      flag: val.flag,
      enabled: val.enabled,
      software_status: val.software_status,
      role: val.role,
      sub_role: val.sub_role,
      nid: val.nid,
      subtype: val.subtype,
      net_type: val.net_type,
      arch: val.arch,
      class: val.class,
      reservation_disabled: val.reservation_disabled,
      locked: val.locked,
    }
  }
}

impl From<FrontEndComponentArrayPostArray> for ComponentArrayPostArray {
  fn from(value: FrontEndComponentArrayPostArray) -> Self {
    let mut component_vec: Vec<ComponentCreate> =
      Vec::with_capacity(value.components.len());

    value
      .components
      .into_iter()
      .for_each(|c| component_vec.push(c.into()));

    ComponentArrayPostArray {
      components: component_vec,
      force: value.force,
    }
  }
}

impl From<ComponentArrayPostArray> for FrontEndComponentArrayPostArray {
  fn from(val: ComponentArrayPostArray) -> Self {
    let mut component_vec: Vec<FrontEndComponentCreate> =
      Vec::with_capacity(val.components.len());

    val
      .components
      .into_iter()
      .for_each(|c| component_vec.push(c.into()));

    FrontEndComponentArrayPostArray {
      components: component_vec,
      force: val.force,
    }
  }
}

impl From<FrontEndComponentCreate> for ComponentCreate {
  fn from(value: FrontEndComponentCreate) -> Self {
    ComponentCreate {
      id: value.id,
      state: value.state,
      flag: value.flag,
      enabled: value.enabled,
      software_status: value.software_status,
      role: value.role,
      sub_role: value.sub_role,
      nid: value.nid,
      subtype: value.subtype,
      net_type: value.net_type,
      arch: value.arch,
      class: value.class,
    }
  }
}

impl From<ComponentCreate> for FrontEndComponentCreate {
  fn from(val: ComponentCreate) -> Self {
    FrontEndComponentCreate {
      id: val.id,
      state: val.state,
      flag: val.flag,
      enabled: val.enabled,
      software_status: val.software_status,
      role: val.role,
      sub_role: val.sub_role,
      nid: val.nid,
      subtype: val.subtype,
      net_type: val.net_type,
      arch: val.arch,
      class: val.class,
    }
  }
}
