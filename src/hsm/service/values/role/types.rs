use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Role {
  #[serde(rename = "Role")]
  pub role: Vec<String>,
}
