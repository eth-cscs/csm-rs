use manta_backend_dispatcher::types::{
  ArtifactSummary as FrontEndArtifactSummary,
  ArtifactType as FrontEndArtifactType, HSNNICFRUInfo as FrontEndHSNNICFRUInfo,
  HSNNICLocationInfo as FrontEndHSNNICLocationInfo,
  HWInvByFRUHSNNIC as FrontEndHWInvByFRUHSNNIC,
  HWInvByFRUMemory as FrontEndHWInvByFRUMemory,
  HWInvByFRUNode as FrontEndHWInvByFRUNode,
  HWInvByFRUNodeAccel as FrontEndHWInvByFRUNodeAccel,
  HWInvByFRUProcessor as FrontEndHWInvByFRUProcessor,
  HWInvByLocHSNNIC as FrontEndHWInvByLocHSNNIC,
  HWInvByLocMemory as FrontEndHWInvByLocMemory,
  HWInvByLocNode as FrontEndHWInvByLocNode,
  HWInvByLocNodeAccel as FrontEndHWInvByLocNodeAccel,
  HWInvByLocProcessor as FrontEndHWInvByLocProcessor,
  HWInventory as FrontEndHWInventory,
  HWInventoryByLocation as FrontEndHWInventoryByLocation,
  HWInventoryByLocationList as FrontEndHWInventoryByLocationList,
  MemoryLocation as FrontEndMemoryLocation,
  MemorySummary as FrontEndMemorySummary,
  NodeLocationInfo as FrontEndNodeLocationInfo,
  NodeSummary as FrontEndNodeSummary,
  ProcessorSummary as FrontEndProcessorSummary,
  RedfishMemoryFRUInfo as FrontEndRedfishMemoryFRUInfo,
  RedfishMemoryLocationInfo as FrontEndRedfishMemoryLocationInfo,
  RedfishProcessorFRUInfo as FrontEndRedfishProcessorFRUInfo,
  RedfishProcessorLocationInfo as FrontEndRedfishProcessorLocationInfo,
  RedfishSystemFRUInfo as FrontEndRedfishSystemFRUInfo,
  RedfishSystemLocationInfo as FrontEndRedfishSystemLocationInfo,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use std::string::ToString;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString, IntoStaticStr};

///////////////////////////////////////////////////////////////////////////////
// MESA - These are nonr official structs created from 'curl' response payload
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

impl From<FrontEndArtifactType> for ArtifactType {
  fn from(value: FrontEndArtifactType) -> Self {
    match value {
      FrontEndArtifactType::Memory => ArtifactType::Memory,
      FrontEndArtifactType::Processor => ArtifactType::Processor,
      FrontEndArtifactType::NodeAccel => ArtifactType::NodeAccel,
      FrontEndArtifactType::NodeHsnNic => ArtifactType::NodeHsnNic,
      FrontEndArtifactType::Drive => ArtifactType::Drive,
      FrontEndArtifactType::CabinetPDU => ArtifactType::CabinetPDU,
      FrontEndArtifactType::CabinetPDUPowerConnector => {
        ArtifactType::CabinetPDUPowerConnector
      }
      FrontEndArtifactType::CMMRectifier => ArtifactType::CMMRectifier,
      FrontEndArtifactType::NodeAccelRiser => ArtifactType::NodeAccelRiser,
      FrontEndArtifactType::NodeEnclosurePowerSupplie => {
        ArtifactType::NodeEnclosurePowerSupplie
      }
      FrontEndArtifactType::NodeBMC => ArtifactType::NodeBMC,
      FrontEndArtifactType::RouterBMC => ArtifactType::RouterBMC,
    }
  }
}

impl From<ArtifactType> for FrontEndArtifactType {
  fn from(val: ArtifactType) -> Self {
    match val {
      ArtifactType::Memory => FrontEndArtifactType::Memory,
      ArtifactType::Processor => FrontEndArtifactType::Processor,
      ArtifactType::NodeAccel => FrontEndArtifactType::NodeAccel,
      ArtifactType::NodeHsnNic => FrontEndArtifactType::NodeHsnNic,
      ArtifactType::Drive => FrontEndArtifactType::Drive,
      ArtifactType::CabinetPDU => FrontEndArtifactType::CabinetPDU,
      ArtifactType::CabinetPDUPowerConnector => {
        FrontEndArtifactType::CabinetPDUPowerConnector
      }
      ArtifactType::CMMRectifier => FrontEndArtifactType::CMMRectifier,
      ArtifactType::NodeAccelRiser => FrontEndArtifactType::NodeAccelRiser,
      ArtifactType::NodeEnclosurePowerSupplie => {
        FrontEndArtifactType::NodeEnclosurePowerSupplie
      }
      ArtifactType::NodeBMC => FrontEndArtifactType::NodeBMC,
      ArtifactType::RouterBMC => FrontEndArtifactType::RouterBMC,
    }
  }
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

impl From<FrontEndNodeSummary> for NodeSummary {
  fn from(value: FrontEndNodeSummary) -> Self {
    NodeSummary {
      xname: value.xname,
      r#type: value.r#type,
      processors: value
        .processors
        .into_iter()
        .map(ArtifactSummary::from)
        .collect(),
      memory: value
        .memory
        .into_iter()
        .map(ArtifactSummary::from)
        .collect(),
      node_accels: value
        .node_accels
        .into_iter()
        .map(ArtifactSummary::from)
        .collect(),
      node_hsn_nics: value
        .node_hsn_nics
        .into_iter()
        .map(ArtifactSummary::from)
        .collect(),
    }
  }
}

impl From<NodeSummary> for FrontEndNodeSummary {
  fn from(val: NodeSummary) -> Self {
    FrontEndNodeSummary {
      xname: val.xname,
      r#type: val.r#type,
      processors: val
        .processors
        .into_iter()
        .map(ArtifactSummary::into)
        .collect(),
      memory: val.memory.into_iter().map(ArtifactSummary::into).collect(),
      node_accels: val
        .node_accels
        .into_iter()
        .map(ArtifactSummary::into)
        .collect(),
      node_hsn_nics: val
        .node_hsn_nics
        .into_iter()
        .map(ArtifactSummary::into)
        .collect(),
    }
  }
}

impl NodeSummary {
  pub fn from_csm_value(hw_artifact_value: Value) -> Self {
    let processors = hw_artifact_value
      .get("Processors")
      .and_then(Value::as_array)
      .map(|proc_vec| {
        proc_vec
          .iter()
          .map(|processor_value| {
            ArtifactSummary::from_processor_value(processor_value.clone())
          })
          .collect()
      })
      .unwrap_or_default();

    let memory = hw_artifact_value
      .get("Memory")
      .and_then(Value::as_array)
      .map(|mem_vec| {
        mem_vec
          .iter()
          .map(|memory_value| {
            ArtifactSummary::from_memory_value(memory_value.clone())
          })
          .collect()
      })
      .unwrap_or_default();

    let node_accels = hw_artifact_value
      .get("NodeAccels")
      .and_then(Value::as_array)
      .map(|nodeaccel_vec| {
        nodeaccel_vec
          .iter()
          .map(|nodeaccel_value| {
            ArtifactSummary::from_nodeaccel_value(nodeaccel_value.clone())
          })
          .collect()
      })
      .unwrap_or_default();

    let node_hsn_nics = hw_artifact_value
      .get("NodeHsnNics")
      .and_then(Value::as_array)
      .map(|hw_artifact_vec| {
        hw_artifact_vec
          .iter()
          .map(|nodehsnnic_value| {
            ArtifactSummary::from_nodehsnnics_value(nodehsnnic_value.clone())
          })
          .collect()
      })
      .unwrap_or_default();

    Self {
      xname: hw_artifact_value
        .get("ID")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap(),
      r#type: hw_artifact_value
        .get("Type")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap(),
      processors,
      memory,
      node_accels,
      node_hsn_nics,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtifactSummary {
  pub xname: String,
  pub r#type: ArtifactType,
  pub info: Option<String>,
}

impl From<FrontEndArtifactSummary> for ArtifactSummary {
  fn from(value: FrontEndArtifactSummary) -> Self {
    ArtifactSummary {
      xname: value.xname,
      r#type: value.r#type.into(),
      info: value.info,
    }
  }
}

impl From<ArtifactSummary> for FrontEndArtifactSummary {
  fn from(val: ArtifactSummary) -> Self {
    FrontEndArtifactSummary {
      xname: val.xname,
      r#type: val.r#type.into(),
      info: val.info,
    }
  }
}

impl ArtifactSummary {
  fn from_processor_value(processor_value: Value) -> Self {
    Self {
      xname: processor_value
        .get("ID")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap(),
      r#type: ArtifactType::from_str(
        processor_value.get("Type").and_then(Value::as_str).unwrap(),
      )
      .unwrap(),
      info: processor_value
        .pointer("/PopulatedFRU/ProcessorFRUInfo/Model")
        .and_then(Value::as_str)
        .map(str::to_string),
    }
  }

  fn from_memory_value(memory_value: Value) -> Self {
    Self {
      xname: memory_value
        .get("ID")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap(),
      r#type: ArtifactType::from_str(
        memory_value.get("Type").and_then(Value::as_str).unwrap(),
      )
      .unwrap(),
      info: memory_value
        .pointer("/PopulatedFRU/MemoryFRUInfo/CapacityMiB")
        .and_then(Value::as_number)
        .map(|v| v.to_string() + " MiB"),
    }
  }

  fn from_nodehsnnics_value(nodehsnnic_value: Value) -> Self {
    Self {
      xname: nodehsnnic_value
        .get("ID")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap(),
      r#type: ArtifactType::from_str(
        nodehsnnic_value
          .get("Type")
          .and_then(Value::as_str)
          .unwrap(),
      )
      .unwrap(),
      info: nodehsnnic_value
        .pointer("/NodeHsnNicLocationInfo/Description")
        .and_then(Value::as_str)
        .map(str::to_string),
    }
  }

