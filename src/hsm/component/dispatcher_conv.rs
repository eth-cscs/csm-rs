//! Bidirectional `From` impls between csm-rs's HSM component types and
//! the dispatcher's mirrors. Gated behind the `manta-dispatcher` Cargo
//! feature so users not on Manta don't pull the dispatcher dep.
//!
//! Field shapes diverge after the progenitor migration: the dispatcher
//! mirror types still use plain `Option<String>` / `Option<usize>`
//! everywhere, while the csm-rs (progenitor-generated) types wrap most
//! string fields in HSM newtype/enum types (`XName100`, `HmsType100`,
//! `HmsState100`, `HmsRole100`, …). The conversions below handle this
//! by:
//!
//! - For newtype wrappers (`XName100`, `HmsRole100`, `HmsSubRole100`,
//!   `XNameRw100`): unwrap `.0` / construct `Newtype(string)`.
//! - For enum types (`HmsType100`, `HmsState100`, `HmsFlag100`,
//!   `HmsArch100`, `HmsClass100`, `NetType100`): the `Display` impl
//!   emits the wire string; `FromStr` parses it back. Parse failures
//!   (e.g. an unknown role coming from the dispatcher side) are silently
//!   dropped to `None` — same lossy behaviour as the historical
//!   `Option<String>` round-trip would have shown if the value got
//!   reflected to a typed-only consumer downstream.
//! - For the NID field: `Option<usize>` <-> `Option<i64>` via `as` cast.
//!   The conservative cast preserves the historical numeric range; CSM
//!   real-world NIDs fit comfortably in both.

use manta_backend_dispatcher::types::{
  Component as FrontEndComponent,
  ComponentArrayPostArray as FrontEndComponentArrayPostArray,
  ComponentCreate as FrontEndComponentCreate,
  NodeMetadataArray as FrontEndNodeMetadataArray,
};

use super::types::{
  Component, ComponentArray, ComponentArrayPostArray, ComponentCreate,
  HmsRole100, HmsState100, HmsSubRole100, XName100, XNameRw100,
};

impl From<FrontEndNodeMetadataArray> for ComponentArray {
  fn from(value: FrontEndNodeMetadataArray) -> Self {
    let component_vec: Vec<Component> = match value.components {
      Some(components) => components.into_iter().map(Component::from).collect(),
      None => Vec::new(),
    };

    ComponentArray {
      components: component_vec,
    }
  }
}

impl From<ComponentArray> for FrontEndNodeMetadataArray {
  fn from(val: ComponentArray) -> Self {
    // The OpenAPI schema makes `Components` `#[serde(default)]`, so the
    // empty/absent distinction is lost on the csm-rs side. Mirror the
    // historical "always `Some(...)`" wrapping to keep the dispatcher
    // contract identical.
    let component_vec: Vec<FrontEndComponent> =
      val.components.into_iter().map(Into::into).collect();

    FrontEndNodeMetadataArray {
      components: Some(component_vec),
    }
  }
}

impl From<FrontEndComponent> for Component {
  fn from(value: FrontEndComponent) -> Self {
    Component {
      id: value.id.map(XName100),
      type_: value.r#type.as_deref().and_then(|s| s.parse().ok()),
      state: value.state.as_deref().and_then(|s| s.parse().ok()),
      flag: value.flag.as_deref().and_then(|s| s.parse().ok()),
      enabled: value.enabled,
      software_status: value.software_status,
      role: value.role.map(HmsRole100),
      sub_role: value.sub_role.map(HmsSubRole100),
      // Schema-faithful widening: dispatcher uses `usize` (its own
      // historical misread), generated client uses `i64` per the
      // OpenAPI `type: integer` declaration. The conversion is safe in
      // practice for any real CSM NID.
      #[allow(clippy::cast_possible_wrap)]
      nid: value.nid.map(|n| n as i64),
      subtype: value.subtype,
      net_type: value.net_type.as_deref().and_then(|s| s.parse().ok()),
      arch: value.arch.as_deref().and_then(|s| s.parse().ok()),
      class: value.class.as_deref().and_then(|s| s.parse().ok()),
      reservation_disabled: value.reservation_disabled,
      locked: value.locked,
    }
  }
}

impl From<Component> for FrontEndComponent {
  fn from(val: Component) -> Self {
    FrontEndComponent {
      id: val.id.map(|x| x.0),
      r#type: val.type_.map(|t| t.to_string()),
      state: val.state.map(|s| s.to_string()),
      flag: val.flag.map(|f| f.to_string()),
      enabled: val.enabled,
      software_status: val.software_status,
      role: val.role.map(|r| r.0),
      sub_role: val.sub_role.map(|s| s.0),
      // Narrowing cast: `i64` -> `usize` is safe for real CSM NIDs,
      // which are non-negative and well under `u32::MAX`.
      #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
      nid: val.nid.map(|n| n as usize),
      subtype: val.subtype,
      net_type: val.net_type.map(|n| n.to_string()),
      arch: val.arch.map(|a| a.to_string()),
      class: val.class.map(|c| c.to_string()),
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
    // ComponentCreate's `state` is required (`HmsState100`); the
    // dispatcher carries it as a plain `String`. Use `parse()` and
    // fall back to `Unknown` on a malformed value — the alternative
    // (returning a Result from `From`) would force the whole call site
    // through `try_into`. The historical code did no validation at all,
    // so this is strictly stricter at the deserialization boundary.
    let state = value
      .state
      .parse::<HmsState100>()
      .unwrap_or(HmsState100::Unknown);

    ComponentCreate {
      id: XNameRw100(value.id),
      state,
      flag: value.flag.as_deref().and_then(|s| s.parse().ok()),
      enabled: value.enabled,
      software_status: value.software_status,
      role: value.role.map(HmsRole100),
      sub_role: value.sub_role.map(HmsSubRole100),
      #[allow(clippy::cast_possible_wrap)]
      nid: value.nid.map(|n| n as i64),
      subtype: value.subtype,
      net_type: value.net_type.as_deref().and_then(|s| s.parse().ok()),
      arch: value.arch.as_deref().and_then(|s| s.parse().ok()),
      class: value.class.as_deref().and_then(|s| s.parse().ok()),
    }
  }
}

impl From<ComponentCreate> for FrontEndComponentCreate {
  fn from(val: ComponentCreate) -> Self {
    FrontEndComponentCreate {
      id: val.id.0,
      state: val.state.to_string(),
      flag: val.flag.map(|f| f.to_string()),
      enabled: val.enabled,
      software_status: val.software_status,
      role: val.role.map(|r| r.0),
      sub_role: val.sub_role.map(|s| s.0),
      #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
      nid: val.nid.map(|n| n as usize),
      subtype: val.subtype,
      net_type: val.net_type.map(|n| n.to_string()),
      arch: val.arch.map(|a| a.to_string()),
      class: val.class.map(|c| c.to_string()),
    }
  }
}
