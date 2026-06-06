//! Wire-format types — mirror the upstream CSM OpenAPI schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

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
  #[serde(rename = "exclusiveGroup")]
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

  /// Get HSM group members.
  ///
  /// Returns owned `Vec<String>` rather than `&[String]` because most
  /// callers immediately collect/filter/clone the IDs anyway and
  /// switching to a borrow would ripple through ~10 sites. A
  /// borrow-returning variant can be added later if it becomes a hot
  /// path; mutations should go through `Group::members.ids` directly,
  /// not through `get_members()` (see the bug fix in
  /// `crate::hsm::group::utils::add_member` for the prior pitfall).
  pub fn get_members(&self) -> Vec<String> {
    self
      .members
      .as_ref()
      .and_then(|members| members.ids.clone())
      .unwrap_or_default()
  }

  /// Get HSM group members, distinguishing "no members" (`Some(empty)`)
  /// from "missing field" (`None`). See [`Group::get_members`] for the
  /// rationale on returning owned `Vec` here.
  pub fn get_members_opt(&self) -> Option<Vec<String>> {
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

