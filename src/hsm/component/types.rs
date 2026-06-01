//! Wire-format types — mirror the upstream CSM OpenAPI schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentArray {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Components")]
  pub components: Option<Vec<Component>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Component {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "ID")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Type")]
  pub r#type: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "State")]
  pub state: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Flag")]
  pub flag: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Enabled")]
  pub enabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "SoftwareStatus")]
  pub software_status: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Role")]
  pub role: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "SubRole")]
  pub sub_role: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "NID")]
  pub nid: Option<usize>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Subtype")]
  pub subtype: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "NetType")]
  pub net_type: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Arch")]
  pub arch: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Class")]
  pub class: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "ReservationDisabled")]
  pub reservation_disabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Locked")]
  pub locked: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentArrayPostQuery {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "ComponentIDs")]
  pub component_ids: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub partition: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub group: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "stateonly")]
  pub state_only: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "flagonly")]
  pub falg_only: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "roleonly")]
  pub role_only: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "nidonly")]
  pub nid_only: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub state: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub flag: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub enabled: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "softwarestatus")]
  pub software_status: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub role: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub subrole: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub subtype: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arch: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub class: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub nid: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub nid_start: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub nid_end: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentArrayPostByNidQuery {
  #[serde(rename = "NIDRanges")]
  pub nid_ranges: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub partition: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "stateonly")]
  pub state_only: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "flagonly")]
  pub falg_only: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "roleonly")]
  pub role_only: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "nidonly")]
  pub nid_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentArrayPostArray {
  #[serde(rename = "Components")]
  pub components: Vec<ComponentCreate>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Force")]
  pub force: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentCreate {
  #[serde(rename = "ID")]
  pub(super) id: String,
  #[serde(rename = "State")]
  pub(super) state: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Flag")]
  pub(super) flag: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Enabled")]
  pub(super) enabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "SoftwareStatus")]
  pub(super) software_status: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Role")]
  pub(super) role: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "SubRole")]
  pub(super) sub_role: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "NID")]
  pub(super) nid: Option<usize>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Subtype")]
  pub(super) subtype: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "NetType")]
  pub(super) net_type: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Arch")]
  pub(super) arch: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Class")]
  pub(super) class: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentPut {
  component: ComponentCreate,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "Force")]
  force: Option<bool>,
}
