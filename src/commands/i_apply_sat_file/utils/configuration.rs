use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum Product {
  ProductVersionBranch {
    name: String,
    version: Option<String>,
    branch: String,
  },
  ProductVersionCommit {
    name: String,
    version: Option<String>,
    commit: String,
  },
  ProductVersion {
    name: String,
    version: String,
  },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum Git {
  GitCommit { url: String, commit: String },
  GitBranch { url: String, branch: String },
  GitTag { url: String, tag: String },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum LayerType {
  Git { git: Git },
  Product { product: Product },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Layer {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(default = "default_playbook")]
  pub playbook: String, // This field is optional but with default value. Therefore we won't
  #[serde(flatten)]
  pub layer_type: LayerType,
}

fn default_playbook() -> String {
  "site.yml".to_string()
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum Inventory {
  InventoryCommit {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    url: String,
    commit: String,
  },
  InventoryBranch {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    url: String,
    branch: String,
  },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Configuration {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  pub layers: Vec<Layer>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub additional_inventory: Option<Inventory>,
}
