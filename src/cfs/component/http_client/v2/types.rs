//! Wire-format types — mirror the upstream CSM OpenAPI schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct State {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "cloneUrl")]
  pub clone_url: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub playbook: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub commit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "sesisonName")]
  pub session_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Component {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub state: Option<Vec<State>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "stateAppend")]
  pub state_append: Option<State>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "desiredConfig")]
  pub desired_config: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "errorCount")]
  pub error_count: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "retryPolicy")]
  pub retry_policy: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub enabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "configurationStatus")]
  pub configuration_status: Option<String>, //values unconfigured, pending, failed, configured
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tags: Option<HashMap<String, String>>,
}

