//! Wire-format types for CAPMC requests and responses.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

/// Per-node failure entry on a power-action response. CAPMC sets `e` to
/// non-zero and populates `err_msg`/`xname` when a particular node
/// could not be transitioned.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct XnameError {
  pub e: i32,
  pub err_msg: String,
  pub xname: String,
}

/// Response shape for CAPMC `xname_on` / `xname_off` / `xname_reinit`.
/// `e == 0` means the call as a whole succeeded; per-node failures (if
/// any) are listed in `xnames`. Both `e` and `err_msg` are nominally
/// required per swagger but CAPMC servers do sometimes omit `err_msg`
/// when empty, so we tolerate the absence via `#[serde(default)]`.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct XnamePowerActionResponse {
  #[serde(default)]
  pub e: i32,
  #[serde(default)]
  pub err_msg: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub xnames: Option<Vec<XnameError>>,
}

/// Response shape for CAPMC `get_xname_status`. The status arrays are
/// each optional — CAPMC omits an array entirely when no xname is in
/// that state (rather than returning an empty array), so consumers
/// should treat absence as "empty". `err_msg` likewise may be omitted
/// when empty.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct XnameStatusResponse {
  #[serde(default)]
  pub e: i32,
  #[serde(default)]
  pub err_msg: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub on: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub off: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub disabled: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ready: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub standby: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub halt: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub undefined: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PowerStatus {
  #[serde(skip_serializing_if = "Option::is_none")]
  reason: Option<String>,
  xnames: Vec<String>,
  force: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  recursive: Option<bool>,
}

impl PowerStatus {
  pub fn new(
    reason: Option<String>,
    xnames: Vec<String>,
    force: bool,
    recursive: Option<bool>,
  ) -> Self {
    Self {
      reason,
      xnames,
      force,
      recursive,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NodeStatus {
  #[serde(skip_serializing_if = "Option::is_none")]
  filter: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  source: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  xnames: Option<Vec<String>>,
}

impl NodeStatus {
  pub fn new(
    filter: Option<String>,
    xnames: Option<Vec<String>>,
    source: Option<String>,
  ) -> Self {
    Self {
      filter,
      source,
      xnames,
    }
  }
}
