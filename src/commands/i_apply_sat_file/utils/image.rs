//! Serde shapes for one section of a SAT (System Admin Toolkit) YAML
//! file; field names and shapes are dictated by the SAT format.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;

#[derive(Deserialize, Serialize, Debug, Clone, AsRefStr)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum Arch {
  #[serde(rename(serialize = "aarch64", deserialize = "aarch64"))]
  Aarch64,
  #[serde(rename(serialize = "x86_64", deserialize = "x86_64"))]
  X86_64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum ImageIms {
  NameIsRecipe { name: String, is_recipe: bool },
  IdIsRecipe { id: String, is_recipe: bool },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum ImageBaseIms {
  NameType { name: String, r#type: String },
  IdType { id: String, r#type: String },
  BackwardCompatible { is_recipe: Option<bool>, id: String },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum Filter {
  Prefix { prefix: String },
  Wildcard { wildcard: String },
  Arch { arch: Arch },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Product {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,
  pub r#type: String,
  pub filter: Option<Filter>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum Base {
  Ims { ims: ImageBaseIms },
  Product { product: Product },
  ImageRef { image_ref: String },
}

// Used for backguard compatibility
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum BaseOrIms {
  Base { base: Base },
  Ims { ims: ImageIms },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Image {
  pub name: String,
  #[serde(flatten)]
  pub base_or_ims: BaseOrIms,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub configuration: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub configuration_group_names: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ref_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}
