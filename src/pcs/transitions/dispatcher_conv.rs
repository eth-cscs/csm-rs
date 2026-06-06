//! Bidirectional `From` impls between csm-rs's PCS transitions types
//! and the dispatcher's mirrors. Gated behind the `manta-dispatcher`
//! Cargo feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::pcs::transitions::types::{
  Location as FrontEndLocation, Operation as FrontEndOperation,
  Task as FrontEndTask, TaskCounts as FrontEndTaskCounts,
  Transition as FrontEndTransition,
  TransitionResponse as FrontEndTransitionResponse,
  TransitionStartOutput as FrontEndTransitionStartOutput,
};

use super::types::{
  Location, Operation, Task, TaskCounts, Transition, TransitionResponse,
  TransitionResponseList, TransitionStartOutput,
};

impl From<FrontEndLocation> for Location {
  fn from(value: FrontEndLocation) -> Self {
    Location {
      xname: value.xname,
      deputy_key: value.deputy_key,
    }
  }
}
impl From<Location> for FrontEndLocation {
  fn from(val: Location) -> Self {
    FrontEndLocation {
      xname: val.xname,
      deputy_key: val.deputy_key,
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
impl From<Operation> for FrontEndOperation {
  fn from(val: Operation) -> Self {
    match val {
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

impl From<FrontEndTransition> for Transition {
  fn from(value: FrontEndTransition) -> Self {
    Transition {
      operation: Operation::from(value.operation),
      task_deadline_minutes: value.task_deadline_minutes,
      location: value.location.into_iter().map(Location::from).collect(),
    }
  }
}
impl From<Transition> for FrontEndTransition {
  fn from(val: Transition) -> Self {
    FrontEndTransition {
      operation: val.operation.into(),
      task_deadline_minutes: val.task_deadline_minutes,
      location: val.location.into_iter().map(std::convert::Into::into).collect(),
    }
  }
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
impl From<TaskCounts> for FrontEndTaskCounts {
  fn from(val: TaskCounts) -> Self {
    FrontEndTaskCounts {
      total: val.total,
      new: val.new,
      in_progress: val.in_progress,
      failed: val.failed,
      succeeded: val.succeeded,
      un_supported: val.un_supported,
    }
  }
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
impl From<Task> for FrontEndTask {
  fn from(val: Task) -> Self {
    FrontEndTask {
      xname: val.xname,
      task_status: val.task_status,
      task_status_description: val.task_status_description,
      error: val.error,
    }
  }
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
      tasks: value.tasks.into_iter().map(Task::from).collect(),
    }
  }
}
impl From<TransitionResponse> for FrontEndTransitionResponse {
  fn from(val: TransitionResponse) -> Self {
    FrontEndTransitionResponse {
      transition_id: val.transition_id,
      create_time: val.create_time,
      automatic_expiration_time: val.automatic_expiration_time,
      transition_status: val.transition_status,
      operation: val.operation.into(),
      task_counts: val.task_counts.into(),
      tasks: val.tasks.into_iter().map(std::convert::Into::into).collect(),
    }
  }
}

impl From<Vec<FrontEndTransitionResponse>> for TransitionResponseList {
  fn from(value: Vec<FrontEndTransitionResponse>) -> Self {
    TransitionResponseList {
      transitions: value.into_iter().map(TransitionResponse::from).collect(),
    }
  }
}
impl From<TransitionResponseList> for Vec<FrontEndTransitionResponse> {
  fn from(val: TransitionResponseList) -> Self {
    val.transitions.into_iter().map(std::convert::Into::into).collect()
  }
}

impl From<FrontEndTransitionStartOutput> for TransitionStartOutput {
  fn from(value: FrontEndTransitionStartOutput) -> Self {
    TransitionStartOutput {
      transition_id: value.transition_id,
      operation: Operation::from(value.operation),
    }
  }
}
impl From<TransitionStartOutput> for FrontEndTransitionStartOutput {
  fn from(val: TransitionStartOutput) -> Self {
    FrontEndTransitionStartOutput {
      transition_id: val.transition_id,
      operation: val.operation.into(),
    }
  }
}
