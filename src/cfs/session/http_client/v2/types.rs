use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use manta_backend_dispatcher::types::cfs::session::{
  Ansible as FrontEndAnsible, Artifact as FrontEndArtifact,
  CfsSessionGetResponse as FrontEndCfsSessionGetResponse,
  Configuration as FrontEndConfiguration, Group as FrontEndGroup,
  Session as FrontEndSession, Status as FrontEndStatus,
  Target as FrontEndTarget,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CfsSessionGetResponse {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub configuration: Option<Configuration>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ansible: Option<Ansible>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub target: Option<Target>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<Status>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tags: Option<HashMap<String, String>>,
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
    }
  }
}

impl Into<FrontEndCfsSessionGetResponse> for CfsSessionGetResponse {
  fn into(self) -> FrontEndCfsSessionGetResponse {
    FrontEndCfsSessionGetResponse {
      name: self.name,
      configuration: self
        .configuration
        .map(|configuration| configuration.into()),
      ansible: self.ansible.map(|ansible| ansible.into()),
      target: self.target.map(|target| target.into()),
      status: self.status.map(|status| status.into()),
      tags: self.tags,
      debug_on_failure: true,
      logs: None,
    }
  }
}

impl CfsSessionGetResponse {
  pub fn name(&self) -> Option<&str> {
    self.name.as_deref()
  }

  /// Get start time
  pub fn get_start_time(&self) -> Option<String> {
    self.status.as_ref().and_then(|status| {
      status
        .session
        .as_ref()
        .and_then(|session| session.start_time.clone())
    })
  }

  /// Returns list of result_ids
  #[deprecated(note = "Use results_id instead")]
  pub fn get_result_id_vec(&self) -> Vec<String> {
    if let Some(status) = &self.status {
      status
        .artifacts
        .as_ref()
        .unwrap_or(&Vec::new())
        .into_iter()
        .filter(|artifact| artifact.result_id.is_some())
        .map(|artifact| artifact.result_id.clone().unwrap())
        .collect()
    } else {
      Vec::new()
    }
  }

  /// Returns list of result_ids
  pub fn results_id(&self) -> impl Iterator<Item = &str> {
    self.status.iter().flat_map(|status| {
      status
        .artifacts
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter_map(|artifact| artifact.result_id.as_deref())
    })
  }

  /// Returns list of result_ids
  #[deprecated(note = "Use first_result_id instead")]
  pub fn get_first_result_id(&self) -> Option<String> {
    CfsSessionGetResponse::get_result_id_vec(&self)
      .first()
      .cloned()
  }

  /// Returns list of result_ids
  pub fn first_result_id(&self) -> Option<&str> {
    CfsSessionGetResponse::results_id(&self).next()
  }

  /// Returns list of HSM groups targeted
  pub fn get_target_hsm(&self) -> Option<Vec<String>> {
    self.target.as_ref().and_then(|target| {
      target.groups.as_ref().map(|group_vec| {
        group_vec.iter().map(|group| group.name.clone()).collect()
      })
    })
  }

  /// Returns list of xnames targeted
  pub fn get_target_xname(&self) -> Option<Vec<String>> {
    self.ansible.as_ref().and_then(|ansible| {
      ansible.limit.as_ref().map(|limit| {
        limit
          .split(',')
          .map(|xname| xname.trim().to_string())
          .collect()
      })
    })
  }

  /// Returns 'true' if the CFS session target definition is 'image'. Otherwise (target
  /// definiton dynamic) will return 'false'
  pub fn is_target_def_image(&self) -> bool {
    self
      .get_target_def()
      .is_some_and(|target_def| target_def == "image")
  }

  /// Returns target definition of the CFS session:
  /// image --> CFS session to build an image
  /// dynamic --> CFS session to configure a node
  pub fn get_target_def(&self) -> Option<String> {
    self
      .target
      .as_ref()
      .and_then(|target| target.definition.clone())
  }

  pub fn configuration_namen(&self) -> Option<&str> {
    self
      .configuration
      .as_ref()
      .and_then(|configuration| configuration.name.as_deref())
  }

