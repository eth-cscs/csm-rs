use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Layer {
  pub name: Option<String>,
  #[serde(rename = "cloneUrl")]
  pub clone_url: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  // Either commit or branch is passed
  pub commit: Option<String>,
  pub playbook: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  // Either commit or branch is passed
  pub branch: Option<String>,
  // pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AdditionalInventory {
  #[serde(rename = "cloneUrl")]
  pub clone_url: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  // Either commit or branch is passed
  pub commit: Option<String>,
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  // Either commit or branch is passed
  pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CfsConfigurationResponse {
  pub name: String,
  #[serde(rename = "lastUpdated")]
  pub last_updated: String,
  pub layers: Vec<Layer>,
  #[serde(skip_serializing_if = "Option::is_none")]
  // Either commit or branch is passed
  pub additional_inventory: Option<AdditionalInventory>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CfsConfigurationVecResponse {
  pub configurations: Vec<CfsConfigurationResponse>,
  pub next: Option<Next>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Next {
  pub(super) limit: Option<u8>,
  pub(super) after_id: Option<String>,
  pub(super) in_use: Option<bool>,
}

impl Layer {
  #[must_use]
  pub fn new(
    clone_url: String,
    commit: Option<String>,
    name: Option<String>,
    playbook: String,
    branch: Option<String>,
  ) -> Self {
    Self {
      name,
      clone_url,
      commit,
      playbook,
      branch,
    }
  }
}

impl AdditionalInventory {
  pub fn new(
    clone_url: String,
    commit: Option<String>,
    name: String,
    branch: Option<String>,
  ) -> Self {
    Self {
      clone_url,
      commit,
      name,
      branch,
    }
  }
}

impl Default for CfsConfigurationResponse {
  fn default() -> Self {
    Self::new()
  }
}

impl CfsConfigurationResponse {
  #[must_use]
  pub fn new() -> Self {
    Self {
      name: String::default(),
      last_updated: String::default(),
      layers: Vec::default(),
      additional_inventory: None,
    }
  }

  pub fn add_layer(&mut self, layer: Layer) {
    self.layers.push(layer);
  }
}
