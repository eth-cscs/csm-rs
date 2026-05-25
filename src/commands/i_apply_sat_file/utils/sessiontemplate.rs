//! Serde shapes for one section of a SAT (System Admin Toolkit) YAML
//! file; field names and shapes are dictated by the SAT format.
#![allow(missing_docs)]

use std::collections::HashMap;
use strum_macros::Display;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionTemplate {
  pub name: String,
  pub image: Image,
  pub configuration: String,
  pub bos_parameters: BosParamters,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum ImsDetails {
  Name { name: String },
  Id { id: String },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum Image {
  Ims { ims: ImsDetails },
  ImageRef(String),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BosParamters {
  pub boot_sets: HashMap<String, BootSet>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BootSet {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arch: Option<Arch>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub kernel_parameters: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub network: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_list: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_roles_group: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_groups: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rootfs_provider: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rootfs_provider_passthrough: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Display)]
pub enum Arch {
  X86,
  ARM,
  Other,
  Unknown,
}
