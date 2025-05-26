use manta_backend_dispatcher::types::pcs::transitions::types::{
    Operation as FrontEndOperation,
    Location as FrontEndLocation,
    Transition as FrontEndTransition,
};

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

#[derive(Debug, Serialize, Deserialize)]
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
  /* pub fn to_string(&self) -> String {
      match self {
          Operation::On => "on".to_string(),
          Operation::Off => "off".to_string(),
          Operation::SoftOff => "soft-off".to_string(),
          Operation::SoftRestart => "soft-restart".to_string(),
          Operation::HardRestart => "hard-restart".to_string(),
          Operation::Init => "init".to_string(),
          Operation::ForceOff => "force-off".to_string(),
      }
  } */

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

/* impl FromStr for Operation {
    type Err = Error;

    fn from_str(operation: &str) -> Result<Operation, Error> {
        Self::from_str(operation)
    }
} */
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
            location: value.location.into_iter().map(|v| Location::from(v)).collect()
        }
    }
}
impl Into<FrontEndTransition> for Transition {
    fn into(self) -> FrontEndTransition {
        FrontEndTransition {
            operation: self.operation.into(),
            task_deadline_minutes: self.task_deadline_minutes,
            location: self.location.into_iter().map(|v| v.into()).collect()
        }
    }
}

