//! Wire-format types — mirror the upstream CSM `OpenAPI` schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

/// Projection types ([`NodeSummary`], [`ArtifactSummary`],
/// [`ArtifactType`]) live in the wrapper layer per the design
/// decision recorded in
/// `docs/superpowers/specs/2026-06-13-progenitor-hsm-codegen-design.md`
/// (§ Type strategy). They are re-exported here so the historical
/// public path
/// `csm_rs::hsm::hw_inventory::hw_component::{NodeSummary,
/// ArtifactSummary, ArtifactType}` remains valid.
pub use crate::hsm::wrapper::hw_component_types::{
  ArtifactSummary, ArtifactType, NodeSummary,
};

/// Generate bidirectional `From` impls for two structs with identical field
/// names where each field has the same type on both sides (primitives,
/// `Option<String>`, etc). Fields are moved unchanged.
///
/// Used by [`super::dispatcher_conv`] when the `manta-dispatcher` Cargo
/// feature is enabled.
#[allow(unused_macros)]
macro_rules! bidirectional_from {
  ($our:ty, $fe:ty, [ $($field:ident),* $(,)? ]) => {
    impl From<$fe> for $our {
      fn from(v: $fe) -> Self {
        Self { $($field: v.$field,)* }
      }
    }
    impl From<$our> for $fe {
      fn from(v: $our) -> Self {
        Self { $($field: v.$field,)* }
      }
    }
  };
}

/// Like `bidirectional_from!` but each field is converted via `Into`.
/// Appropriate when one or more field types differ between sides and each
/// has its own paired `From` impls (so `.into()` recurses).
///
/// Used by [`super::dispatcher_conv`] when the `manta-dispatcher` Cargo
/// feature is enabled.
#[allow(unused_macros)]
macro_rules! bidirectional_from_into {
  ($our:ty, $fe:ty, [ $($field:ident),* $(,)? ]) => {
    impl From<$fe> for $our {
      fn from(v: $fe) -> Self {
        Self { $($field: v.$field.into(),)* }
      }
    }
    impl From<$our> for $fe {
      fn from(v: $our) -> Self {
        Self { $($field: v.$field.into(),)* }
      }
    }
  };
}

/// Bidirectional `From` impls with per-field conversion strategies.
///
/// Fields are partitioned into categories:
///   - `direct`: copied as-is (same type on both sides)
///   - `into`: converted via `.into()` (paired nested types)
///   - `opt_into`: `Option<T>` → `.map(Into::into)`
///   - `vec_into`: `Vec<T>` → `.into_iter().map(Into::into).collect()`
///   - `opt_vec_into`: `Option<Vec<T>>` → `.map(|v| v.into_iter().map(Into::into).collect())`
///
/// Any category may be omitted.
///
/// Used by [`super::dispatcher_conv`] when the `manta-dispatcher` Cargo
/// feature is enabled.
#[allow(unused_macros)]
macro_rules! bidirectional_from_mixed {
  (
    $our:ty, $fe:ty,
    $(direct: [ $($df:ident),* $(,)? ],)?
    $(into: [ $($if:ident),* $(,)? ],)?
    $(opt_into: [ $($of:ident),* $(,)? ],)?
    $(vec_into: [ $($vf:ident),* $(,)? ],)?
    $(opt_vec_into: [ $($ovf:ident),* $(,)? ] $(,)?)?
  ) => {
    impl From<$fe> for $our {
      fn from(v: $fe) -> Self {
        Self {
          $($($df: v.$df,)*)?
          $($($if: v.$if.into(),)*)?
          $($($of: v.$of.map(Into::into),)*)?
          $($($vf: v.$vf.into_iter().map(Into::into).collect(),)*)?
          $($($ovf: v.$ovf.map(|vec| vec.into_iter().map(Into::into).collect()),)*)?
        }
      }
    }
    impl From<$our> for $fe {
      fn from(v: $our) -> Self {
        Self {
          $($($df: v.$df,)*)?
          $($($if: v.$if.into(),)*)?
          $($($of: v.$of.map(Into::into),)*)?
          $($($vf: v.$vf.into_iter().map(Into::into).collect(),)*)?
          $($($ovf: v.$ovf.map(|vec| vec.into_iter().map(Into::into).collect()),)*)?
        }
      }
    }
  };
}

