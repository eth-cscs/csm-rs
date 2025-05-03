use manta_backend_dispatcher::types::cfs::cfs_configuration_response::{
    AdditionalInventory as FrontEndAdditionalInventory,
    CfsConfigurationResponse as FrontendCfsConfigurationResponse,
    CfsConfigurationVecResponse as FrontendCfsConfigurationVecResponse, Layer as FrontendLayer,
    Next as FrontendNext,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Layer {
    pub name: String,
    #[serde(rename = "cloneUrl")]
    pub clone_url: String,
    #[serde(skip_serializing_if = "Option::is_none")] // Either commit or branch is passed
    pub commit: Option<String>,
    pub playbook: String,
    #[serde(skip_serializing_if = "Option::is_none")] // Either commit or branch is passed
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

impl Into<FrontendLayer> for Layer {
    fn into(self) -> FrontendLayer {
        FrontendLayer {
            name: self.name,
            clone_url: self.clone_url,
            source: None,
            commit: self.commit,
            playbook: self.playbook,
            branch: self.branch,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AdditionalInventory {
    #[serde(rename = "cloneUrl")]
    pub clone_url: String,
    #[serde(skip_serializing_if = "Option::is_none")] // Either commit or branch is passed
    pub commit: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")] // Either commit or branch is passed
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

impl Into<FrontEndAdditionalInventory> for AdditionalInventory {
    fn into(self) -> FrontEndAdditionalInventory {
        FrontEndAdditionalInventory {
            clone_url: self.clone_url,
            commit: self.commit,
            name: self.name,
            branch: self.branch,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CfsConfigurationResponse {
    pub name: String,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
    pub layers: Vec<Layer>,
    #[serde(skip_serializing_if = "Option::is_none")] // Either commit or branch is passed
    pub additional_inventory: Option<AdditionalInventory>,
}

impl From<FrontendCfsConfigurationResponse> for CfsConfigurationResponse {
    fn from(value: FrontendCfsConfigurationResponse) -> Self {
        CfsConfigurationResponse {
            name: value.name,
            last_updated: value.last_updated,
            layers: value.layers.into_iter().map(Layer::from).collect(),
            additional_inventory: value.additional_inventory.map(AdditionalInventory::from),
        }
    }
}

impl Into<FrontendCfsConfigurationResponse> for CfsConfigurationResponse {
    fn into(self) -> FrontendCfsConfigurationResponse {
        FrontendCfsConfigurationResponse {
            name: self.name,
            last_updated: self.last_updated,
            layers: self.layers.into_iter().map(Into::into).collect(),
            additional_inventory: self.additional_inventory.map(Into::into),
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

impl Into<FrontendCfsConfigurationVecResponse> for CfsConfigurationVecResponse {
    fn into(self) -> FrontendCfsConfigurationVecResponse {
        FrontendCfsConfigurationVecResponse {
            configurations: self.configurations.into_iter().map(Into::into).collect(),
            next: self.next.map(Into::into),
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

impl Into<FrontendNext> for Next {
    fn into(self) -> FrontendNext {
        FrontendNext {
            limit: self.limit,
            after_id: self.after_id,
            in_use: self.in_use,
        }
    }
}

impl Layer {
    pub fn new(
        clone_url: String,
        // source: Option<String>,
        commit: Option<String>,
        name: String,
        playbook: String,
        branch: Option<String>,
    ) -> Self {
        Self {
            clone_url,
            // source,
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

    pub fn from_sat_file_serde_yaml(configuration_yaml: &serde_yaml::Value) -> Self {
        let mut cfs_configuration = Self::new();

        cfs_configuration.name = configuration_yaml["name"].as_str().unwrap().to_string();

        for layer_yaml in configuration_yaml["layers"].as_sequence().unwrap() {
            // println!("\n\n### Layer:\n{:#?}\n", layer_json);

            if layer_yaml.get("git").is_some() {
                // Git layer
                let repo_name = layer_yaml["name"].as_str().unwrap().to_string();
                let repo_url = layer_yaml["git"]["url"].as_str().unwrap().to_string();
                let layer = Layer::new(
                    repo_url,
                    // None, // TODO: replace with real source value
                    // Some(layer_json["git"]["commit"].as_str().unwrap_or_default().to_string()),
                    None,
                    repo_name,
                    layer_yaml["playbook"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    Some(
                        layer_yaml["git"]["branch"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                    ),
                );
                cfs_configuration.add_layer(layer);
            } else {
                // Product layer
                let repo_url = format!(
                    "https://api-gw-service-nmn.local/vcs/cray/{}-config-management.git",
                    layer_yaml["name"].as_str().unwrap()
                );
                let layer = Layer::new(
                    repo_url,
                    // None, // TODO: replace with real source value
                    // Some(layer_json["product"]["commit"].as_str().unwrap_or_default().to_string()),
                    None,
                    layer_yaml["product"]["name"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    layer_yaml["playbook"].as_str().unwrap().to_string(),
                    Some(
                        layer_yaml["product"]["branch"]
                            .as_str()
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
