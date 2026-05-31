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
  pub fn new(
    clone_url: String,
    commit: Option<String>,
    name: Option<String>,
    playbook: String,
    branch: Option<String>,
  ) -> Self {
    Self {
      clone_url,
      commit,
      name,
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

  /// Build a `CfsConfigurationResponse` from a SAT-file YAML node.
  ///
  /// # Panics
  ///
  /// Panics if the YAML does not have the expected SAT-file shape (top-level
  /// `name` / `layers`, each git layer with `name` + `git.url`, each product
  /// layer with `name` + `playbook`). Callers are expected to have run SAT
  /// file validation first.
  ///
  /// Prefer `CfsConfigurationRequest::from_sat_file_serde_yaml`, which
  /// returns `Result<_, Error>` and produces actionable error messages.
  pub fn from_sat_file_serde_yaml(
    configuration_yaml: &serde_yaml::Value,
  ) -> Self {
    let mut cfs_configuration = Self::new();

    cfs_configuration.name = configuration_yaml
      .get("name")
      .and_then(serde_yaml::Value::as_str)
      .map(str::to_string)
      .expect("SAT file: configuration is missing 'name'");

    let layers = configuration_yaml
      .get("layers")
      .and_then(serde_yaml::Value::as_sequence)
      .expect("SAT file: configuration is missing 'layers'");
    for layer_yaml in layers {
      // log::info!("\n\n### Layer:\n{:#?}\n", layer_json);

      if layer_yaml.get("git").is_some() {
        // Git layer
        let repo_name = layer_yaml
          .get("name")
          .and_then(serde_yaml::Value::as_str)
          .map(str::to_string)
          .expect("SAT file: git layer is missing 'name'");
        let repo_url = layer_yaml
          .get("git")
          .and_then(|git| git.get("url"))
          .and_then(serde_yaml::Value::as_str)
          .map(str::to_string)
          .expect("SAT file: git layer is missing 'git.url'");
        let layer = Layer::new(
          repo_url,
          None,
          Some(repo_name),
          layer_yaml
            .get("playbook")
            .and_then(serde_yaml::Value::as_str)
            .map(str::to_string)
            .unwrap_or_default(),
          layer_yaml
            .get("git")
            .and_then(|git| git.get("branch"))
            .and_then(serde_yaml::Value::as_str)
            .map(str::to_string),
        );
        cfs_configuration.add_layer(layer);
      } else {
        // Product layer
        let product_name = layer_yaml
          .get("name")
          .and_then(serde_yaml::Value::as_str)
          .expect("SAT file: product layer is missing 'name'");
        let repo_url = format!(
          "https://api-gw-service-nmn.local/vcs/cray/{}-config-management.git",
          product_name
        );
        let layer = Layer::new(
          repo_url,
          None,
          Some(
            layer_yaml["product"]
              .get("name")
              .and_then(serde_yaml::Value::as_str)
              .unwrap_or_default()
              .to_string(),
          ),
          layer_yaml
            .get("playbook")
            .and_then(serde_yaml::Value::as_str)
            .map(str::to_string)
            .expect("SAT file: product layer is missing 'playbook'"),
          Some(
            layer_yaml["product"]
              .get("branch")
              .and_then(serde_yaml::Value::as_str)
              .unwrap_or_default()
              .to_string(),
          ),
        );
        cfs_configuration.add_layer(layer);
      }
    }
    cfs_configuration
  }
}
