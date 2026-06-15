//! Wire-format types — mirror the upstream CSM `OpenAPI` schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
}

impl CfsSessionGetResponse {
  #[must_use]
  pub fn configuration(&self) -> Option<&Configuration> {
    self.configuration.as_ref()
  }

  #[must_use]
  pub fn tags(&self) -> Option<&HashMap<String, String>> {
    self.tags.as_ref()
  }
}

impl CfsSessionGetResponse {
  /// Get start time
  #[must_use]
  pub fn get_start_time(&self) -> Option<String> {
    self.status.as_ref().and_then(|status| {
      status
        .session
        .as_ref()
        .and_then(|session| session.start_time.clone())
    })
  }

  /// Returns list of `result_ids`
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

  /// Returns list of `result_ids`
  #[must_use]
  pub fn first_result_id(&self) -> Option<&str> {
    CfsSessionGetResponse::results_id(self).next()
  }

  /// Returns list of HSM groups targeted
  #[must_use]
  pub fn get_target_hsm(&self) -> Option<Vec<String>> {
    self.target.as_ref().and_then(|target| {
      target.groups.as_ref().map(|group_vec| {
        group_vec.iter().map(|group| group.name.clone()).collect()
      })
    })
  }

  /// Returns list of xnames targeted
  #[must_use]
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
  #[must_use]
  pub fn is_target_def_image(&self) -> bool {
    self
      .get_target_def()
      .is_some_and(|target_def| target_def == "image")
  }

  /// Returns target definition of the CFS session:
  /// image --> CFS session to build an image
  /// dynamic --> CFS session to configure a node
  #[must_use]
  pub fn get_target_def(&self) -> Option<String> {
    self
      .target
      .as_ref()
      .and_then(|target| target.definition.clone())
  }

  #[must_use]
  pub fn configuration_name(&self) -> Option<&str> {
    self
      .configuration
      .as_ref()
      .and_then(|configuration| configuration.name.as_deref())
  }

  #[must_use]
  pub fn is_success(&self) -> bool {
    self.status.as_ref().is_some_and(|status| {
      status
        .session
        .as_ref()
        .is_some_and(|session| session.succeeded == Some("true".to_string()))
    })
  }
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
  pub verbosity: Option<u8>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub passthrough: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Status {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub artifacts: Option<Vec<Artifact>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub session: Option<Session>, // FIXME: make it none Optional
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
pub struct Session {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub job: Option<String>,
  #[serde(rename = "completionTime")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub completion_time: Option<String>,
  #[serde(rename = "startTime")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub start_time: Option<String>, // FIXME: make it not Optional
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub succeeded: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Target {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub definition: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub groups: Option<Vec<Group>>,
}

impl CfsSessionPostRequest {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    name: String,
    configuration_name: &str,
    ansible_limit: Option<&str>,
    ansible_verbosity: Option<u8>,
    ansible_passthrough: Option<&str>,
    is_target_definition_image: bool,
    groups_name: Option<&[&str]>,
    base_image_id: Option<&str>,
  ) -> Self {
    // This code is fine... the fact that I put Self behind a variable is ok, since image param
    // is not a default param, then doing things differently is not an issue. I checked with
    // other Rust developers in their discord https://discord.com/channels/442252698964721669/448238009733742612/1081686300182188207
    let mut cfs_session = Self {
      name,
      configuration_name: configuration_name.to_string(),
      ansible_limit: ansible_limit.map(str::to_string),
      ansible_verbosity,
      ansible_passthrough: ansible_passthrough.map(str::to_string),
      ..Default::default()
    };

    if is_target_definition_image {
      // CFS session to build an image
      let target_groups: Vec<Group> = groups_name
        .unwrap_or_default()
        .iter()
        .map(|group_name| Group {
          name: group_name.to_string(),
          members: vec![base_image_id.unwrap_or_default().to_string()],
        })
        .collect();

      cfs_session.target.definition = Some("image".to_string());
      cfs_session.target.groups = Some(target_groups);
    }

    cfs_session
  }

  /// Returns all target.groups[].members[]. Most probably will return duplicated values.
  ///
  /// Example wire shape:
  /// ```json
  /// {
  ///  "name": "gallina-mc-compute-test-manuel-0.0",
  ///  "configurationName": "tmp-gallina-mc-compute-cfg-test-manuel-0.0",
  ///  "ansibleVerbosity": 2,
  ///  "target": {
  ///    "definition": "image",
  ///    "groups": [
  ///      {
  ///        "name": "Compute",
  ///        "members": [
  ///          "fd73dd9f-21c3-4328-bdc3-5fac65275d49"
  ///        ]
  ///      },
  ///      {
  ///        "name": "prealps",
  ///        "members": [
  ///          "fd73dd9f-21c3-4328-bdc3-5fac65275d49"
  ///        ]
  ///      },
  ///      {
  ///        "name": "gallina",
  ///        "members": [
  ///          "fd73dd9f-21c3-4328-bdc3-5fac65275d49"
  ///        ]
  ///      }
  ///    ]
  ///  }
  /// }
  /// ```
  #[must_use]
  pub fn get_base_image_ids(&self) -> Vec<String> {
    self
      .target
      .groups
      .as_ref()
      .unwrap_or(&Vec::new())
      .iter()
      .flat_map(|group| group.members.clone())
      .collect()
  }
}
