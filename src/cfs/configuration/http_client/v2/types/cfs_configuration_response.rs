use manta_backend_dispatcher::types::cfs::cfs_configuration_response::{
  AdditionalInventory as FrontEndAdditionalInventory,
  CfsConfigurationResponse as FrontendCfsConfigurationResponse,
  CfsConfigurationVecResponse as FrontendCfsConfigurationVecResponse,
  Layer as FrontendLayer, Next as FrontendNext,
};

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

impl From<FrontendLayer> for Layer {
  fn from(frontend_layer: FrontendLayer) -> Self {
    Self {
      name: frontend_layer.name,
      clone_url: frontend_layer.clone_url,
      commit: frontend_layer.commit,
      playbook: frontend_layer.playbook,
      branch: frontend_layer.branch,
    }
  }
}

impl From<Layer> for FrontendLayer {
  fn from(val: Layer) -> Self {
    FrontendLayer {
      name: val.name,
      clone_url: val.clone_url,
      source: None,
      commit: val.commit,
      playbook: val.playbook,
      branch: val.branch,
    }
  }
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

impl From<FrontEndAdditionalInventory> for AdditionalInventory {
  fn from(value: FrontEndAdditionalInventory) -> Self {
    Self {
      clone_url: value.clone_url,
      commit: value.commit,
      name: value.name,
      branch: value.branch,
    }
  }
}

impl From<AdditionalInventory> for FrontEndAdditionalInventory {
  fn from(val: AdditionalInventory) -> Self {
    FrontEndAdditionalInventory {
      clone_url: val.clone_url,
      commit: val.commit,
      name: val.name,
      branch: val.branch,
    }
  }
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

impl From<FrontendCfsConfigurationResponse> for CfsConfigurationResponse {
  fn from(value: FrontendCfsConfigurationResponse) -> Self {
    CfsConfigurationResponse {
      name: value.name,
      last_updated: value.last_updated,
      layers: value.layers.into_iter().map(Layer::from).collect(),
      additional_inventory: value
        .additional_inventory
        .map(AdditionalInventory::from),
    }
  }
}

impl From<CfsConfigurationResponse> for FrontendCfsConfigurationResponse {
  fn from(val: CfsConfigurationResponse) -> Self {
    FrontendCfsConfigurationResponse {
      name: val.name,
      last_updated: val.last_updated,
      layers: val.layers.into_iter().map(Into::into).collect(),
      additional_inventory: val.additional_inventory.map(Into::into),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CfsConfigurationVecResponse {
  pub configurations: Vec<CfsConfigurationResponse>,
  pub next: Option<Next>,
}

impl From<FrontendCfsConfigurationVecResponse> for CfsConfigurationVecResponse {
  fn from(value: FrontendCfsConfigurationVecResponse) -> Self {
    CfsConfigurationVecResponse {
      configurations: value
        .configurations
        .into_iter()
        .map(CfsConfigurationResponse::from)
        .collect(),
      next: value.next.map(Next::from),
    }
  }
}

impl From<CfsConfigurationVecResponse> for FrontendCfsConfigurationVecResponse {
  fn from(val: CfsConfigurationVecResponse) -> Self {
    FrontendCfsConfigurationVecResponse {
      configurations: val.configurations.into_iter().map(Into::into).collect(),
      next: val.next.map(Into::into),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Next {
  limit: Option<u8>,
  after_id: Option<String>,
  in_use: Option<bool>,
}

impl From<FrontendNext> for Next {
  fn from(value: FrontendNext) -> Self {
    Next {
      limit: value.limit,
      after_id: value.after_id,
      in_use: value.in_use,
    }
  }
}

impl From<Next> for FrontendNext {
  fn from(val: Next) -> Self {
    FrontendNext {
      limit: val.limit,
      after_id: val.after_id,
      in_use: val.in_use,
    }
  }
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

  pub fn from_sat_file_serde_yaml(
    configuration_yaml: &serde_yaml::Value,
  ) -> Self {
    let mut cfs_configuration = Self::new();

    cfs_configuration.name = configuration_yaml
      .get("name")
      .and_then(serde_yaml::Value::as_str)
      .map(str::to_string)
      .unwrap();

    for layer_yaml in configuration_yaml
      .get("layers")
      .and_then(serde_yaml::Value::as_sequence)
      .unwrap()
    {
      // log::info!("\n\n### Layer:\n{:#?}\n", layer_json);

      if layer_yaml.get("git").is_some() {
        // Git layer
        let repo_name = layer_yaml
          .get("name")
          .and_then(serde_yaml::Value::as_str)
          .map(str::to_string)
          .unwrap();
        let repo_url = layer_yaml
          .get("git")
          .and_then(|git| git.get("url"))
          .and_then(serde_yaml::Value::as_str)
          .map(str::to_string)
          .unwrap();
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
        let repo_url = format!(
          "https://api-gw-service-nmn.local/vcs/cray/{}-config-management.git",
          layer_yaml
            .get("name")
            .and_then(serde_yaml::Value::as_str)
            .unwrap()
        );
        let layer = Layer::new(
          repo_url,
          None,
          Some(layer_yaml["product"]
            .get("name")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string()),
          layer_yaml
            .get("playbook")
            .and_then(serde_yaml::Value::as_str)
            .map(str::to_string)
            .unwrap(),
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