///////////////////////////////////////////////////////////////////////////////
// CSM - structs from CSM API documentation. FIXME: need to address FRU structs properly with enums
#[derive(Debug, Serialize, Deserialize)]
pub enum ComponentType {
  CDU,
  CabinetCDU,
  CabinetPDU,
  CabinetPDUOutlet,
  CabinetPDUPowerConnector,
  CabinetPDUController,
  r#Cabinet,
  Chassis,
  ChassisBMC,
  CMMRectifier,
  CMMFpga,
  CEC,
  ComputeModule,
  RouterModule,
  NodeBMC,
  NodeEnclosure,
  NodeEnclosurePowerSupply,
  HSNBoard,
  Node,
  Processor,
  Drive,
  StorageGroup,
  NodeNIC,
  Memory,
  NodeAccel,
  NodeAccelRiser,
  NodeFpga,
  HSNAsic,
  RouterFpga,
  RouterBMC,
  HSNLink,
  HSNConnector,
  INVALID,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessorId {
  #[serde(rename = "EffectiveFamily")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub effective_family: Option<String>,
  #[serde(rename = "EffectiveModel")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub efffective_model: Option<String>,
  #[serde(rename = "IdentificationRegisters")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub identification_registers: Option<String>,
  #[serde(rename = "MicrocodeInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub microcode_info: Option<String>,
  #[serde(rename = "Step")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub step: Option<String>,
  #[serde(rename = "VendorId")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub vendor_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishProcessorFRUInfo {
  #[serde(rename = "InstructionSet")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub instruction_set: Option<String>,
  #[serde(rename = "Manufacturer")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub manufacturer: Option<String>,
  #[serde(rename = "MaxSpeedMHz")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_speed_mhz: Option<usize>,
  #[serde(rename = "Model")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub model: Option<String>,
  #[serde(rename = "ProcessorArchitecture")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processor_architecture: Option<String>,
  #[serde(rename = "ProcessorId")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processor_id: Option<ProcessorId>,
  #[serde(rename = "ProcessorType")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processor_type: Option<String>,
  #[serde(rename = "TotalCores")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total_cores: Option<usize>,
  #[serde(rename = "TotalThreads")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total_threads: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByFRUProcessor {
  #[serde(rename = "FRUID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_id: Option<String>,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "FRUSubType")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_sub_type: Option<String>,
  #[serde(rename = "HWInventoryByFRUType")]
  pub hw_inventory_by_fru_type: String,
  #[serde(rename = "ProcessorFRUInfo")]
  pub processor_fru_info: RedfishProcessorFRUInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishMemoryFRUInfo {
  #[serde(rename = "BaseModuleType")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub base_module_type: Option<String>,
  #[serde(rename = "BusWidthBits")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub bus_width_bits: Option<usize>,
  #[serde(rename = "CapacityMiB")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub capacity_mib: Option<usize>,
  #[serde(rename = "DataWidthBits")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data_width_bits: Option<usize>,
  #[serde(rename = "ErrorCorrection")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error_correction: Option<String>,
  #[serde(rename = "Manufacturer")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub manufacturer: Option<String>,
  #[serde(rename = "MemoryType")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_type: Option<String>,
  #[serde(rename = "MemoryDeviceType")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_device_type: Option<String>,
  #[serde(rename = "OperatingSpeedMhz")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub operating_speed_mhz: Option<usize>,
  #[serde(rename = "PartNumber")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub part_number: Option<String>,
  #[serde(rename = "RankCount")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rank_count: Option<usize>,
  #[serde(rename = "SerialNumber")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub serial_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByFRUMemory {
  #[serde(rename = "FRUID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_id: Option<String>,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "FRUSubType")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_sub_type: Option<String>,
  #[serde(rename = "HWInventoryByFRUType")]
  pub hw_inventory_by_fru_type: String,
  #[serde(rename = "MemoryFRUInfo")]
  pub memory_fru_info: RedfishMemoryFRUInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByFRUNodeAccel {
  #[serde(rename = "FRUID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_id: Option<String>,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "FRUSubType")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_sub_type: Option<String>,
  #[serde(rename = "HWInventoryByFRUType")]
  pub hw_inventory_by_fru_type: String,
  #[serde(rename = "NodeAccelFRUInfo")]
  pub node_accel_fru_info: RedfishProcessorFRUInfo, // NOTE: according to API
                                                    // docs, yes this is using the redfish for "processor"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HSNNICFRUInfo {
  #[serde(rename = "Manufacturer")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub manufacturer: Option<String>,
  #[serde(rename = "Model")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub model: Option<String>,
  #[serde(rename = "PartNumber")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub part_number: Option<String>,
  #[serde(rename = "SKU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sku: Option<String>,
  #[serde(rename = "SerialNumber")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub serial_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByFRUHSNNIC {
  #[serde(rename = "FRUID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_id: Option<String>,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "FRUSubType")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_sub_type: Option<String>,
  #[serde(rename = "HWInventoryByFRUType")]
  pub hw_inventory_by_fru_type: String,
  #[serde(rename = "HSNNICFRUInfo")]
  pub hsn_nic_fru_info: HSNNICFRUInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInventoryByFRU {
  #[serde(rename = "FRUID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_id: Option<String>,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "FRUSubType")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_sub_type: Option<String>,
  #[serde(rename = "HWInventoryByFRUType")]
  pub hw_inventory_by_fru_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishChassisLocationInfo {
  #[serde(rename = "Id")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename = "Hostname")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hostname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocChassis {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "ChassisLocatinInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub chassis_location_info: Option<RedfishChassisLocationInfo>,
  #[serde(rename = "ComputeModules")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub compute_modules: Option<HWInvByLocComputeModule>,
  #[serde(rename = "RouterModules")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub router_modules: Option<HWInvByLocRouterModule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocNodeEnclosure {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "NodeEnclosureLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_enclosure_location_info: Option<RedfishChassisLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocComputeModule {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "ComputeModuleLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub compute_module_location_info: Option<RedfishChassisLocationInfo>,
  #[serde(rename = "NodeEnclosures")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_enclosures: Option<HWInvByLocNodeEnclosure>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocHSNBoard {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "HSNBoardLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hsn_board_location_info: Option<RedfishChassisLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocRouterModule {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "RouterModuleLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub router_module_location_info: Option<RedfishChassisLocationInfo>,
  pub hsn_boards: Option<HWInvByLocHSNBoard>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocCabinet {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "CabinetLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cabinet_location_info: Option<RedfishChassisLocationInfo>,
  #[serde(rename = "Chassis")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub chassis: Option<HWInvByLocChassis>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocMgmtSwitch {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "MgmtSwitchLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mgmt_switch_location_info: Option<RedfishChassisLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocMgmtHLSwitch {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "MgmtHLSwitchLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mgmt_hl_switch_location_info: Option<RedfishChassisLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocCDUMgmtSwitch {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "CDUMgmtSwitchLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cdu_mgmt_switch_location_info: Option<RedfishChassisLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessorSummary {
  #[serde(rename = "Count")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(super) count: Option<u32>,
  #[serde(rename = "Model")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(super) model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemorySummary {
  #[serde(rename = "TotalSystemMemoryGiB")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total_system_memory_gib: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishSystemLocationInfo {
  #[serde(rename = "Id")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename = "Hostname")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hostname: Option<String>,
  #[serde(rename = "ProcessorSummary")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processor_summary: Option<ProcessorSummary>,
  #[serde(rename = "MemorySummary")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_summary: Option<MemorySummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishProcessorLocationInfo {
  #[serde(rename = "Id")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename = "Socket")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub socket: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocProcessor {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInvByFRUProcessor>,
  #[serde(rename = "ProcessorLocationInfo")]
  pub processor_location_info: RedfishProcessorLocationInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocNodeAccel {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInvByFRUNodeAccel>,
  #[serde(rename = "NodeAccelLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_accel_location_info: Option<RedfishProcessorLocationInfo>, // NOTE: according to API
                                                                      // docs, yes this is using the redfish for "processor""
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishDriveLocationInfo {
  #[serde(rename = "Id")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocDrive {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "DriveLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub drive_location_info: Option<RedfishDriveLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryLocation {
  #[serde(rename = "Socket")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub socket: Option<u32>,
  #[serde(rename = "MemoryController")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_controller: Option<u32>,
  #[serde(rename = "Channel")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub channel: Option<u32>,
  #[serde(rename = "Slot")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub slot: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishMemoryLocationInfo {
  #[serde(rename = "Id")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename = "MemoryLocation")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_location: Option<MemoryLocation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocMemory {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInvByFRUMemory>,
  #[serde(rename = "MemoryLocationInfo")]
  pub memory_location_info: RedfishMemoryLocationInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishNodeAccelRiserLocationInfo {
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocNodeAccelRiser {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "NodeAccelRiserLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_accel_riser_location_info: Option<RedfishNodeAccelRiserLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HSNNICLocationInfo {
  #[serde(rename = "Id")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocHSNNIC {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInvByFRUHSNNIC>,
  /* #[serde(rename = "NodeHsnNicLocationInfo")]
  pub node_hsn_nic_location_info: HSNNICLocationInfo, */
  #[serde(rename = "HSNNICLocationInfo")]
  pub hsn_nic_location_info: HSNNICLocationInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hardware {
  #[serde(rename = "Hardware")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hardware: Option<Vec<HWInvByLocNode>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocNode {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInvByFRUNode>,
  #[serde(rename = "NodeLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_location_info: Option<RedfishSystemLocationInfo>,
  #[serde(rename = "Processors")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processors: Option<Vec<HWInvByLocProcessor>>,
  #[serde(rename = "NodeAccels")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_accels: Option<Vec<HWInvByLocNodeAccel>>,
  #[serde(rename = "Dives")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub drives: Option<Vec<HWInvByLocDrive>>,
  #[serde(rename = "Memory")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory: Option<Vec<HWInvByLocMemory>>,
  #[serde(rename = "NodeAccelRisers")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_accel_risers: Option<Vec<HWInvByLocNodeAccelRiser>>,
  #[serde(rename = "NodeHsnNICs")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_hsn_nics: Option<Vec<HWInvByLocHSNNIC>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishPDULocationInfo {
  #[serde(rename = "Id")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename = "UUID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishOutletLocationInfo {
  #[serde(rename = "Id")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocOutlet {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "OutletLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub outlet_location_info: Option<RedfishOutletLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocPDU {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "PDULocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pdu_location_info: Option<RedfishPDULocationInfo>,
  #[serde(rename = "CabinetPDUPowerConnectors")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cabinet_pdu_power_connectors: Option<Vec<HWInvByLocOutlet>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishCMMRectifierLocationInfo {
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "FirmwareVersion")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub firmware_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocCMMRectifier {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "CMMRectifierLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cmm_rectifier_location_info: Option<RedfishCMMRectifierLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishNodeEnclosurePowerSupplyLocationInfo {
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "FirmwareVersion")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub firmware_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocNodePowerSupply {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "NodeEnclosurePowerSupplyLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_enclosure_power_supply_location_info:
    Option<RedfishNodeEnclosurePowerSupplyLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishManagerLocationInfo {
  #[serde(rename = "Id")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename = "DateTime")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub date_time: Option<String>,
  #[serde(rename = "DateTimeLocalOffset")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub date_time_local_offset: Option<String>,
  #[serde(rename = "FirmwareVersion")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub firmware_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocNodeBMC {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "NodeBMCLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_bmc_location_info: Option<RedfishManagerLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocRouterBMC {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "Ordinal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename = "PopulatedFRU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename = "RouterBMCLocationInfo")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub router_bmc_location_info: Option<RedfishManagerLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInventory {
  #[serde(rename = "XName")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub xname: Option<String>,
  #[serde(rename = "Format")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub format: Option<String>,
  #[serde(rename = "Cabinets")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cabinets: Option<Vec<HWInvByLocCabinet>>,
  #[serde(rename = "Chassis")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub chassis: Option<Vec<HWInvByLocChassis>>,
  #[serde(rename = "ComputeModules")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub compute_modules: Option<Vec<HWInvByLocComputeModule>>,
  #[serde(rename = "RouterModules")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub router_modules: Option<Vec<HWInvByLocRouterModule>>,
  #[serde(rename = "NodeEnclosures")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_enclosures: Option<Vec<HWInvByLocNodeEnclosure>>,
  #[serde(rename = "HSNBoards")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hsn_boards: Option<Vec<HWInvByLocHSNBoard>>,
  #[serde(rename = "MgmtSwitches")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mgmt_switches: Option<Vec<HWInvByLocMgmtSwitch>>,
  #[serde(rename = "MgmtHLSwitches")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mgmt_hl_switches: Option<Vec<HWInvByLocMgmtHLSwitch>>,
  #[serde(rename = "CDUMgmtSwitches")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cdu_mgmt_switches: Option<Vec<HWInvByLocCDUMgmtSwitch>>,
  #[serde(rename = "Nodes")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub nodes: Option<Vec<HWInvByLocNode>>,
  #[serde(rename = "Processors")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processors: Option<Vec<HWInvByLocProcessor>>,
  #[serde(rename = "NodeAccels")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_accels: Option<Vec<HWInvByLocNodeAccel>>,
  #[serde(rename = "Drives")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub drives: Option<Vec<HWInvByLocDrive>>,
  #[serde(rename = "Memory")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory: Option<Vec<HWInvByLocMemory>>,
  #[serde(rename = "CabinetPDUs")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cabinet_pdus: Option<Vec<HWInvByLocPDU>>,
  #[serde(rename = "CabinetPDUPowerConnectors")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cabinet_pdu_power_connectors: Option<Vec<HWInvByLocOutlet>>,
  #[serde(rename = "CMMRectifiers")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cmm_rectifiers: Option<Vec<HWInvByLocCMMRectifier>>,
  #[serde(rename = "NodeAccelRisers")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_accel_risers: Option<Vec<HWInvByLocNodeAccelRiser>>,
  #[serde(rename = "NodeHsnNICs")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_hsn_nics: Option<Vec<HWInvByLocHSNNIC>>,
  #[serde(rename = "NodeEnclosurePowerSupplies")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_enclosure_power_supplies: Option<Vec<HWInvByLocNodePowerSupply>>,
  #[serde(rename = "NodeBMC")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_bmc: Option<Vec<HWInvByLocNodeBMC>>,
  #[serde(rename = "RouterBMC")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub router_bmc: Option<Vec<HWInvByLocRouterBMC>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByFRUNode {
  #[serde(rename = "FRUID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_id: Option<String>,
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename = "FRUSubType")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_sub_type: Option<String>,
  #[serde(rename = "HWInventoryByFRUType")]
  pub hw_inventory_by_fru_type: String,
  #[serde(rename = "NodeFRUInfo")]
  pub node_fru_info: RedfishSystemFRUInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishSystemFRUInfo {
  #[serde(rename = "AssetTag")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub asset_tag: Option<String>,
  #[serde(rename = "BiosVersion")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub bios_version: Option<String>,
  #[serde(rename = "Model")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub model: Option<String>,
  #[serde(rename = "Manufacturer")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub manufacturer: Option<String>,
  #[serde(rename = "PartNumber")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub part_number: Option<String>,
  #[serde(rename = "SerialNumber")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub serial_number: Option<String>,
  #[serde(rename = "SKU")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sku: Option<String>,
  #[serde(rename = "SystemType")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub system_type: Option<String>,
  #[serde(rename = "UUID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeLocationInfo {
  #[serde(rename = "Id")]
  pub id: String,
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "Description")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename = "Hostname")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hostname: Option<String>,
  #[serde(rename = "ProcessorSummary")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processor_summary: Option<ProcessorSummary>,
  #[serde(rename = "MemorySummary")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_summary: Option<MemorySummary>,
}

// Internally tagged: the CSM/HSM wire format puts the discriminator in
// the `HWInventoryByLocationType` field alongside the variant's other
// fields. Serde reads it to choose the variant, then deserializes the
// remaining fields into the inner struct — which is why each leaf
// struct no longer carries a `hw_inventory_by_location_type` field of
// its own. The tag value must match the Rust variant name (e.g.
// `HWInvByLocCabinet`).
// The two largest variants (`HWInvByLocCabinet`, `HWInvByLocChassis`)
// are boxed so the enum doesn't pay the size cost of the rarely-used
// big variants on every value. Serde `Box<T>` deserializes the same as
// `T`, so this is wire-compatible. See `clippy::large_enum_variant`.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "HWInventoryByLocationType")]
pub enum HWInventoryByLocation {
  HWInvByLocCDUMgmtSwitch(HWInvByLocCDUMgmtSwitch),
  HWInvByLocCMMRectifier(HWInvByLocCMMRectifier),
  HWInvByLocCabinet(Box<HWInvByLocCabinet>),
  HWInvByLocChassis(Box<HWInvByLocChassis>),
  HWInvByLocComputeModule(HWInvByLocComputeModule),
  HWInvByLocDrive(HWInvByLocDrive),
  HWInvByLocHSNBoard(HWInvByLocHSNBoard),
  HWInvByLocHSNNIC(HWInvByLocHSNNIC),
  HWInvByLocMemory(HWInvByLocMemory),
  HWInvByLocMgmtHLSwitch(HWInvByLocMgmtHLSwitch),
  HWInvByLocMgmtSwitch(HWInvByLocMgmtSwitch),
  HWInvByLocNode(HWInvByLocNode),
  HWInvByLocNodeAccel(HWInvByLocNodeAccel),
  HWInvByLocNodeAccelRiser(HWInvByLocNodeAccelRiser),
  HWInvByLocNodeBMC(HWInvByLocNodeBMC),
  HWInvByLocNodeEnclosure(HWInvByLocNodeEnclosure),
  HWInvByLocNodePowerSupply(HWInvByLocNodePowerSupply),
  HWInvByLocOutlet(HWInvByLocOutlet),
  HWInvByLocPDU(HWInvByLocPDU),
  HWInvByLocProcessor(HWInvByLocProcessor),
  HWInvByLocRouterBMC(HWInvByLocRouterBMC),
  HWInvByLocRouterModule(HWInvByLocRouterModule),
}

/// struct used in POST and GET endpoints that manage multiple instances of '`HWInventoryByLocation`'
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInventoryByLocationList {
  #[serde(rename = "Hardware")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hardware: Option<Vec<HWInventoryByLocation>>,
}

