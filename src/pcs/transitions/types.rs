//! Wire-format types — mirror the upstream CSM OpenAPI schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

use strum_macros::Display;

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Location {
  pub xname: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "deputyKey")]
  pub deputy_key: Option<String>,
}

#[derive(Display, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Operation {
  #[serde(rename = "On")]
  On,
  #[serde(rename = "Off")]
  Off,
  #[serde(rename = "Soft-Off")]
  SoftOff,
  #[serde(rename = "Soft-Restart")]
  SoftRestart,
  #[serde(rename = "Hard-Restart")]
  HardRestart,
  #[serde(rename = "Init")]
  Init,
  #[serde(rename = "Force-Off")]
  ForceOff,
}

impl std::str::FromStr for Operation {
  type Err = Error;

  fn from_str(operation: &str) -> Result<Self, Self::Err> {
    match operation {
      "on" => Ok(Operation::On),
      "off" => Ok(Operation::Off),
      "soft-off" => Ok(Operation::SoftOff),
      "soft-restart" => Ok(Operation::SoftRestart),
      "hard-restart" => Ok(Operation::HardRestart),
      "init" => Ok(Operation::Init),
      "force-off" => Ok(Operation::ForceOff),
      _ => Err(Error::Message("Operation not valid".to_string())),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transition {
  pub operation: Operation,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "taskDeadlineMinutes")]
  pub task_deadline_minutes: Option<usize>,
  pub location: Vec<Location>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskCounts {
  pub total: usize,
  pub new: usize,
  #[serde(rename = "in-progress")]
  pub in_progress: usize,
  pub failed: usize,
  pub succeeded: usize,
  #[serde(rename = "un-supported")]
  pub un_supported: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
  pub xname: String,
  #[serde(rename = "taskStatus")]
  pub task_status: String,
  #[serde(rename = "taskStatusDescription")]
  pub task_status_description: String,
  pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransitionResponse {
  #[serde(rename = "transitionID")]
  pub transition_id: String,
  #[serde(rename = "createTime")]
  pub create_time: String,
  #[serde(rename = "automaticExpirationTime")]
  pub automatic_expiration_time: String,
  #[serde(rename = "transitionStatus")]
  pub transition_status: String,
  pub operation: Operation,
  #[serde(rename = "taskCounts")]
  pub task_counts: TaskCounts,
  pub tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransitionResponseList {
  pub transitions: Vec<TransitionResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransitionStartOutput {
  #[serde(rename = "transitionID")]
  pub transition_id: String,
  pub operation: Operation,
}

