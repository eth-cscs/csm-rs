//! Bidirectional `From` impls between csm-rs's BOS v2 session types and
//! the dispatcher's mirrors. Gated behind the `manta-dispatcher` Cargo
//! feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::bos::session::{
  BosSession as FrontEndBosSession, Operation as FrontEndOperation,
  Status as FrontEndStatus, StatusLabel as FrontEndStatusLabel,
};

use super::types::{BosSession, Operation, Status, StatusLabel};

impl From<FrontEndBosSession> for BosSession {
  fn from(frontend_bos_session: FrontEndBosSession) -> Self {
    Self {
      name: frontend_bos_session.name,
      tenant: frontend_bos_session.tenant,
      operation: frontend_bos_session
        .operation
        .map(|operation| operation.into()),
      template_name: frontend_bos_session.template_name,
      limit: frontend_bos_session.limit,
      stage: frontend_bos_session.stage,
      components: frontend_bos_session.components,
      include_disabled: frontend_bos_session.include_disabled,
      status: frontend_bos_session.status.map(|status| status.into()),
    }
  }
}

impl From<BosSession> for FrontEndBosSession {
  fn from(val: BosSession) -> Self {
    FrontEndBosSession {
      name: val.name,
      tenant: val.tenant,
      operation: val.operation.map(|operation| operation.into()),
      template_name: val.template_name,
      limit: val.limit,
      stage: val.stage,
      components: val.components,
      include_disabled: val.include_disabled,
      status: val.status.map(|status| status.into()),
    }
  }
}

impl From<FrontEndOperation> for Operation {
  fn from(frontend_operation: FrontEndOperation) -> Self {
    match frontend_operation {
      FrontEndOperation::Boot => Self::Boot,
      FrontEndOperation::Reboot => Self::Reboot,
      FrontEndOperation::Shutdown => Self::Shutdown,
    }
  }
}

impl From<Operation> for FrontEndOperation {
  fn from(val: Operation) -> Self {
    match val {
      Operation::Boot => FrontEndOperation::Boot,
      Operation::Reboot => FrontEndOperation::Reboot,
      Operation::Shutdown => FrontEndOperation::Shutdown,
    }
  }
}

impl From<FrontEndStatus> for Status {
  fn from(frontend_status: FrontEndStatus) -> Self {
    Self {
      start_time: frontend_status.start_time,
      end_time: frontend_status.end_time,
      status: frontend_status.status.into(),
      error: frontend_status.error,
    }
  }
}

impl From<Status> for FrontEndStatus {
  fn from(val: Status) -> Self {
    FrontEndStatus {
      start_time: val.start_time,
      end_time: val.end_time,
      status: val.status.into(),
      error: val.error,
    }
  }
}

impl From<FrontEndStatusLabel> for StatusLabel {
  fn from(frontend_status_label: FrontEndStatusLabel) -> Self {
    match frontend_status_label {
      FrontEndStatusLabel::Pending => Self::Pending,
      FrontEndStatusLabel::Running => Self::Running,
      FrontEndStatusLabel::Complete => Self::Complete,
    }
  }
}

impl From<StatusLabel> for FrontEndStatusLabel {
  fn from(val: StatusLabel) -> Self {
    match val {
      StatusLabel::Pending => FrontEndStatusLabel::Pending,
      StatusLabel::Running => FrontEndStatusLabel::Running,
      StatusLabel::Complete => FrontEndStatusLabel::Complete,
    }
  }
}
