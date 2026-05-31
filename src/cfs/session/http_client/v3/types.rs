//! Wire-format types — mirror the upstream CSM OpenAPI schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CfsSessionGetResponseList {
  pub sessions: Vec<CfsSessionGetResponse>,
  pub next: Option<Next>,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // TODO: investigate why serde can Deserialize dynamically syzed structs `Vec<Layer>`
pub struct Next {
  pub limit: Option<u8>,
  pub after_id: Option<String>,
  pub in_use: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configuration {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub limit: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
  pub name: String,
  pub members: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageMap {
  pub source_id: String,
  pub result_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Target {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub definition: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub groups: Option<Vec<Group>>,
  pub image_map: Option<Vec<ImageMap>>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Status {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub artifacts: Option<Vec<Artifact>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub session: Option<Session>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CfsSessionGetResponse {
  pub name: String,
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
  pub debug_on_failure: bool,
  pub logs: Option<String>,
}

impl CfsSessionGetResponse {
  /// Get start time
  pub fn get_start_time(&self) -> Option<String> {
    self
      .status
      .as_ref()
      .and_then(|status| status.session.as_ref())
      .and_then(|session| session.start_time.clone())
  }

  /// Returns list of result_ids
  pub fn get_result_id_vec(&self) -> Vec<String> {
    self
      .status
      .as_ref()
      .map(|status| {
        status
          .artifacts
          .clone()
          .unwrap_or_default()
          .iter()
          .filter_map(|artifact| artifact.result_id.clone())
          .collect::<Vec<String>>()
      })
      .unwrap_or_default()
  }

  /// Returns list of result_ids
  pub fn get_first_result_id(&self) -> Option<String> {
    CfsSessionGetResponse::get_result_id_vec(self)
      .first()
      .cloned()
  }

  /// Returns list of targets (either groups or xnames)
  pub fn get_targets(&self) -> Option<Vec<String>> {
    self.get_target_hsm().or_else(|| self.get_target_xname())
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

  pub fn get_configuration_name(&self) -> Option<String> {
    self
      .configuration
      .as_ref()
      .and_then(|configuration| configuration.name.clone())
  }

  /// Returns 'true' if CFS session succeeded
  pub fn is_success(&self) -> bool {
    self.status.as_ref().is_some_and(|status| {
      status.session.as_ref().is_some_and(|session| {
        session
          .succeeded
          .as_ref()
          .is_some_and(|succeeded| succeeded.as_str() == "true")
      })
    })
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub job: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ims_job: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub completion_time: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub start_time: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub succeeded: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CfsSessionPostRequest {
  pub name: String,
  pub configuration_name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub configuration_limit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ansible_limit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ansible_config: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ansible_verbosity: Option<u8>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ansible_passthrough: Option<String>,
  #[serde(default)]
  pub target: Target,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tags: Option<HashMap<String, String>>,
  pub debug_on_failure: bool,
}

impl CfsSessionPostRequest {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    name: String,
    configuration_name: String,
    configuration_limit_opt: Option<String>,
    ansible_limit_opt: Option<String>,
    ansible_config_opt: Option<String>,
    ansible_verbosity_opt: Option<u8>,
    ansible_passthrough_opt: Option<String>,
    is_target_definition_image: bool,
    groups_name_opt: Option<Vec<String>>,
    base_image_id_opt: Option<String>,
    tags_opt: Option<HashMap<String, String>>,
    debug_on_failure: bool,
    result_image_name_opt: Option<String>,
  ) -> Result<Self, Error> {
    // This code is fine... the fact that I put Self behind a variable is ok, since image param
    // is not a default param, then doing things differently is not an issue. I checked with
    // other Rust developers in their discord https://discord.com/channels/442252698964721669/448238009733742612/1081686300182188207
    let mut cfs_session = Self {
      name,
      configuration_name,
      configuration_limit: configuration_limit_opt,
      ansible_config: ansible_config_opt,
      ansible_limit: ansible_limit_opt,
      ansible_verbosity: ansible_verbosity_opt,
      ansible_passthrough: ansible_passthrough_opt,
      ..Default::default()
    };

    if is_target_definition_image {
      // Validation
      let base_image_id = base_image_id_opt.ok_or_else(|| {
        Error::Message(
          "Can't create a CFS session to build an image without base image id"
            .to_string(),
        )
      })?;

      let result_image_name = result_image_name_opt.ok_or_else(|| {
        Error::Message("Can't create a CFS sessions to build an image without result image name".to_string())
      })?;
      // End validation

      let target_groups: Vec<Group> = groups_name_opt
        .map(|group_vec| {
          group_vec
            .into_iter()
            .map(|group_name| Group {
              name: group_name,
              members: vec![base_image_id.clone()],
            })
            .collect::<Vec<Group>>()
        })
        .unwrap_or_default();

      cfs_session.target.definition = Some("image".to_string());
      cfs_session.target.groups = Some(target_groups);
      cfs_session.target.image_map = Some(vec![ImageMap {
        source_id: base_image_id,
        result_name: result_image_name,
      }]);
    } else {
      cfs_session.target.definition = Some("dynamic".to_string());
      cfs_session.target.groups = None;
      cfs_session.target.image_map = Some(Vec::new());
    }

    cfs_session.tags = tags_opt;
    cfs_session.debug_on_failure = debug_on_failure;

    Ok(cfs_session)
  }
}