  fn from_nodeaccel_value(nodeaccel_value: Value) -> Self {
    Self {
      xname: nodeaccel_value
        .get("ID")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap(),
      r#type: ArtifactType::from_str(
        nodeaccel_value.get("Type").and_then(Value::as_str).unwrap(),
      )
      .unwrap(),
      info: nodeaccel_value
        .pointer("/PopulatedFRU/NodeAccelFRUInfo/Model")
        .and_then(Value::as_str)
        .map(str::to_string),
    }
  }
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
  #[serde(rename(serialize = "InstructionSet"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub instruction_set: Option<String>,
  #[serde(rename(serialize = "Manufacturer"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub manufacturer: Option<String>,
  #[serde(rename(serialize = "MaxSpeedMHz"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_speed_mhz: Option<usize>,
  #[serde(rename(serialize = "Model"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub model: Option<String>,
  #[serde(rename(serialize = "ProcessorArchitecture"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processor_architecture: Option<String>,
  #[serde(rename(serialize = "ProcessorId"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processor_id: Option<ProcessorId>,
  #[serde(rename(serialize = "ProcessorType"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processor_type: Option<String>,
  #[serde(rename(serialize = "TotalCores"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total_cores: Option<usize>,
  #[serde(rename(serialize = "TotalThreads"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total_threads: Option<usize>,
}

impl From<FrontEndRedfishProcessorFRUInfo> for RedfishProcessorFRUInfo {
  fn from(value: FrontEndRedfishProcessorFRUInfo) -> Self {
    RedfishProcessorFRUInfo {
      instruction_set: value.instruction_set,
      manufacturer: value.manufacturer,
      max_speed_mhz: value.max_speed_mhz,
      model: value.model,
      processor_architecture: value.processor_architecture,
      processor_id: None,
      processor_type: value.processor_type,
      total_cores: value.total_cores,
      total_threads: value.total_threads,
    }
  }
}

