//! Projection types returned by csm-rs's `hsm_hw_inventory_get`. They
//! have no spec equivalent; the wrapper builds them from the raw
//! `serde_json::Value` payload (today) or from progenitor's generated
//! `HwInventory100HwInventoryByLocation` type (future migration).
//!
//! Lives in the wrapper layer per the design decision that `types.rs`
//! files mirror wire shapes only; hand-rolled helpers/projections
//! belong with the wrapper that creates them. The existing public path
//! `csm_rs::hsm::hw_inventory::hw_component::{NodeSummary,
//! ArtifactSummary, ArtifactType}` stays valid via re-exports from
//! `src/hsm/hw_inventory/hw_component/types.rs`.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use strum_macros::{
  AsRefStr, Display, EnumIter, EnumString, IntoStaticStr,
};

use crate::error::Error;

/// Extract a required string field from an HSM inventory JSON object.
/// Returns a descriptive [`Error::HsmInventoryShape`] if the key is
/// missing or the value isn't a string, instead of panicking like the
/// previous `.unwrap()`.
fn required_str(v: &Value, key: &str) -> Result<String, Error> {
  v.get(key)
    .and_then(Value::as_str)
    .map(str::to_string)
    .ok_or_else(|| {
      Error::HsmInventoryShape(format!(
        "required field '{key}' is missing or not a string"
      ))
    })
}

///////////////////////////////////////////////////////////////////////////////
// MESA - These are non-official structs created from 'curl' response payload
#[derive(
  Debug,
  EnumIter,
  EnumString,
  IntoStaticStr,
  AsRefStr,
  Display,
  Serialize,
  Deserialize,
  Clone,
)]
pub enum ArtifactType {
  Memory,
  Processor,
  NodeAccel,
  NodeHsnNic,
  Drive,
  CabinetPDU,
  CabinetPDUPowerConnector,
  CMMRectifier,
  NodeAccelRiser,
  NodeEnclosurePowerSupplie,
  NodeBMC,
  RouterBMC,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeSummary {
  pub xname: String,
  pub r#type: String,
  pub processors: Vec<ArtifactSummary>,
  pub memory: Vec<ArtifactSummary>,
  pub node_accels: Vec<ArtifactSummary>,
  pub node_hsn_nics: Vec<ArtifactSummary>,
}

impl NodeSummary {
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub fn try_from_csm_value(
    hw_artifact_value: &Value,
  ) -> Result<Self, Error> {
    let processors = parse_artifact_array(
      hw_artifact_value,
      "Processors",
      ArtifactSummary::try_from_processor_value,
    )?;
    let memory = parse_artifact_array(
      hw_artifact_value,
      "Memory",
      ArtifactSummary::try_from_memory_value,
    )?;
    let node_accels = parse_artifact_array(
      hw_artifact_value,
      "NodeAccels",
      ArtifactSummary::try_from_nodeaccel_value,
    )?;
    let node_hsn_nics = parse_artifact_array(
      hw_artifact_value,
      "NodeHsnNics",
      ArtifactSummary::try_from_nodehsnnics_value,
    )?;

    Ok(Self {
      xname: required_str(hw_artifact_value, "ID")?,
      r#type: required_str(hw_artifact_value, "Type")?,
      processors,
      memory,
      node_accels,
      node_hsn_nics,
    })
  }
}

fn parse_artifact_array(
  parent: &Value,
  key: &str,
  parse_one: fn(&Value) -> Result<ArtifactSummary, Error>,
) -> Result<Vec<ArtifactSummary>, Error> {
  match parent.get(key).and_then(Value::as_array) {
    Some(arr) => arr.iter().map(parse_one).collect(),
    None => Ok(Vec::new()),
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtifactSummary {
  pub xname: String,
  pub r#type: ArtifactType,
  pub info: Option<String>,
}

impl ArtifactSummary {
  fn try_from_processor_value(
    processor_value: &Value,
  ) -> Result<Self, Error> {
    Ok(Self {
      xname: required_str(processor_value, "ID")?,
      r#type: parse_artifact_type(processor_value)?,
      info: processor_value
        .pointer("/PopulatedFRU/ProcessorFRUInfo/Model")
        .and_then(Value::as_str)
        .map(str::to_string),
    })
  }

  fn try_from_memory_value(memory_value: &Value) -> Result<Self, Error> {
    Ok(Self {
      xname: required_str(memory_value, "ID")?,
      r#type: parse_artifact_type(memory_value)?,
      info: memory_value
        .pointer("/PopulatedFRU/MemoryFRUInfo/CapacityMiB")
        .and_then(Value::as_number)
        .map(|v| v.to_string() + " MiB"),
    })
  }

  fn try_from_nodehsnnics_value(
    nodehsnnic_value: &Value,
  ) -> Result<Self, Error> {
    Ok(Self {
      xname: required_str(nodehsnnic_value, "ID")?,
      r#type: parse_artifact_type(nodehsnnic_value)?,
      info: nodehsnnic_value
        .pointer("/NodeHsnNicLocationInfo/Description")
        .and_then(Value::as_str)
        .map(str::to_string),
    })
  }

  fn try_from_nodeaccel_value(nodeaccel_value: &Value) -> Result<Self, Error> {
    Ok(Self {
      xname: required_str(nodeaccel_value, "ID")?,
      r#type: parse_artifact_type(nodeaccel_value)?,
      info: nodeaccel_value
        .pointer("/PopulatedFRU/NodeAccelFRUInfo/Model")
        .and_then(Value::as_str)
        .map(str::to_string),
    })
  }
}

fn parse_artifact_type(v: &Value) -> Result<ArtifactType, Error> {
  let type_str = required_str(v, "Type")?;
  ArtifactType::from_str(&type_str).map_err(|e| {
    Error::HsmInventoryShape(format!(
      "unknown ArtifactType '{type_str}': {e:?}"
    ))
  })
}
