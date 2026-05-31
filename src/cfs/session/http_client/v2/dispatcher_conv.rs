//! Bidirectional `From` impls between csm-rs's CFS v2 session types and
//! the dispatcher's mirrors. Gated behind the `manta-dispatcher` Cargo
//! feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::cfs::session::{
  Ansible as FrontEndAnsible, Artifact as FrontEndArtifact,
  CfsSessionGetResponse as FrontEndCfsSessionGetResponse,
  Configuration as FrontEndConfiguration, Group as FrontEndGroup,
  Session as FrontEndSession, Status as FrontEndStatus,
  Target as FrontEndTarget,
};

use super::types::{
  Ansible, Artifact, CfsSessionGetResponse, Configuration, Group, Session,
  Status, Target,
};

impl From<FrontEndCfsSessionGetResponse> for CfsSessionGetResponse {
  fn from(value: FrontEndCfsSessionGetResponse) -> Self {
    CfsSessionGetResponse {
      name: value.name,
      configuration: value.configuration.map(Configuration::from),
      ansible: value.ansible.map(Ansible::from),
      target: value.target.map(Target::from),
      status: value.status.map(Status::from),
      tags: value.tags,
    }
  }
}

impl From<CfsSessionGetResponse> for FrontEndCfsSessionGetResponse {
  fn from(val: CfsSessionGetResponse) -> Self {
    FrontEndCfsSessionGetResponse {
      name: val.name,
      configuration: val
        .configuration
        .map(|configuration| configuration.into()),
      ansible: val.ansible.map(|ansible| ansible.into()),
      target: val.target.map(|target| target.into()),
      status: val.status.map(|status| status.into()),
      tags: val.tags,
      debug_on_failure: true,
      logs: None,
    }
  }
}

impl From<FrontEndConfiguration> for Configuration {
  fn from(value: FrontEndConfiguration) -> Self {
    Configuration {
      name: value.name,
      limit: value.limit,
    }
  }
}

impl From<Configuration> for FrontEndConfiguration {
  fn from(val: Configuration) -> Self {
    FrontEndConfiguration {
      name: val.name,
      limit: val.limit,
    }
  }
}

impl From<FrontEndAnsible> for Ansible {
  fn from(value: FrontEndAnsible) -> Self {
    Ansible {
      config: value.config,
      limit: value.limit,
      verbosity: value.verbosity,
      passthrough: value.passthrough,
    }
  }
}

impl From<Ansible> for FrontEndAnsible {
  fn from(val: Ansible) -> Self {
    FrontEndAnsible {
      config: val.config,
      limit: val.limit,
      verbosity: val.verbosity,
      passthrough: val.passthrough,
    }
  }
}

impl From<FrontEndStatus> for Status {
  fn from(value: FrontEndStatus) -> Self {
    Status {
      artifacts: value
        .artifacts
        .map(|artifacts| artifacts.into_iter().map(Artifact::from).collect()),
      session: value.session.map(Session::from),
    }
  }
}

impl From<Status> for FrontEndStatus {
  fn from(val: Status) -> Self {
    FrontEndStatus {
      artifacts: val
        .artifacts
        .map(|artifacts| artifacts.into_iter().map(Artifact::into).collect()),
      session: val.session.map(Session::into),
    }
  }
}

impl From<FrontEndArtifact> for Artifact {
  fn from(value: FrontEndArtifact) -> Self {
    Artifact {
      image_id: value.image_id,
      result_id: value.result_id,
      r#type: value.r#type,
    }
  }
}

impl From<Artifact> for FrontEndArtifact {
  fn from(val: Artifact) -> Self {
    FrontEndArtifact {
      image_id: val.image_id,
      result_id: val.result_id,
      r#type: val.r#type,
    }
  }
}

impl From<FrontEndSession> for Session {
  fn from(value: FrontEndSession) -> Self {
    Session {
      job: value.job,
      completion_time: value.completion_time,
      start_time: value.start_time,
      status: value.status,
      succeeded: value.succeeded,
    }
  }
}

impl From<Session> for FrontEndSession {
  fn from(val: Session) -> Self {
    FrontEndSession {
      job: val.job,
      ims_job: None,
      completion_time: val.completion_time,
      start_time: val.start_time,
      status: val.status,
      succeeded: val.succeeded,
    }
  }
}

impl From<FrontEndGroup> for Group {
  fn from(value: FrontEndGroup) -> Self {
    Group {
      name: value.name,
      members: value.members,
    }
  }
}

impl From<Group> for FrontEndGroup {
  fn from(val: Group) -> Self {
    FrontEndGroup {
      name: val.name,
      members: val.members,
    }
  }
}

impl From<FrontEndTarget> for Target {
  fn from(value: FrontEndTarget) -> Self {
    Target {
      definition: value.definition,
      groups: value
        .groups
        .map(|groups| groups.into_iter().map(Group::from).collect()),
    }
  }
}

impl From<Target> for FrontEndTarget {
  fn from(val: Target) -> Self {
    FrontEndTarget {
      definition: val.definition,
      groups: val
        .groups
        .map(|groups| groups.into_iter().map(Group::into).collect()),
      image_map: None,
    }
  }
}