  pub fn is_success(&self) -> bool {
    self
      .status
      .as_ref()
      .unwrap()
      .session
      .as_ref()
      .unwrap()
      .succeeded
      .as_ref()
      .unwrap()
      == "true"
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configuration {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub limit: Option<String>,
}

impl From<FrontEndConfiguration> for Configuration {
  fn from(value: FrontEndConfiguration) -> Self {
    Configuration {
      name: value.name,
      limit: value.limit,
    }
  }
}

impl Into<FrontEndConfiguration> for Configuration {
  fn into(self) -> FrontEndConfiguration {
    FrontEndConfiguration {
      name: self.name,
      limit: self.limit,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ansible {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub config: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub limit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub verbosity: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub passthrough: Option<String>,
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

impl Into<FrontEndAnsible> for Ansible {
  fn into(self) -> FrontEndAnsible {
    FrontEndAnsible {
      config: self.config,
      limit: self.limit,
      verbosity: self.verbosity,
      passthrough: self.passthrough,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Status {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub artifacts: Option<Vec<Artifact>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub session: Option<Session>,
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

impl Into<FrontEndStatus> for Status {
  fn into(self) -> FrontEndStatus {
    FrontEndStatus {
      artifacts: self
        .artifacts
        .map(|artifacts| artifacts.into_iter().map(Artifact::into).collect()),
      session: self.session.map(Session::into),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artifact {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub image_id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub result_id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
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

impl Into<FrontEndArtifact> for Artifact {
  fn into(self) -> FrontEndArtifact {
    FrontEndArtifact {
      image_id: self.image_id,
      result_id: self.result_id,
      r#type: self.r#type,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub job: Option<String>,
  #[serde(rename = "completionTime")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub completion_time: Option<String>,
  #[serde(rename = "startTime")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub start_time: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub succeeded: Option<String>,
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

impl Into<FrontEndSession> for Session {
  fn into(self) -> FrontEndSession {
    FrontEndSession {
      job: self.job,
      ims_job: None,
      completion_time: self.completion_time,
      start_time: self.start_time,
      status: self.status,
      succeeded: self.succeeded,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CfsSessionPostRequest {
  pub name: String,
  #[serde(rename = "configurationName")]
  pub configuration_name: String,
  #[serde(rename = "configurationLimit")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub configuration_limit: Option<String>,
  #[serde(rename = "ansibleLimit")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ansible_limit: Option<String>,
  #[serde(rename = "ansibleConfig")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ansible_config: Option<String>,
  #[serde(rename = "ansibleVerbosity")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ansible_verbosity: Option<u8>,
  #[serde(rename = "ansiblePassthrough")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ansible_passthrough: Option<String>,
  #[serde(default)]
  pub target: Target,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tags: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
  pub name: String,
  pub members: Vec<String>,
}

impl From<FrontEndGroup> for Group {
  fn from(value: FrontEndGroup) -> Self {
    Group {
      name: value.name,
      members: value.members,
    }
  }
}

impl Into<FrontEndGroup> for Group {
  fn into(self) -> FrontEndGroup {
    FrontEndGroup {
      name: self.name,
      members: self.members,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Target {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub definition: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub groups: Option<Vec<Group>>,
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

impl Into<FrontEndTarget> for Target {
  fn into(self) -> FrontEndTarget {
    FrontEndTarget {
      definition: self.definition,
      groups: self
        .groups
        .map(|groups| groups.into_iter().map(Group::into).collect()),
      image_map: None,
    }
  }
}

impl CfsSessionPostRequest {
  pub fn new(
    name: String,
    configuration_name: String,
    ansible_limit: Option<String>,
    ansible_verbosity: Option<u8>,
    ansible_passthrough: Option<String>,
    is_target_definition_image: bool,
    groups_name: Option<Vec<String>>,
    base_image_id: Option<String>,
  ) -> Self {
    // This code is fine... the fact that I put Self behind a variable is ok, since image param
    // is not a default param, then doing things differently is not an issue. I checked with
    // other Rust developers in their discord https://discord.com/channels/442252698964721669/448238009733742612/1081686300182188207
    let mut cfs_session = Self {
      name,
      configuration_name,
      ansible_limit,
      ansible_verbosity,
      ansible_passthrough,
      ..Default::default()
    };

    if is_target_definition_image {
      let target_groups: Vec<Group> = groups_name
        .unwrap()
        .into_iter()
        .map(|group_name| Group {
          name: group_name,
          members: vec![base_image_id.as_ref().unwrap().to_string()],
        })
        .collect();

      cfs_session.target.definition = Some("image".to_string());
      cfs_session.target.groups = Some(target_groups);
    }

    cfs_session
  }
}