impl From<RedfishProcessorFRUInfo> for FrontEndRedfishProcessorFRUInfo {
  fn from(val: RedfishProcessorFRUInfo) -> Self {
    FrontEndRedfishProcessorFRUInfo {
      instruction_set: val.instruction_set,
      manufacturer: val.manufacturer,
      max_speed_mhz: val.max_speed_mhz,
      model: val.model,
      processor_architecture: val.processor_architecture,
      processor_id: None, // FIXME: Implement From and Into traits for this field/type
      processor_type: val.processor_type,
      total_cores: val.total_cores,
      total_threads: val.total_threads,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByFRUProcessor {
  #[serde(rename(serialize = "FRUID"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_id: Option<String>,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "FRUSubType"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_sub_type: Option<String>,
  #[serde(rename(serialize = "HWInventoryByFRUType"))]
  pub hw_inventory_by_fru_type: String,
  #[serde(rename(serialize = "ProcessorFRUInfo"))]
  pub processor_fru_info: RedfishProcessorFRUInfo,
}

impl From<FrontEndHWInvByFRUProcessor> for HWInvByFRUProcessor {
  fn from(value: FrontEndHWInvByFRUProcessor) -> Self {
    HWInvByFRUProcessor {
      fru_id: value.fru_id,
      r#type: value.r#type,
      fru_sub_type: value.fru_sub_type,
      hw_inventory_by_fru_type: value.hw_inventory_by_fru_type,
      processor_fru_info: RedfishProcessorFRUInfo::from(
        value.processor_fru_info,
      ),
    }
  }
}

impl From<HWInvByFRUProcessor> for FrontEndHWInvByFRUProcessor {
  fn from(val: HWInvByFRUProcessor) -> Self {
    FrontEndHWInvByFRUProcessor {
      fru_id: val.fru_id,
      r#type: val.r#type,
      fru_sub_type: val.fru_sub_type,
      hw_inventory_by_fru_type: val.hw_inventory_by_fru_type,
      processor_fru_info: val.processor_fru_info.into(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishMemoryFRUInfo {
  #[serde(rename(serialize = "BaseModuleType"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub base_module_type: Option<String>,
  #[serde(rename(serialize = "BusWidthBits"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub bus_width_bits: Option<usize>,
  #[serde(rename(serialize = "CapacityMiB"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub capacity_mib: Option<usize>,
  #[serde(rename(serialize = "DataWidthBits"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data_width_bits: Option<usize>,
  #[serde(rename(serialize = "ErrorCorrection"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error_correction: Option<String>,
  #[serde(rename(serialize = "Manufacturer"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub manufacturer: Option<String>,
  #[serde(rename(serialize = "MemoryType"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_type: Option<String>,
  #[serde(rename(serialize = "MemoryDeviceType"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_device_type: Option<String>,
  #[serde(rename(serialize = "OperatingSpeedMhz"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub operating_speed_mhz: Option<usize>,
  #[serde(rename(serialize = "PartNumber"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub part_number: Option<String>,
  #[serde(rename(serialize = "RankCount"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rank_count: Option<usize>,
  #[serde(rename(serialize = "SerialNumber"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub serial_number: Option<String>,
}

impl From<FrontEndRedfishMemoryFRUInfo> for RedfishMemoryFRUInfo {
  fn from(value: FrontEndRedfishMemoryFRUInfo) -> Self {
    RedfishMemoryFRUInfo {
      base_module_type: value.base_module_type,
      bus_width_bits: value.bus_width_bits,
      capacity_mib: value.capacity_mib,
      data_width_bits: value.data_width_bits,
      error_correction: value.error_correction,
      manufacturer: value.manufacturer,
      memory_type: value.memory_type,
      memory_device_type: value.memory_device_type,
      operating_speed_mhz: value.operating_speed_mhz,
      part_number: value.part_number,
      rank_count: value.rank_count,
      serial_number: value.serial_number,
    }
  }
}

impl From<RedfishMemoryFRUInfo> for FrontEndRedfishMemoryFRUInfo {
  fn from(val: RedfishMemoryFRUInfo) -> Self {
    FrontEndRedfishMemoryFRUInfo {
      base_module_type: val.base_module_type,
      bus_width_bits: val.bus_width_bits,
      capacity_mib: val.capacity_mib,
      data_width_bits: val.data_width_bits,
      error_correction: val.error_correction,
      manufacturer: val.manufacturer,
      memory_type: val.memory_type,
      memory_device_type: val.memory_device_type,
      operating_speed_mhz: val.operating_speed_mhz,
      part_number: val.part_number,
      rank_count: val.rank_count,
      serial_number: val.serial_number,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByFRUMemory {
  #[serde(rename(serialize = "FRUID"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_id: Option<String>,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "FRUSubType"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_sub_type: Option<String>,
  #[serde(rename(serialize = "HWInventoryByFRUType"))]
  pub hw_inventory_by_fru_type: String,
  #[serde(rename(serialize = "MemoryFRUInfo"))]
  pub memory_fru_info: RedfishMemoryFRUInfo,
}

impl From<FrontEndHWInvByFRUMemory> for HWInvByFRUMemory {
  fn from(value: FrontEndHWInvByFRUMemory) -> Self {
    HWInvByFRUMemory {
      fru_id: value.fru_id,
      r#type: value.r#type,
      fru_sub_type: value.fru_sub_type,
      hw_inventory_by_fru_type: value.hw_inventory_by_fru_type,
      memory_fru_info: RedfishMemoryFRUInfo::from(value.memory_fru_info),
    }
  }
}

impl From<HWInvByFRUMemory> for FrontEndHWInvByFRUMemory {
  fn from(val: HWInvByFRUMemory) -> Self {
    FrontEndHWInvByFRUMemory {
      fru_id: val.fru_id,
      r#type: val.r#type,
      fru_sub_type: val.fru_sub_type,
      hw_inventory_by_fru_type: val.hw_inventory_by_fru_type,
      memory_fru_info: val.memory_fru_info.into(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByFRUNodeAccel {
  #[serde(rename(serialize = "FRUID"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_id: Option<String>,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "FRUSubType"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_sub_type: Option<String>,
  #[serde(rename(serialize = "HWInventoryByFRUType"))]
  pub hw_inventory_by_fru_type: String,
  #[serde(rename(serialize = "NodeAccelFRUInfo"))]
  pub node_accel_fru_info: RedfishProcessorFRUInfo, // NOTE: according to API
                                                    // docs, yes this is using the redfish for "processor"
}

impl From<FrontEndHWInvByFRUNodeAccel> for HWInvByFRUNodeAccel {
  fn from(value: FrontEndHWInvByFRUNodeAccel) -> Self {
    HWInvByFRUNodeAccel {
      fru_id: value.fru_id,
      r#type: value.r#type,
      fru_sub_type: value.fru_sub_type,
      hw_inventory_by_fru_type: value.hw_inventory_by_fru_type,
      node_accel_fru_info: RedfishProcessorFRUInfo::from(
        value.node_accel_fru_info,
      ),
    }
  }
}

impl From<HWInvByFRUNodeAccel> for FrontEndHWInvByFRUNodeAccel {
  fn from(val: HWInvByFRUNodeAccel) -> Self {
    FrontEndHWInvByFRUNodeAccel {
      fru_id: val.fru_id,
      r#type: val.r#type,
      fru_sub_type: val.fru_sub_type,
      hw_inventory_by_fru_type: val.hw_inventory_by_fru_type,
      node_accel_fru_info: val.node_accel_fru_info.into(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HSNNICFRUInfo {
  #[serde(rename(serialize = "Manufacturer"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub manufacturer: Option<String>,
  #[serde(rename(serialize = "Model"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub model: Option<String>,
  #[serde(rename(serialize = "PartNumber"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub part_number: Option<String>,
  #[serde(rename(serialize = "SKU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sku: Option<String>,
  #[serde(rename(serialize = "SerialNumber"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub serial_number: Option<String>,
}

impl From<FrontEndHSNNICFRUInfo> for HSNNICFRUInfo {
  fn from(value: FrontEndHSNNICFRUInfo) -> Self {
    HSNNICFRUInfo {
      manufacturer: value.manufacturer,
      model: value.model,
      part_number: value.part_number,
      sku: value.sku,
      serial_number: value.serial_number,
    }
  }
}

impl From<HSNNICFRUInfo> for FrontEndHSNNICFRUInfo {
  fn from(val: HSNNICFRUInfo) -> Self {
    FrontEndHSNNICFRUInfo {
      manufacturer: val.manufacturer,
      model: val.model,
      part_number: val.part_number,
      sku: val.sku,
      serial_number: val.serial_number,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByFRUHSNNIC {
  #[serde(rename(serialize = "FRUID"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_id: Option<String>,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "FRUSubType"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_sub_type: Option<String>,
  #[serde(rename(serialize = "HWInventoryByFRUType"))]
  pub hw_inventory_by_fru_type: String,
  #[serde(rename(serialize = "HSNNICFRUInfo"))]
  pub hsn_nic_fru_info: HSNNICFRUInfo,
}

impl From<FrontEndHWInvByFRUHSNNIC> for HWInvByFRUHSNNIC {
  fn from(value: FrontEndHWInvByFRUHSNNIC) -> Self {
    HWInvByFRUHSNNIC {
      fru_id: value.fru_id,
      r#type: value.r#type,
      fru_sub_type: value.fru_sub_type,
      hw_inventory_by_fru_type: value.hw_inventory_by_fru_type,
      hsn_nic_fru_info: HSNNICFRUInfo::from(value.hsn_nic_fru_info),
    }
  }
}

impl From<HWInvByFRUHSNNIC> for FrontEndHWInvByFRUHSNNIC {
  fn from(val: HWInvByFRUHSNNIC) -> Self {
    FrontEndHWInvByFRUHSNNIC {
      fru_id: val.fru_id,
      r#type: val.r#type,
      fru_sub_type: val.fru_sub_type,
      hw_inventory_by_fru_type: val.hw_inventory_by_fru_type,
      hsn_nic_fru_info: val.hsn_nic_fru_info.into(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInventoryByFRU {
  #[serde(rename(serialize = "FRUID"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_id: Option<String>,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "FRUSubType"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_sub_type: Option<String>,
  #[serde(rename(serialize = "HWInventoryByFRUType"))]
  pub hw_inventory_by_fru_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishChassisLocationInfo {
  #[serde(rename(serialize = "Id"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename(serialize = "Name"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename(serialize = "Description"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename(serialize = "Hostname"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hostname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocChassis {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "ChassisLocatinInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub chassis_location_info: Option<RedfishChassisLocationInfo>,
  #[serde(rename(serialize = "ComputeModules"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub compute_modules: Option<HWInvByLocComputeModule>,
  #[serde(rename(serialize = "RouterModules"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub router_modules: Option<HWInvByLocRouterModule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocNodeEnclosure {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "NodeEnclosureLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_enclosure_location_info: Option<RedfishChassisLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocComputeModule {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "ComputeModuleLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub compute_module_location_info: Option<RedfishChassisLocationInfo>,
  #[serde(rename(serialize = "NodeEnclosures"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_enclosures: Option<HWInvByLocNodeEnclosure>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocHSNBoard {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "HSNBoardLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hsn_board_location_info: Option<RedfishChassisLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocRouterModule {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "RouterModuleLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub router_module_location_info: Option<RedfishChassisLocationInfo>,
  pub hsn_boards: Option<HWInvByLocHSNBoard>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocCabinet {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "CabinetLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cabinet_location_info: Option<RedfishChassisLocationInfo>,
  #[serde(rename(serialize = "Chassis"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub chassis: Option<HWInvByLocChassis>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocMgmtSwitch {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "MgmtSwitchLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mgmt_switch_location_info: Option<RedfishChassisLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocMgmtHLSwitch {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "MgmtHLSwitchLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mgmt_hl_switch_location_info: Option<RedfishChassisLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocCDUMgmtSwitch {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "CDUMgmtSwitchLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cdu_mgmt_switch_location_info: Option<RedfishChassisLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessorSummary {
  #[serde(rename(serialize = "Count"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  count: Option<u32>,
  #[serde(rename(serialize = "Model"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  model: Option<String>,
}

impl From<FrontEndProcessorSummary> for ProcessorSummary {
  fn from(value: FrontEndProcessorSummary) -> Self {
    ProcessorSummary {
      count: value.count,
      model: value.model,
    }
  }
}

impl From<ProcessorSummary> for FrontEndProcessorSummary {
  fn from(val: ProcessorSummary) -> Self {
    FrontEndProcessorSummary {
      count: val.count,
      model: val.model,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemorySummary {
  #[serde(rename(serialize = "TotalSystemMemoryGiB"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub total_system_memory_gib: Option<u32>,
}

impl From<FrontEndMemorySummary> for MemorySummary {
  fn from(value: FrontEndMemorySummary) -> Self {
    MemorySummary {
      total_system_memory_gib: value.total_system_memory_gib,
    }
  }
}

impl From<MemorySummary> for FrontEndMemorySummary {
  fn from(val: MemorySummary) -> Self {
    FrontEndMemorySummary {
      total_system_memory_gib: val.total_system_memory_gib,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishSystemLocationInfo {
  #[serde(rename(serialize = "Id"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename(serialize = "Name"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename(serialize = "Description"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename(serialize = "Hostname"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hostname: Option<String>,
  #[serde(rename(serialize = "ProcessorSummary"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processor_summary: Option<ProcessorSummary>,
  #[serde(rename(serialize = "MemorySummary"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_summary: Option<MemorySummary>,
}

impl From<FrontEndRedfishSystemLocationInfo> for RedfishSystemLocationInfo {
  fn from(value: FrontEndRedfishSystemLocationInfo) -> Self {
    RedfishSystemLocationInfo {
      id: value.id,
      name: value.name,
      description: value.description,
      hostname: value.hostname,
      processor_summary: value.processor_summary.map(ProcessorSummary::from),
      memory_summary: value.memory_summary.map(MemorySummary::from),
    }
  }
}

impl From<RedfishSystemLocationInfo> for FrontEndRedfishSystemLocationInfo {
  fn from(val: RedfishSystemLocationInfo) -> Self {
    FrontEndRedfishSystemLocationInfo {
      id: val.id,
      name: val.name,
      description: val.description,
      hostname: val.hostname,
      processor_summary: val.processor_summary.map(|v| v.into()),
      memory_summary: val.memory_summary.map(|v| v.into()),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishProcessorLocationInfo {
  #[serde(rename(serialize = "Id"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename(serialize = "Name"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename(serialize = "Description"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename(serialize = "Socket"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub socket: Option<String>,
}

impl From<FrontEndRedfishProcessorLocationInfo>
  for RedfishProcessorLocationInfo
{
  fn from(value: FrontEndRedfishProcessorLocationInfo) -> Self {
    RedfishProcessorLocationInfo {
      id: value.id,
      name: value.name,
      description: value.description,
      socket: value.socket,
    }
  }
}

impl From<RedfishProcessorLocationInfo>
  for FrontEndRedfishProcessorLocationInfo
{
  fn from(val: RedfishProcessorLocationInfo) -> Self {
    FrontEndRedfishProcessorLocationInfo {
      id: val.id,
      name: val.name,
      description: val.description,
      socket: val.socket,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocProcessor {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInvByFRUProcessor>,
  #[serde(rename(serialize = "ProcessorLocationInfo"))]
  pub processor_location_info: RedfishProcessorLocationInfo,
}

impl From<FrontEndHWInvByLocProcessor> for HWInvByLocProcessor {
  fn from(value: FrontEndHWInvByLocProcessor) -> Self {
    HWInvByLocProcessor {
      id: value.id,
      r#type: value.r#type,
      ordinal: value.ordinal,
      status: value.status,
      hw_inventory_by_location_type: value.hw_inventory_by_location_type,
      populated_fru: value.populated_fru.map(HWInvByFRUProcessor::from),
      processor_location_info: RedfishProcessorLocationInfo::from(
        value.processor_location_info,
      ),
    }
  }
}

impl From<HWInvByLocProcessor> for FrontEndHWInvByLocProcessor {
  fn from(val: HWInvByLocProcessor) -> Self {
    FrontEndHWInvByLocProcessor {
      id: val.id,
      r#type: val.r#type,
      ordinal: val.ordinal,
      status: val.status,
      hw_inventory_by_location_type: val.hw_inventory_by_location_type,
      populated_fru: val.populated_fru.map(|v| v.into()),
      processor_location_info: val.processor_location_info.into(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocNodeAccel {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInvByFRUNodeAccel>,
  #[serde(rename(serialize = "NodeAccelLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_accel_location_info: Option<RedfishProcessorLocationInfo>, // NOTE: according to API
                                                                      // docs, yes this is using the redfish for "processor""
}

impl From<FrontEndHWInvByLocNodeAccel> for HWInvByLocNodeAccel {
  fn from(value: FrontEndHWInvByLocNodeAccel) -> Self {
    HWInvByLocNodeAccel {
      id: value.id,
      r#type: value.r#type,
      ordinal: value.ordinal,
      status: value.status,
      hw_inventory_by_location_type: value.hw_inventory_by_location_type,
      populated_fru: value.populated_fru.map(HWInvByFRUNodeAccel::from),
      node_accel_location_info: value
        .node_accel_location_info
        .map(RedfishProcessorLocationInfo::from),
    }
  }
}

impl From<HWInvByLocNodeAccel> for FrontEndHWInvByLocNodeAccel {
  fn from(val: HWInvByLocNodeAccel) -> Self {
    FrontEndHWInvByLocNodeAccel {
      id: val.id,
      r#type: val.r#type,
      ordinal: val.ordinal,
      status: val.status,
      hw_inventory_by_location_type: val.hw_inventory_by_location_type,
      populated_fru: val.populated_fru.map(|v| v.into()),
      node_accel_location_info: val.node_accel_location_info.map(|v| v.into()),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishDriveLocationInfo {
  #[serde(rename(serialize = "Id"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename(serialize = "Name"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename(serialize = "Description"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocDrive {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "DriveLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub drive_location_info: Option<RedfishDriveLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryLocation {
  #[serde(rename(serialize = "Socket"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub socket: Option<u32>,
  #[serde(rename(serialize = "MemoryController"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_controller: Option<u32>,
  #[serde(rename(serialize = "Channel"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub channel: Option<u32>,
  #[serde(rename(serialize = "Slot"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub slot: Option<u32>,
}

impl From<FrontEndMemoryLocation> for MemoryLocation {
  fn from(value: FrontEndMemoryLocation) -> Self {
    MemoryLocation {
      socket: value.socket,
      memory_controller: value.memory_controller,
      channel: value.channel,
      slot: value.slot,
    }
  }
}

impl From<MemoryLocation> for FrontEndMemoryLocation {
  fn from(val: MemoryLocation) -> Self {
    FrontEndMemoryLocation {
      socket: val.socket,
      memory_controller: val.memory_controller,
      channel: val.channel,
      slot: val.slot,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishMemoryLocationInfo {
  #[serde(rename(serialize = "Id"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename(serialize = "Name"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename(serialize = "Description"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename(serialize = "MemoryLocation"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_location: Option<MemoryLocation>,
}

impl From<FrontEndRedfishMemoryLocationInfo> for RedfishMemoryLocationInfo {
  fn from(value: FrontEndRedfishMemoryLocationInfo) -> Self {
    RedfishMemoryLocationInfo {
      id: value.id,
      name: value.name,
      description: value.description,
      memory_location: value.memory_location.map(MemoryLocation::from),
    }
  }
}

impl From<RedfishMemoryLocationInfo> for FrontEndRedfishMemoryLocationInfo {
  fn from(val: RedfishMemoryLocationInfo) -> Self {
    FrontEndRedfishMemoryLocationInfo {
      id: val.id,
      name: val.name,
      description: val.description,
      memory_location: val.memory_location.map(|v| v.into()),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocMemory {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInvByFRUMemory>,
  #[serde(rename(serialize = "MemoryLocationInfo"))]
  pub memory_location_info: RedfishMemoryLocationInfo,
}

impl From<FrontEndHWInvByLocMemory> for HWInvByLocMemory {
  fn from(value: FrontEndHWInvByLocMemory) -> Self {
    HWInvByLocMemory {
      id: value.id,
      r#type: value.r#type,
      ordinal: value.ordinal,
      status: value.status,
      hw_inventory_by_location_type: value.hw_inventory_by_location_type,
      populated_fru: value.populated_fru.map(HWInvByFRUMemory::from),
      memory_location_info: RedfishMemoryLocationInfo::from(
        value.memory_location_info,
      ),
    }
  }
}

impl From<HWInvByLocMemory> for FrontEndHWInvByLocMemory {
  fn from(val: HWInvByLocMemory) -> Self {
    FrontEndHWInvByLocMemory {
      id: val.id,
      r#type: val.r#type,
      ordinal: val.ordinal,
      status: val.status,
      hw_inventory_by_location_type: val.hw_inventory_by_location_type,
      populated_fru: val.populated_fru.map(|v| v.into()),
      memory_location_info: val.memory_location_info.into(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishNodeAccelRiserLocationInfo {
  #[serde(rename(serialize = "Name"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename(serialize = "Description"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocNodeAccelRiser {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "NodeAccelRiserLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_accel_riser_location_info: Option<RedfishNodeAccelRiserLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HSNNICLocationInfo {
  #[serde(rename(serialize = "Id"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename(serialize = "Name"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename(serialize = "Description"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

impl From<FrontEndHSNNICLocationInfo> for HSNNICLocationInfo {
  fn from(value: FrontEndHSNNICLocationInfo) -> Self {
    HSNNICLocationInfo {
      id: value.id,
      name: value.name,
      description: value.description,
    }
  }
}

impl From<HSNNICLocationInfo> for FrontEndHSNNICLocationInfo {
  fn from(val: HSNNICLocationInfo) -> Self {
    FrontEndHSNNICLocationInfo {
      id: val.id,
      name: val.name,
      description: val.description,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocHSNNIC {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInvByFRUHSNNIC>,
  /* #[serde(rename = "NodeHsnNicLocationInfo")]
  pub node_hsn_nic_location_info: HSNNICLocationInfo, */
  #[serde(rename = "HSNNICLocationInfo")]
  pub hsn_nic_location_info: HSNNICLocationInfo,
}

impl From<FrontEndHWInvByLocHSNNIC> for HWInvByLocHSNNIC {
  fn from(value: FrontEndHWInvByLocHSNNIC) -> Self {
    HWInvByLocHSNNIC {
      id: value.id,
      r#type: value.r#type,
      ordinal: value.ordinal,
      status: value.status,
      hw_inventory_by_location_type: value.hw_inventory_by_location_type,
      populated_fru: value.populated_fru.map(HWInvByFRUHSNNIC::from),
      hsn_nic_location_info: HSNNICLocationInfo::from(
        value.hsn_nic_location_info,
      ),
    }
  }
}

impl From<HWInvByLocHSNNIC> for FrontEndHWInvByLocHSNNIC {
  fn from(val: HWInvByLocHSNNIC) -> Self {
    FrontEndHWInvByLocHSNNIC {
      id: val.id,
      r#type: val.r#type,
      ordinal: val.ordinal,
      status: val.status,
      hw_inventory_by_location_type: val.hw_inventory_by_location_type,
      populated_fru: val.populated_fru.map(|v| v.into()),
      hsn_nic_location_info: val.hsn_nic_location_info.into(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hardware {
  #[serde(rename = "Hardware")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hardware: Option<Vec<HWInvByLocNode>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocNode {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInvByFRUNode>,
  #[serde(rename(serialize = "NodeLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_location_info: Option<RedfishSystemLocationInfo>,
  #[serde(rename(serialize = "Processors"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processors: Option<Vec<HWInvByLocProcessor>>,
  #[serde(rename(serialize = "NodeAccels"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_accels: Option<Vec<HWInvByLocNodeAccel>>,
  #[serde(rename(serialize = "Dives"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub drives: Option<Vec<HWInvByLocDrive>>,
  #[serde(rename(serialize = "Memory"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory: Option<Vec<HWInvByLocMemory>>,
  #[serde(rename(serialize = "NodeAccelRisers"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_accel_risers: Option<Vec<HWInvByLocNodeAccelRiser>>,
  #[serde(rename(serialize = "NodeHsnNICs"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_hsn_nics: Option<Vec<HWInvByLocHSNNIC>>,
}

impl From<FrontEndHWInvByLocNode> for HWInvByLocNode {
  fn from(value: FrontEndHWInvByLocNode) -> Self {
    HWInvByLocNode {
      id: value.id,
      r#type: value.r#type,
      ordinal: value.ordinal,
      status: value.status,
      hw_inventory_by_location_type: value.hw_inventory_by_location_type,
      populated_fru: value.populated_fru.map(HWInvByFRUNode::from),
      node_location_info: value
        .node_location_info
        .map(RedfishSystemLocationInfo::from),
      processors: value.processors.map(|processor_vec| {
        processor_vec
          .into_iter()
          .map(HWInvByLocProcessor::from)
          .collect()
      }),
      node_accels: value.node_accels.map(|node_accel_vec| {
        node_accel_vec
          .into_iter()
          .map(HWInvByLocNodeAccel::from)
          .collect()
      }),
      drives: None,
      memory: value.memory.map(|memory_vec| {
        memory_vec.into_iter().map(HWInvByLocMemory::from).collect()
      }),
      node_accel_risers: None,
      node_hsn_nics: value.node_hsn_nics.map(|node_hsn_nic_vec| {
        node_hsn_nic_vec
          .into_iter()
          .map(HWInvByLocHSNNIC::from)
          .collect()
      }),
    }
  }
}

impl From<HWInvByLocNode> for FrontEndHWInvByLocNode {
  fn from(val: HWInvByLocNode) -> Self {
    FrontEndHWInvByLocNode {
      id: val.id,
      r#type: val.r#type,
      ordinal: val.ordinal,
      status: val.status,
      hw_inventory_by_location_type: val.hw_inventory_by_location_type,
      populated_fru: val.populated_fru.map(|v| v.into()),
      node_location_info: val.node_location_info.map(|v| v.into()),
      processors: val.processors.map(|processor_vec| {
        processor_vec
          .into_iter()
          .map(|processor| processor.into())
          .collect()
      }),
      node_accels: val.node_accels.map(|node_accel_vec| {
        node_accel_vec
          .into_iter()
          .map(|node_accel| node_accel.into())
          .collect()
      }),
      drives: None,
      memory: val.memory.map(|memory_vec| {
        memory_vec.into_iter().map(|memory| memory.into()).collect()
      }),
      node_accel_risers: None,
      node_hsn_nics: val.node_hsn_nics.map(|node_hsn_nic_vec| {
        node_hsn_nic_vec.into_iter().map(|v| v.into()).collect()
      }),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishPDULocationInfo {
  #[serde(rename(serialize = "Id"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename(serialize = "Name"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename(serialize = "Description"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename(serialize = "UUID"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishOutletLocationInfo {
  #[serde(rename(serialize = "Id"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename(serialize = "Name"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename(serialize = "Description"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocOutlet {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "OutletLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub outlet_location_info: Option<RedfishOutletLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocPDU {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "PDULocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pdu_location_info: Option<RedfishPDULocationInfo>,
  #[serde(rename(serialize = "CabinetPDUPowerConnectors"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cabinet_pdu_power_connectors: Option<Vec<HWInvByLocOutlet>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishCMMRectifierLocationInfo {
  #[serde(rename(serialize = "Name"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename(serialize = "FirmwareVersion"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub firmware_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocCMMRectifier {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "CMMRectifierLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  cmm_rectifier_location_info: Option<RedfishCMMRectifierLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishNodeEnclosurePowerSupplyLocationInfo {
  #[serde(rename(serialize = "Name"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename(serialize = "FirmwareVersion"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub firmware_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocNodePowerSupply {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "NodeEnclosurePowerSupplyLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_enclosure_power_supply_location_info:
    Option<RedfishNodeEnclosurePowerSupplyLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishManagerLocationInfo {
  #[serde(rename(serialize = "Id"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename(serialize = "Name"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename(serialize = "Description"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename(serialize = "DateTime"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub date_time: Option<String>,
  #[serde(rename(serialize = "DateTimeLocalOffset"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub date_time_local_offset: Option<String>,
  #[serde(rename(serialize = "FirmwareVersion"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub firmware_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocNodeBMC {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "NodeBMCLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_bmc_location_info: Option<RedfishManagerLocationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByLocRouterBMC {
  #[serde(rename(serialize = "ID"))]
  pub id: String,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "Ordinal"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ordinal: Option<u32>,
  #[serde(rename(serialize = "Status"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(rename(serialize = "HWInventoryByLocationType"))]
  pub hw_inventory_by_location_type: String,
  #[serde(rename(serialize = "PopulatedFRU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub populated_fru: Option<HWInventoryByFRU>,
  #[serde(rename(serialize = "RouterBMCLocationInfo"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub router_bmc_location_info: Option<RedfishManagerLocationInfo>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInventory {
  #[serde(rename(serialize = "XName"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub xname: Option<String>,
  #[serde(rename(serialize = "Format"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub format: Option<String>,
  #[serde(rename(serialize = "Cabinets"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cabinets: Option<Vec<HWInvByLocCabinet>>,
  #[serde(rename(serialize = "Chassis"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub chassis: Option<Vec<HWInvByLocChassis>>,
  #[serde(rename(serialize = "ComputeModules"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub compute_modules: Option<Vec<HWInvByLocComputeModule>>,
  #[serde(rename(serialize = "RouterModules"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub router_modules: Option<Vec<HWInvByLocRouterModule>>,
  #[serde(rename(serialize = "NodeEnclosures"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_enclosures: Option<Vec<HWInvByLocNodeEnclosure>>,
  #[serde(rename(serialize = "HSNBoards"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hsn_boards: Option<Vec<HWInvByLocHSNBoard>>,
  #[serde(rename(serialize = "MgmtSwitches"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mgmt_switches: Option<Vec<HWInvByLocMgmtSwitch>>,
  #[serde(rename(serialize = "MgmtHLSwitches"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mgmt_hl_switches: Option<Vec<HWInvByLocMgmtHLSwitch>>,
  #[serde(rename(serialize = "CDUMgmtSwitches"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cdu_mgmt_switches: Option<Vec<HWInvByLocCDUMgmtSwitch>>,
  #[serde(rename(serialize = "Nodes"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub nodes: Option<Vec<HWInvByLocNode>>,
  #[serde(rename(serialize = "Processors"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub processors: Option<Vec<HWInvByLocProcessor>>,
  #[serde(rename(serialize = "NodeAccels"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_accels: Option<Vec<HWInvByLocNodeAccel>>,
  #[serde(rename(serialize = "Drives"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub drives: Option<Vec<HWInvByLocDrive>>,
  #[serde(rename(serialize = "Memory"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory: Option<Vec<HWInvByLocMemory>>,
  #[serde(rename(serialize = "CabinetPDUs"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cabinet_pdus: Option<Vec<HWInvByLocPDU>>,
  #[serde(rename(serialize = "CabinetPDUPowerConnectors"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cabinet_pdu_power_connectors: Option<Vec<HWInvByLocOutlet>>,
  #[serde(rename(serialize = "CMMRectifiers"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cmm_rectifiers: Option<Vec<HWInvByLocCMMRectifier>>,
  #[serde(rename(serialize = "NodeAccelRisers"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_accel_risers: Option<Vec<HWInvByLocNodeAccelRiser>>,
  #[serde(rename(serialize = "NodeHsnNICs"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_hsn_nics: Option<Vec<HWInvByLocHSNNIC>>,
  #[serde(rename(serialize = "NodeEnclosurePowerSupplies"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_enclosure_power_supplies: Option<Vec<HWInvByLocNodePowerSupply>>,
  #[serde(rename(serialize = "NodeBMC"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_bmc: Option<Vec<HWInvByLocNodeBMC>>,
  #[serde(rename(serialize = "RouterBMC"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub router_bmc: Option<Vec<HWInvByLocRouterBMC>>,
}

impl From<FrontEndHWInventory> for HWInventory {
  fn from(value: FrontEndHWInventory) -> Self {
    HWInventory {
      xname: value.xname,
      format: value.format,
      cabinets: None, // FIXME: Implement From and Into traits for this field/type
      chassis: None, // FIXME: Implement From and Into traits for this field/type
      compute_modules: None, // FIXME: Implement From and Into traits for this field/type
      router_modules: None, // FIXME: Implement From and Into traits for this field/type
      node_enclosures: None, // FIXME: Implement From and Into traits for this field/type
      hsn_boards: None, // FIXME: Implement From and Into traits for this field/type
      mgmt_switches: None, // FIXME: Implement From and Into traits for this field/type
      mgmt_hl_switches: None, // FIXME: Implement From and Into traits for this field/type
      cdu_mgmt_switches: None, // FIXME: Implement From and Into traits for this field/type
      nodes: value.nodes.map(|node_vec| {
        node_vec.into_iter().map(HWInvByLocNode::from).collect()
      }),
      processors: value.processors.map(|processor_vec| {
        processor_vec
          .into_iter()
          .map(HWInvByLocProcessor::from)
          .collect()
      }),
      node_accels: value.node_accels.map(|node_accel_vec| {
        node_accel_vec
          .into_iter()
          .map(HWInvByLocNodeAccel::from)
          .collect()
      }),
      drives: None, // FIXME: Implement From and Into traits for this field/type
      memory: value.memory.map(|memory_vec| {
        memory_vec.into_iter().map(HWInvByLocMemory::from).collect()
      }),
      cabinet_pdus: None, // FIXME: Implement From and Into traits for this field/type
      cabinet_pdu_power_connectors: None, // FIXME: Implement From and Into traits for this field/type
      cmm_rectifiers: None, // FIXME: Implement From and Into traits for this field/type
      node_accel_risers: None, // FIXME: Implement From and Into traits for this field/type
      node_hsn_nics: None, // FIXME: Implement From and Into traits for this field/type
      node_enclosure_power_supplies: None, // FIXME: Implement From and Into traits for this field/type
      node_bmc: None, // FIXME: Implement From and Into traits for this field/type
      router_bmc: None, // FIXME: Implement From and Into traits for this field/type
    }
  }
}

impl From<HWInventory> for FrontEndHWInventory {
  fn from(val: HWInventory) -> Self {
    FrontEndHWInventory {
      xname: val.xname,
      format: val.format,
      cabinets: None, // FIXME: Implement From and Into traits for this field/type
      chassis: None, // FIXME: Implement From and Into traits for this field/type
      compute_modules: None, // FIXME: Implement From and Into traits for this field/type
      router_modules: None, // FIXME: Implement From and Into traits for this field/type
      node_enclosures: None, // FIXME: Implement From and Into traits for this field/type
      hsn_boards: None, // FIXME: Implement From and Into traits for this field/type
      mgmt_switches: None, // FIXME: Implement From and Into traits for this field/type
      mgmt_hl_switches: None, // FIXME: Implement From and Into traits for this field/type
      cdu_mgmt_switches: None, // FIXME: Implement From and Into traits for this field/type
      nodes: val
        .nodes
        .map(|node_vec| node_vec.into_iter().map(|node| node.into()).collect()),
      processors: val.processors.map(|processor_vec| {
        processor_vec
          .into_iter()
          .map(|processor| processor.into())
          .collect()
      }),
      node_accels: val.node_accels.map(|node_accel_vec| {
        node_accel_vec
          .into_iter()
          .map(|node_accel| node_accel.into())
          .collect()
      }),
      drives: None, // FIXME: Implement From and Into traits for this field/type
      memory: val.memory.map(|memory_vec| {
        memory_vec.into_iter().map(|memory| memory.into()).collect()
      }),
      cabinet_pdus: None, // FIXME: Implement From and Into traits for this field/type
      cabinet_pdu_power_connectors: None, // FIXME: Implement From and Into traits for this field/type
      cmm_rectifiers: None, // FIXME: Implement From and Into traits for this field/type
      node_accel_risers: None, // FIXME: Implement From and Into traits for this field/type
      node_hsn_nics: None, // FIXME: Implement From and Into traits for this field/type
      node_enclosure_power_supplies: None, // FIXME: Implement From and Into traits for this field/type
      node_bmc: None, // FIXME: Implement From and Into traits for this field/type
      router_bmc: None, // FIXME: Implement From and Into traits for this field/type
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInvByFRUNode {
  #[serde(rename(serialize = "FRUID"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_id: Option<String>,
  #[serde(rename(serialize = "Type"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(rename(serialize = "FRUSubType"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fru_sub_type: Option<String>,
  #[serde(rename(serialize = "HWInventoryByFRUType"))]
  pub hw_inventory_by_fru_type: String,
  #[serde(rename(serialize = "NodeFRUInfo"))]
  pub node_fru_info: RedfishSystemFRUInfo,
}

impl From<FrontEndHWInvByFRUNode> for HWInvByFRUNode {
  fn from(value: FrontEndHWInvByFRUNode) -> Self {
    HWInvByFRUNode {
      fru_id: value.fru_id,
      r#type: value.r#type,
      fru_sub_type: value.fru_sub_type,
      hw_inventory_by_fru_type: value.hw_inventory_by_fru_type,
      node_fru_info: RedfishSystemFRUInfo::from(value.node_fru_info),
    }
  }
}

impl From<HWInvByFRUNode> for FrontEndHWInvByFRUNode {
  fn from(val: HWInvByFRUNode) -> Self {
    FrontEndHWInvByFRUNode {
      fru_id: val.fru_id,
      r#type: val.r#type,
      fru_sub_type: val.fru_sub_type,
      hw_inventory_by_fru_type: val.hw_inventory_by_fru_type,
      node_fru_info: val.node_fru_info.into(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedfishSystemFRUInfo {
  #[serde(rename(serialize = "AssetTag"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub asset_tag: Option<String>,
  #[serde(rename(serialize = "BiosVersion"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub bios_version: Option<String>,
  #[serde(rename(serialize = "Model"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub model: Option<String>,
  #[serde(rename(serialize = "Manufacturer"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub manufacturer: Option<String>,
  #[serde(rename(serialize = "PartNumber"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub part_number: Option<String>,
  #[serde(rename(serialize = "SerialNumber"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub serial_number: Option<String>,
  #[serde(rename(serialize = "SKU"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sku: Option<String>,
  #[serde(rename(serialize = "SystemType"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub system_type: Option<String>,
  #[serde(rename(serialize = "UUID"))]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub uuid: Option<String>,
}

impl From<FrontEndRedfishSystemFRUInfo> for RedfishSystemFRUInfo {
  fn from(value: FrontEndRedfishSystemFRUInfo) -> Self {
    RedfishSystemFRUInfo {
      asset_tag: value.asset_tag,
      bios_version: value.bios_version,
      model: value.model,
      manufacturer: value.manufacturer,
      part_number: value.part_number,
      serial_number: value.serial_number,
      sku: value.sku,
      system_type: value.system_type,
      uuid: value.uuid,
    }
  }
}

impl From<RedfishSystemFRUInfo> for FrontEndRedfishSystemFRUInfo {
  fn from(val: RedfishSystemFRUInfo) -> Self {
    FrontEndRedfishSystemFRUInfo {
      asset_tag: val.asset_tag,
      bios_version: val.bios_version,
      model: val.model,
      manufacturer: val.manufacturer,
      part_number: val.part_number,
      serial_number: val.serial_number,
      sku: val.sku,
      system_type: val.system_type,
      uuid: val.uuid,
    }
  }
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

impl From<FrontEndNodeLocationInfo> for NodeLocationInfo {
  fn from(value: FrontEndNodeLocationInfo) -> Self {
    NodeLocationInfo {
      id: value.id,
      name: value.name,
      description: value.description,
      hostname: value.hostname,
      processor_summary: value.processor_summary.map(ProcessorSummary::from),
      memory_summary: value.memory_summary.map(MemorySummary::from),
    }
  }
}

impl From<NodeLocationInfo> for FrontEndNodeLocationInfo {
  fn from(val: NodeLocationInfo) -> Self {
    FrontEndNodeLocationInfo {
      id: val.id,
      name: val.name,
      description: val.description,
      hostname: val.hostname,
      processor_summary: val.processor_summary.map(|value| value.into()),
      memory_summary: val.memory_summary.map(|value| value.into()),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
pub enum HWInventoryByLocation {
  HWInvByLocNode(HWInvByLocNode),
  HWInvByLocProcessor(HWInvByLocProcessor),
  HWInvByLocNodeAccel(HWInvByLocNodeAccel),
  HWInvByLocMemory(HWInvByLocMemory),
  HWInvByLocHSNNIC(HWInvByLocHSNNIC),
}

impl From<FrontEndHWInventoryByLocation> for HWInventoryByLocation {
  fn from(f: FrontEndHWInventoryByLocation) -> Self {
    match f {
      FrontEndHWInventoryByLocation::HWInvByLocNode(hwinv_by_loc_nnode) => {
        HWInventoryByLocation::HWInvByLocNode(HWInvByLocNode::from(
          hwinv_by_loc_nnode,
        ))
      }
      FrontEndHWInventoryByLocation::HWInvByLocProcessor(
        hwinv_by_loc_processor,
      ) => HWInventoryByLocation::HWInvByLocProcessor(
        HWInvByLocProcessor::from(hwinv_by_loc_processor),
      ),
      FrontEndHWInventoryByLocation::HWInvByLocNodeAccel(
        hwinv_by_node_accel,
      ) => HWInventoryByLocation::HWInvByLocNodeAccel(
        HWInvByLocNodeAccel::from(hwinv_by_node_accel),
      ),
      FrontEndHWInventoryByLocation::HWInvByLocMemory(hwinv_by_loc_memory) => {
        HWInventoryByLocation::HWInvByLocMemory(HWInvByLocMemory::from(
          hwinv_by_loc_memory,
        ))
      }
      FrontEndHWInventoryByLocation::HWInvByLocHSNNIC(hwinv_by_loc_hsnnic) => {
        HWInventoryByLocation::HWInvByLocHSNNIC(HWInvByLocHSNNIC::from(
          hwinv_by_loc_hsnnic,
        ))
      }
    }
  }
}

impl From<HWInventoryByLocation> for FrontEndHWInventoryByLocation {
  fn from(val: HWInventoryByLocation) -> Self {
    match val {
      HWInventoryByLocation::HWInvByLocNode(f) => {
        FrontEndHWInventoryByLocation::HWInvByLocNode(f.into())
      }
      HWInventoryByLocation::HWInvByLocProcessor(f) => {
        FrontEndHWInventoryByLocation::HWInvByLocProcessor(f.into())
      }
      HWInventoryByLocation::HWInvByLocNodeAccel(f) => {
        FrontEndHWInventoryByLocation::HWInvByLocNodeAccel(f.into())
      }
      HWInventoryByLocation::HWInvByLocMemory(f) => {
        FrontEndHWInventoryByLocation::HWInvByLocMemory(f.into())
      }
      HWInventoryByLocation::HWInvByLocHSNNIC(hwinv_by_loc_hsnnic) => {
        FrontEndHWInventoryByLocation::HWInvByLocHSNNIC(
          hwinv_by_loc_hsnnic.into(),
        )
      }
    }
  }
}

/// struct used in POST and GET endpoints that manage multiple instances of 'HWInventoryByLocation'
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HWInventoryByLocationList {
  #[serde(rename = "Hardware")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hardware: Option<Vec<HWInventoryByLocation>>,
}

impl From<FrontEndHWInventoryByLocationList> for HWInventoryByLocationList {
  fn from(value: FrontEndHWInventoryByLocationList) -> Self {
    HWInventoryByLocationList {
      hardware: value.hardware.map(|hardware_vec| {
        hardware_vec
          .into_iter()
          .map(|hardware_inventory_by_location| {
            hardware_inventory_by_location.into()
          })
          .collect()
      }),
    }
  }
}

impl From<HWInventoryByLocationList> for FrontEndHWInventoryByLocationList {
  fn from(val: HWInventoryByLocationList) -> Self {
    FrontEndHWInventoryByLocationList {
      hardware: val.hardware.map(|hardware_vec| {
        hardware_vec
          .into_iter()
          .map(|hardware_inventory_by_location| {
            hardware_inventory_by_location.into()
          })
          .collect()
      }),
    }
  }
}
