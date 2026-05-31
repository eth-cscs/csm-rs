//! Wire-format types — mirror the upstream CSM OpenAPI schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct BosSession {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tenant: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub operation: Option<Operation>,
  pub template_name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub limit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stage: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub components: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub include_disabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<Status>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Operation {
  #[serde(rename = "boot")]
  Boot,
  #[serde(rename = "reboot")]
  Reboot,
  #[serde(rename = "shutdown")]
  Shutdown,
}

impl std::fmt::Display for Operation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = match self {
      Operation::Boot => "boot",
      Operation::Reboot => "reboot",
      Operation::Shutdown => "shutdown",
    };
    f.write_str(s)
  }
}

impl std::str::FromStr for Operation {
  type Err = Error;

  fn from_str(operation: &str) -> Result<Self, Self::Err> {
    match operation {
      "boot" => Ok(Operation::Boot),
      "reboot" => Ok(Operation::Reboot),
      "shutdown" => Ok(Operation::Shutdown),
      _ => Err(Error::Message("Operation not valid".to_string())),
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
  pub start_time: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_time: Option<String>,
  pub status: StatusLabel,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StatusLabel {
  #[serde(rename = "pending")]
  Pending,
  #[serde(rename = "running")]
  Running,
  #[serde(rename = "complete")]
  Complete,
}

