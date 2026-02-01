use manta_backend_dispatcher::types::pcs::transitions::types::{
  Location as FrontEndLocation, Operation as FrontEndOperation,
  Task as FrontEndTask, TaskCounts as FrontEndTaskCounts,
  Transition as FrontEndTransition,
  TransitionResponse as FrontEndTransitionResponse,
};

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

impl From<FrontEndLocation> for Location {
  fn from(value: FrontEndLocation) -> Self {
    Location {
      xname: value.xname,
      deputy_key: value.deputy_key,
    }
  }
}
impl Into<FrontEndLocation> for Location {
  fn into(self) -> FrontEndLocation {
    FrontEndLocation {
      xname: self.xname,
      deputy_key: self.deputy_key,
    }
  }
}

#[derive(Display, Debug, Serialize, Deserialize)]
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

impl Operation {
  pub fn from_str(operation: &str) -> Result<Operation, Error> {
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

impl From<FrontEndOperation> for Operation {
  fn from(value: FrontEndOperation) -> Self {
    match value {
      FrontEndOperation::On => Operation::On,
      FrontEndOperation::Off => Operation::Off,
      FrontEndOperation::SoftOff => Operation::SoftOff,
      FrontEndOperation::SoftRestart => Operation::SoftRestart,
      FrontEndOperation::HardRestart => Operation::HardRestart,
      FrontEndOperation::Init => Operation::Init,
      FrontEndOperation::ForceOff => Operation::ForceOff,
    }
  }
}
impl Into<FrontEndOperation> for Operation {
  fn into(self) -> FrontEndOperation {
    match self {
      Operation::On => FrontEndOperation::On,
      Operation::Off => FrontEndOperation::Off,
      Operation::SoftOff => FrontEndOperation::SoftOff,
      Operation::SoftRestart => FrontEndOperation::SoftRestart,
      Operation::HardRestart => FrontEndOperation::HardRestart,
      Operation::Init => FrontEndOperation::Init,
      Operation::ForceOff => FrontEndOperation::ForceOff,
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

impl From<FrontEndTransition> for Transition {
  fn from(value: FrontEndTransition) -> Self {
    Transition {
      operation: Operation::from(value.operation),
      task_deadline_minutes: value.task_deadline_minutes,
      location: value
        .location
        .into_iter()
        .map(|v| Location::from(v))
        .collect(),
    }
  }
}
impl Into<FrontEndTransition> for Transition {
  fn into(self) -> FrontEndTransition {
    FrontEndTransition {
      operation: self.operation.into(),
      task_deadline_minutes: self.task_deadline_minutes,
      location: self.location.into_iter().map(|v| v.into()).collect(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskCounts {
  pub total: usize,
  pub new: usize,
  pub in_progress: usize,
  pub failed: usize,
  pub succeeded: usize,
  pub un_supported: usize,
}

impl From<FrontEndTaskCounts> for TaskCounts {
  fn from(value: FrontEndTaskCounts) -> Self {
    TaskCounts {
      total: value.total,
      new: value.new,
      in_progress: value.in_progress,
      failed: value.failed,
      succeeded: value.succeeded,
      un_supported: value.un_supported,
    }
  }
}

impl Into<FrontEndTaskCounts> for TaskCounts {
  fn into(self) -> FrontEndTaskCounts {
    FrontEndTaskCounts {
      total: self.total,
      new: self.new,
      in_progress: self.in_progress,
      failed: self.failed,
      succeeded: self.succeeded,
      un_supported: self.un_supported,
    }
  }
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

impl From<FrontEndTask> for Task {
  fn from(value: FrontEndTask) -> Self {
    Task {
      xname: value.xname,
      task_status: value.task_status,
      task_status_description: value.task_status_description,
      error: value.error,
    }
  }
}

impl Into<manta_backend_dispatcher::types::pcs::transitions::types::Task>
  for Task
{
  fn into(
    self,
  ) -> manta_backend_dispatcher::types::pcs::transitions::types::Task {
    manta_backend_dispatcher::types::pcs::transitions::types::Task {
      xname: self.xname,
      task_status: self.task_status,
      task_status_description: self.task_status_description,
      error: self.error,
    }
  }
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

impl From<FrontEndTransitionResponse> for TransitionResponse {
  fn from(value: FrontEndTransitionResponse) -> Self {
    TransitionResponse {
      transition_id: value.transition_id,
      create_time: value.create_time,
      automatic_expiration_time: value.automatic_expiration_time,
      transition_status: value.transition_status,
      operation: Operation::from(value.operation),
      task_counts: TaskCounts::from(value.task_counts),
      tasks: value.tasks.into_iter().map(|v| Task::from(v)).collect(),
    }
  }
}

impl Into<FrontEndTransitionResponse> for TransitionResponse {
  fn into(self) -> FrontEndTransitionResponse {
    FrontEndTransitionResponse {
      transition_id: self.transition_id,
      create_time: self.create_time,
      automatic_expiration_time: self.automatic_expiration_time,
      transition_status: self.transition_status,
      operation: self.operation.into(),
      task_counts: self.task_counts.into(),
      tasks: self.tasks.into_iter().map(|v| v.into()).collect(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransitionResponseList {
  pub transitions: Vec<TransitionResponse>,
}

impl From<Vec<FrontEndTransitionResponse>> for TransitionResponseList {
  fn from(value: Vec<FrontEndTransitionResponse>) -> Self {
    TransitionResponseList {
      transitions: value
        .into_iter()
        .map(|v| TransitionResponse::from(v))
        .collect(),
    }
  }
}

impl Into<Vec<FrontEndTransitionResponse>> for TransitionResponseList {
  fn into(self) -> Vec<FrontEndTransitionResponse> {
    self.transitions.into_iter().map(|v| v.into()).collect()
  }
}
