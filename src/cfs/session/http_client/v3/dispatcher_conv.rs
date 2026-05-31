//! Bidirectional `From` impls between csm-rs's CFS v3 session types and
//! the dispatcher's mirrors. Gated behind the `manta-dispatcher` Cargo
//! feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::cfs::session::{
  Ansible as FrontEndAnsible, Artifact as FrontEndArtifact,
  CfsSessionGetResponse as FrontEndCfsSessionGetResponse,
  CfsSessionGetResponseList as FrontEndCfsSessionGetResponseList,
  CfsSessionPostRequest as FrontEndCfsSessionPostRequest,
  Configuration as FrontEndConfiguration, Group as FrontEndGroup,
  ImageMap as FrontEndImageMap, Next as FrontEndNext,
  Session as FrontEndSession, Status as FrontEndStatus,
  Target as FrontEndTarget,
};

use super::types::{
  Ansible, Artifact, CfsSessionGetResponse, CfsSessionGetResponseList,
  CfsSessionPostRequest, Configuration, Group, ImageMap, Next, Session, Status,
  Target,
};

impl From<FrontEndCfsSessionGetResponseList> for CfsSessionGetResponseList {
  fn from(value: FrontEndCfsSessionGetResponseList) -> Self {
    CfsSessionGetResponseList {
      sessions: value
        .sessions
        .into_iter()
        .map(CfsSessionGetResponse::from)
        .collect(),
      next: value.next.map(Next::from),
    }
  }
}

impl From<CfsSessionGetResponseList> for FrontEndCfsSessionGetResponseList {
  fn from(val: CfsSessionGetResponseList) -> Self {
    FrontEndCfsSessionGetResponseList {
      sessions: val
        .sessions
        .into_iter()
        .map(|session| session.into())
        .collect(),
      next: val.next.map(|next| next.into()),
    }
  }
}

impl From<FrontEndNext> for Next {
  fn from(value: FrontEndNext) -> Self {
    Next {
      limit: value.limit,
      after_id: value.after_id,
      in_use: value.in_use,
    }
  }
}

impl From<Next> for FrontEndNext {
  fn from(val: Next) -> Self {
    FrontEndNext {
      limit: val.limit,
      after_id: val.after_id,
      in_use: val.in_use,
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

impl From<FrontEndImageMap> for ImageMap {
  fn from(value: FrontEndImageMap) -> Self {
    ImageMap {
      source_id: value.source_id,
      result_name: value.result_name,
    }
  }
}

impl From<ImageMap> for FrontEndImageMap {
  fn from(val: ImageMap) -> Self {
    FrontEndImageMap {
      source_id: val.source_id,
      result_name: val.result_name,
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
      image_map: value
        .image_map
        .map(|image_map| image_map.into_iter().map(ImageMap::from).collect()),
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
      image_map: val
        .image_map
        .map(|image_map| image_map.into_iter().map(ImageMap::into).collect()),
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

impl From<FrontEndCfsSessionGetResponse> for CfsSessionGetResponse {
  fn from(value: FrontEndCfsSessionGetResponse) -> Self {
    CfsSessionGetResponse {
      name: value.name,
      configuration: value.configuration.map(Configuration::from),
      ansible: value.ansible.map(Ansible::from),
      target: value.target.map(Target::from),
      status: value.status.map(Status::from),
      tags: value.tags,
      debug_on_failure: value.debug_on_failure,
      logs: value.logs,
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
      debug_on_failure: val.debug_on_failure,
      logs: val.logs,
    }
  }
}

impl From<FrontEndSession> for Session {
  fn from(value: FrontEndSession) -> Self {
    Session {
      job: value.job,
      ims_job: value.ims_job,
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
      ims_job: val.ims_job,
      completion_time: val.completion_time,
      start_time: val.start_time,
      status: val.status,
      succeeded: val.succeeded,
    }
  }
}

impl From<FrontEndCfsSessionPostRequest> for CfsSessionPostRequest {
  fn from(value: FrontEndCfsSessionPostRequest) -> Self {
    CfsSessionPostRequest {
      name: value.name,
      configuration_name: value.configuration_name,
      configuration_limit: value.configuration_limit,
      ansible_limit: value.ansible_limit,
      ansible_config: value.ansible_config,
      ansible_verbosity: value.ansible_verbosity,
      ansible_passthrough: value.ansible_passthrough,
      target: value.target.into(),
      tags: value.tags,
      debug_on_failure: value.debug_on_failure,
    }
  }
}

impl From<CfsSessionPostRequest> for FrontEndCfsSessionPostRequest {
  fn from(val: CfsSessionPostRequest) -> Self {
    FrontEndCfsSessionPostRequest {
      name: val.name,
      configuration_name: val.configuration_name,
      configuration_limit: val.configuration_limit,
      ansible_limit: val.ansible_limit,
      ansible_config: val.ansible_config,
      ansible_verbosity: val.ansible_verbosity,
      ansible_passthrough: val.ansible_passthrough,
      target: val.target.into(),
      tags: val.tags,
      debug_on_failure: val.debug_on_failure,
    }
  }
}
