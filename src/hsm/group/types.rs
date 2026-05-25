//! Wire-format types — mirror the upstream CSM OpenAPI schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

use manta_backend_dispatcher::types::{
  Group as FrontEndGroup, Member as FrontEndMember,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
  pub label: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tags: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub members: Option<Members>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename(serialize = "exclusiveGroup"))]
  pub exclusive_group: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Members {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Member {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
}

impl Group {
  pub fn new(label: &str, member_vec_opt: Option<Vec<&str>>) -> Self {
    let members_opt = member_vec_opt.map(|member_vec| Members {
      ids: Some(member_vec.iter().map(|&id| id.to_string()).collect()),
    });

    Self {
      label: label.to_string(),
      description: None,
      tags: None,
      members: members_opt,
      exclusive_group: None,
    }
  }

  /// Get HSM group members
  pub fn get_members(&self) -> Vec<String> {
    // FIXME: try to improve this logic by introducing "smart pointers" or "lifetimes"
    self
      .members
      .as_ref()
      .and_then(|members| members.ids.clone())
      .unwrap_or_default()
  }

  /// Get HSM group members
  pub fn get_members_opt(&self) -> Option<Vec<String>> {
    // FIXME: try to improve this logic by introducing "smart pointers" or "lifetimes"
    self
      .members
      .as_ref()
      .and_then(|members| members.ids.clone())
  }

  /// Add list of xnames to HSM group members
  pub fn add_xnames(&mut self, xnames: &[String]) -> Vec<String> {
    self.members.as_mut().and_then(|members| {
      members
        .ids
        .as_mut()
        .map(|ids| ids.extend_from_slice(xnames))
    });

    self.get_members()
  }
}

impl From<FrontEndGroup> for Group {
  fn from(value: FrontEndGroup) -> Self {
    let mut member_vec = Vec::new();
    let member_vec_backend = value.get_members();

    for member in member_vec_backend {
      member_vec.push(member);
    }

    let members = Members {
      ids: Some(member_vec),
    };

    Group {
      label: value.label,
      description: value.description,
      tags: value.tags,
      members: Some(members),
      exclusive_group: value.exclusive_group,
    }
  }
}

impl From<Group> for FrontEndGroup {
  fn from(val: Group) -> Self {
    let mut member_vec = Vec::new();
    let member_vec_backend = val.get_members();

    for member in member_vec_backend {
      member_vec.push(member);
    }

    let members = FrontEndMember {
      ids: Some(member_vec),
    };

    FrontEndGroup {
      label: val.label,
      description: val.description,
      tags: val.tags,
      members: Some(members),
      exclusive_group: val.exclusive_group,
    }
  }
}
