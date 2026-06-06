//! Bidirectional `From` impls between csm-rs's HSM hardware-component
//! types and the dispatcher's mirrors. Gated behind the
//! `manta-dispatcher` Cargo feature so users not on Manta don't pull the
//! dispatcher dep.
//!
//! The `bidirectional_from*` macros used below are defined in
//! [`super::types`]; the parent `mod.rs` brings them into scope here via
//! `#[macro_use] pub mod types;`.

use manta_backend_dispatcher::types::{
  ArtifactSummary as FrontEndArtifactSummary,
  ArtifactType as FrontEndArtifactType, HSNNICFRUInfo as FrontEndHSNNICFRUInfo,
  HSNNICLocationInfo as FrontEndHSNNICLocationInfo,
  HWInvByFRUHSNNIC as FrontEndHWInvByFRUHSNNIC,
  HWInvByFRUMemory as FrontEndHWInvByFRUMemory,
  HWInvByFRUNode as FrontEndHWInvByFRUNode,
  HWInvByFRUNodeAccel as FrontEndHWInvByFRUNodeAccel,
  HWInvByFRUProcessor as FrontEndHWInvByFRUProcessor,
  HWInvByLocCDUMgmtSwitch as FrontEndHWInvByLocCDUMgmtSwitch,
  HWInvByLocCMMRectifier as FrontEndHWInvByLocCMMRectifier,
  HWInvByLocCabinet as FrontEndHWInvByLocCabinet,
  HWInvByLocChassis as FrontEndHWInvByLocChassis,
  HWInvByLocComputeModule as FrontEndHWInvByLocComputeModule,
  HWInvByLocDrive as FrontEndHWInvByLocDrive,
  HWInvByLocHSNBoard as FrontEndHWInvByLocHSNBoard,
  HWInvByLocHSNNIC as FrontEndHWInvByLocHSNNIC,
  HWInvByLocMemory as FrontEndHWInvByLocMemory,
  HWInvByLocMgmtHLSwitch as FrontEndHWInvByLocMgmtHLSwitch,
  HWInvByLocMgmtSwitch as FrontEndHWInvByLocMgmtSwitch,
  HWInvByLocNode as FrontEndHWInvByLocNode,
  HWInvByLocNodeAccel as FrontEndHWInvByLocNodeAccel,
  HWInvByLocNodeAccelRiser as FrontEndHWInvByLocNodeAccelRiser,
  HWInvByLocNodeBMC as FrontEndHWInvByLocNodeBMC,
  HWInvByLocNodeEnclosure as FrontEndHWInvByLocNodeEnclosure,
  HWInvByLocNodePowerSupply as FrontEndHWInvByLocNodePowerSupply,
  HWInvByLocOutlet as FrontEndHWInvByLocOutlet,
  HWInvByLocPDU as FrontEndHWInvByLocPDU,
  HWInvByLocProcessor as FrontEndHWInvByLocProcessor,
  HWInvByLocRouterBMC as FrontEndHWInvByLocRouterBMC,
  HWInvByLocRouterModule as FrontEndHWInvByLocRouterModule,
  HWInventory as FrontEndHWInventory,
  HWInventoryByFRU as FrontEndHWInventoryByFRU,
  HWInventoryByLocation as FrontEndHWInventoryByLocation,
  HWInventoryByLocationList as FrontEndHWInventoryByLocationList,
  MemoryLocation as FrontEndMemoryLocation,
  MemorySummary as FrontEndMemorySummary,
  NodeLocationInfo as FrontEndNodeLocationInfo,
  NodeSummary as FrontEndNodeSummary,
  ProcessorSummary as FrontEndProcessorSummary,
  RedfishCMMRectifierLocationInfo as FrontEndRedfishCMMRectifierLocationInfo,
  RedfishChassisLocationInfo as FrontEndRedfishChassisLocationInfo,
  RedfishDriveLocationInfo as FrontEndRedfishDriveLocationInfo,
  RedfishManagerLocationInfo as FrontEndRedfishManagerLocationInfo,
  RedfishMemoryFRUInfo as FrontEndRedfishMemoryFRUInfo,
  RedfishMemoryLocationInfo as FrontEndRedfishMemoryLocationInfo,
  RedfishNodeAccelRiserLocationInfo as FrontEndRedfishNodeAccelRiserLocationInfo,
  RedfishNodeEnclosurePowerSupplyLocationInfo as FrontEndRedfishNodeEnclosurePowerSupplyLocationInfo,
  RedfishOutletLocationInfo as FrontEndRedfishOutletLocationInfo,
  RedfishPDULocationInfo as FrontEndRedfishPDULocationInfo,
  RedfishProcessorFRUInfo as FrontEndRedfishProcessorFRUInfo,
  RedfishProcessorLocationInfo as FrontEndRedfishProcessorLocationInfo,
  RedfishSystemFRUInfo as FrontEndRedfishSystemFRUInfo,
  RedfishSystemLocationInfo as FrontEndRedfishSystemLocationInfo,
};

use super::types::{
  ArtifactSummary, ArtifactType, HSNNICFRUInfo, HSNNICLocationInfo,
  HWInvByFRUHSNNIC, HWInvByFRUMemory, HWInvByFRUNode, HWInvByFRUNodeAccel,
  HWInvByFRUProcessor, HWInvByLocCDUMgmtSwitch, HWInvByLocCMMRectifier,
  HWInvByLocCabinet, HWInvByLocChassis, HWInvByLocComputeModule,
  HWInvByLocDrive, HWInvByLocHSNBoard, HWInvByLocHSNNIC, HWInvByLocMemory,
  HWInvByLocMgmtHLSwitch, HWInvByLocMgmtSwitch, HWInvByLocNode,
  HWInvByLocNodeAccel, HWInvByLocNodeAccelRiser, HWInvByLocNodeBMC,
  HWInvByLocNodeEnclosure, HWInvByLocNodePowerSupply, HWInvByLocOutlet,
  HWInvByLocPDU, HWInvByLocProcessor, HWInvByLocRouterBMC,
  HWInvByLocRouterModule, HWInventory, HWInventoryByFRU, HWInventoryByLocation,
  HWInventoryByLocationList, MemoryLocation, MemorySummary, NodeLocationInfo,
  NodeSummary, ProcessorSummary, RedfishCMMRectifierLocationInfo,
  RedfishChassisLocationInfo, RedfishDriveLocationInfo,
  RedfishManagerLocationInfo, RedfishMemoryFRUInfo, RedfishMemoryLocationInfo,
  RedfishNodeAccelRiserLocationInfo,
  RedfishNodeEnclosurePowerSupplyLocationInfo, RedfishOutletLocationInfo,
  RedfishPDULocationInfo, RedfishProcessorFRUInfo,
  RedfishProcessorLocationInfo, RedfishSystemFRUInfo,
  RedfishSystemLocationInfo,
};

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

bidirectional_from_mixed!(
  NodeSummary,
  FrontEndNodeSummary,
  direct: [xname, r#type],
  vec_into: [processors, memory, node_accels, node_hsn_nics],
);

bidirectional_from_into!(
  ArtifactSummary,
  FrontEndArtifactSummary,
  [xname, r#type, info]
);

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

bidirectional_from_into!(
  HWInvByFRUProcessor,
  FrontEndHWInvByFRUProcessor,
  [
    fru_id,
    r#type,
    fru_sub_type,
    hw_inventory_by_fru_type,
    processor_fru_info
  ]
);

bidirectional_from!(
  RedfishMemoryFRUInfo,
  FrontEndRedfishMemoryFRUInfo,
  [
    base_module_type,
    bus_width_bits,
    capacity_mib,
    data_width_bits,
    error_correction,
    manufacturer,
    memory_type,
    memory_device_type,
    operating_speed_mhz,
    part_number,
    rank_count,
    serial_number,
  ]
);

bidirectional_from_into!(
  HWInvByFRUMemory,
  FrontEndHWInvByFRUMemory,
  [
    fru_id,
    r#type,
    fru_sub_type,
    hw_inventory_by_fru_type,
    memory_fru_info
  ]
);

bidirectional_from_into!(
  HWInvByFRUNodeAccel,
  FrontEndHWInvByFRUNodeAccel,
  [
    fru_id,
    r#type,
    fru_sub_type,
    hw_inventory_by_fru_type,
    node_accel_fru_info
  ]
);

bidirectional_from!(
  HSNNICFRUInfo,
  FrontEndHSNNICFRUInfo,
  [manufacturer, model, part_number, sku, serial_number]
);

bidirectional_from_into!(
  HWInvByFRUHSNNIC,
  FrontEndHWInvByFRUHSNNIC,
  [
    fru_id,
    r#type,
    fru_sub_type,
    hw_inventory_by_fru_type,
    hsn_nic_fru_info
  ]
);

bidirectional_from!(ProcessorSummary, FrontEndProcessorSummary, [count, model]);

bidirectional_from!(
  MemorySummary,
  FrontEndMemorySummary,
  [total_system_memory_gib]
);

bidirectional_from_mixed!(
  RedfishSystemLocationInfo,
  FrontEndRedfishSystemLocationInfo,
  direct: [id, name, description, hostname],
  opt_into: [processor_summary, memory_summary],
);

bidirectional_from!(
  RedfishProcessorLocationInfo,
  FrontEndRedfishProcessorLocationInfo,
  [id, name, description, socket]
);

bidirectional_from_mixed!(
  HWInvByLocProcessor,
  FrontEndHWInvByLocProcessor,
  direct: [id, r#type, ordinal, status],
  into: [processor_location_info],
  opt_into: [populated_fru],
);

bidirectional_from_mixed!(
  HWInvByLocNodeAccel,
  FrontEndHWInvByLocNodeAccel,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, node_accel_location_info],
);

bidirectional_from!(
  MemoryLocation,
  FrontEndMemoryLocation,
  [socket, memory_controller, channel, slot]
);

bidirectional_from_mixed!(
  RedfishMemoryLocationInfo,
  FrontEndRedfishMemoryLocationInfo,
  direct: [id, name, description],
  opt_into: [memory_location],
);

bidirectional_from_mixed!(
  HWInvByLocMemory,
  FrontEndHWInvByLocMemory,
  direct: [id, r#type, ordinal, status],
  into: [memory_location_info],
  opt_into: [populated_fru],
);

bidirectional_from!(
  HSNNICLocationInfo,
  FrontEndHSNNICLocationInfo,
  [id, name, description]
);

bidirectional_from_mixed!(
  HWInvByLocHSNNIC,
  FrontEndHWInvByLocHSNNIC,
  direct: [id, r#type, ordinal, status],
  into: [hsn_nic_location_info],
  opt_into: [populated_fru],
);

impl From<FrontEndHWInvByLocNode> for HWInvByLocNode {
  fn from(value: FrontEndHWInvByLocNode) -> Self {
    HWInvByLocNode {
      id: value.id,
      r#type: value.r#type,
      ordinal: value.ordinal,
      status: value.status,
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

bidirectional_from_into!(
  HWInvByFRUNode,
  FrontEndHWInvByFRUNode,
  [
    fru_id,
    r#type,
    fru_sub_type,
    hw_inventory_by_fru_type,
    node_fru_info
  ]
);

bidirectional_from!(
  RedfishSystemFRUInfo,
  FrontEndRedfishSystemFRUInfo,
  [
    asset_tag,
    bios_version,
    model,
    manufacturer,
    part_number,
    serial_number,
    sku,
    system_type,
    uuid,
  ]
);

bidirectional_from_mixed!(
  NodeLocationInfo,
  FrontEndNodeLocationInfo,
  direct: [id, name, description, hostname],
  opt_into: [processor_summary, memory_summary],
);

// Supporting From impls for the additional HWInventoryByLocation enum
// variants (HWInvByLocChassis, HWInvByLocCabinet, etc.). The enum
// variants themselves and their leaf From impls are added separately;
// these are the shared building blocks each leaf depends on.
bidirectional_from!(
  HWInventoryByFRU,
  FrontEndHWInventoryByFRU,
  [fru_id, r#type, fru_sub_type, hw_inventory_by_fru_type]
);
bidirectional_from!(
  RedfishChassisLocationInfo,
  FrontEndRedfishChassisLocationInfo,
  [id, name, description, hostname]
);
bidirectional_from!(
  RedfishDriveLocationInfo,
  FrontEndRedfishDriveLocationInfo,
  [id, name, description]
);
bidirectional_from!(
  RedfishNodeAccelRiserLocationInfo,
  FrontEndRedfishNodeAccelRiserLocationInfo,
  [name, description]
);
bidirectional_from!(
  RedfishOutletLocationInfo,
  FrontEndRedfishOutletLocationInfo,
  [id, name, description]
);
bidirectional_from!(
  RedfishPDULocationInfo,
  FrontEndRedfishPDULocationInfo,
  [id, name, description, uuid]
);
bidirectional_from!(
  RedfishCMMRectifierLocationInfo,
  FrontEndRedfishCMMRectifierLocationInfo,
  [name, firmware_version]
);
bidirectional_from!(
  RedfishNodeEnclosurePowerSupplyLocationInfo,
  FrontEndRedfishNodeEnclosurePowerSupplyLocationInfo,
  [name, firmware_version]
);
bidirectional_from!(
  RedfishManagerLocationInfo,
  FrontEndRedfishManagerLocationInfo,
  [id, name, description, date_time, date_time_local_offset, firmware_version]
);

// Per-leaf From impls for the 17 HWInventoryByLocation variants not
// previously wired up. Each follows the same skeleton: id/type/ordinal
// /status/discriminator are `direct`; populated_fru and the per-type
// *_location_info are `opt_into`; any nested HWInvByLoc* field is also
// `opt_into`; HWInvByLocPDU's cabinet_pdu_power_connectors is
// `opt_vec_into`.
bidirectional_from_mixed!(
  HWInvByLocChassis,
  FrontEndHWInvByLocChassis,
  direct: [id, r#type, ordinal, status],
  opt_into: [
    populated_fru,
    chassis_location_info,
    compute_modules,
    router_modules,
  ],
);
bidirectional_from_mixed!(
  HWInvByLocNodeEnclosure,
  FrontEndHWInvByLocNodeEnclosure,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, node_enclosure_location_info],
);
bidirectional_from_mixed!(
  HWInvByLocComputeModule,
  FrontEndHWInvByLocComputeModule,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, compute_module_location_info, node_enclosures],
);
bidirectional_from_mixed!(
  HWInvByLocHSNBoard,
  FrontEndHWInvByLocHSNBoard,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, hsn_board_location_info],
);
bidirectional_from_mixed!(
  HWInvByLocRouterModule,
  FrontEndHWInvByLocRouterModule,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, router_module_location_info, hsn_boards],
);
bidirectional_from_mixed!(
  HWInvByLocCabinet,
  FrontEndHWInvByLocCabinet,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, cabinet_location_info, chassis],
);
bidirectional_from_mixed!(
  HWInvByLocMgmtSwitch,
  FrontEndHWInvByLocMgmtSwitch,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, mgmt_switch_location_info],
);
bidirectional_from_mixed!(
  HWInvByLocMgmtHLSwitch,
  FrontEndHWInvByLocMgmtHLSwitch,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, mgmt_hl_switch_location_info],
);
bidirectional_from_mixed!(
  HWInvByLocCDUMgmtSwitch,
  FrontEndHWInvByLocCDUMgmtSwitch,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, cdu_mgmt_switch_location_info],
);
bidirectional_from_mixed!(
  HWInvByLocDrive,
  FrontEndHWInvByLocDrive,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, drive_location_info],
);
bidirectional_from_mixed!(
  HWInvByLocNodeAccelRiser,
  FrontEndHWInvByLocNodeAccelRiser,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, node_accel_riser_location_info],
);
bidirectional_from_mixed!(
  HWInvByLocOutlet,
  FrontEndHWInvByLocOutlet,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, outlet_location_info],
);
bidirectional_from_mixed!(
  HWInvByLocPDU,
  FrontEndHWInvByLocPDU,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, pdu_location_info],
  opt_vec_into: [cabinet_pdu_power_connectors],
);
bidirectional_from_mixed!(
  HWInvByLocCMMRectifier,
  FrontEndHWInvByLocCMMRectifier,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, cmm_rectifier_location_info],
);
bidirectional_from_mixed!(
  HWInvByLocNodePowerSupply,
  FrontEndHWInvByLocNodePowerSupply,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, node_enclosure_power_supply_location_info],
);
bidirectional_from_mixed!(
  HWInvByLocNodeBMC,
  FrontEndHWInvByLocNodeBMC,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, node_bmc_location_info],
);
bidirectional_from_mixed!(
  HWInvByLocRouterBMC,
  FrontEndHWInvByLocRouterBMC,
  direct: [id, r#type, ordinal, status],
  opt_into: [populated_fru, router_bmc_location_info],
);

impl From<FrontEndHWInventoryByLocation> for HWInventoryByLocation {
  fn from(f: FrontEndHWInventoryByLocation) -> Self {
    match f {
      FrontEndHWInventoryByLocation::HWInvByLocCDUMgmtSwitch(v) => {
        HWInventoryByLocation::HWInvByLocCDUMgmtSwitch(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocCMMRectifier(v) => {
        HWInventoryByLocation::HWInvByLocCMMRectifier(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocCabinet(v) => {
        HWInventoryByLocation::HWInvByLocCabinet(Box::new(HWInvByLocCabinet::from(v)))
      }
      FrontEndHWInventoryByLocation::HWInvByLocChassis(v) => {
        HWInventoryByLocation::HWInvByLocChassis(Box::new(HWInvByLocChassis::from(v)))
      }
      FrontEndHWInventoryByLocation::HWInvByLocComputeModule(v) => {
        HWInventoryByLocation::HWInvByLocComputeModule(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocDrive(v) => {
        HWInventoryByLocation::HWInvByLocDrive(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocHSNBoard(v) => {
        HWInventoryByLocation::HWInvByLocHSNBoard(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocHSNNIC(v) => {
        HWInventoryByLocation::HWInvByLocHSNNIC(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocMemory(v) => {
        HWInventoryByLocation::HWInvByLocMemory(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocMgmtHLSwitch(v) => {
        HWInventoryByLocation::HWInvByLocMgmtHLSwitch(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocMgmtSwitch(v) => {
        HWInventoryByLocation::HWInvByLocMgmtSwitch(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocNode(v) => {
        HWInventoryByLocation::HWInvByLocNode(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocNodeAccel(v) => {
        HWInventoryByLocation::HWInvByLocNodeAccel(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocNodeAccelRiser(v) => {
        HWInventoryByLocation::HWInvByLocNodeAccelRiser(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocNodeBMC(v) => {
        HWInventoryByLocation::HWInvByLocNodeBMC(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocNodeEnclosure(v) => {
        HWInventoryByLocation::HWInvByLocNodeEnclosure(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocNodePowerSupply(v) => {
        HWInventoryByLocation::HWInvByLocNodePowerSupply(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocOutlet(v) => {
        HWInventoryByLocation::HWInvByLocOutlet(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocPDU(v) => {
        HWInventoryByLocation::HWInvByLocPDU(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocProcessor(v) => {
        HWInventoryByLocation::HWInvByLocProcessor(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocRouterBMC(v) => {
        HWInventoryByLocation::HWInvByLocRouterBMC(v.into())
      }
      FrontEndHWInventoryByLocation::HWInvByLocRouterModule(v) => {
        HWInventoryByLocation::HWInvByLocRouterModule(v.into())
      }
    }
  }
}

impl From<HWInventoryByLocation> for FrontEndHWInventoryByLocation {
  fn from(val: HWInventoryByLocation) -> Self {
    match val {
      HWInventoryByLocation::HWInvByLocCDUMgmtSwitch(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocCDUMgmtSwitch(v.into())
      }
      HWInventoryByLocation::HWInvByLocCMMRectifier(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocCMMRectifier(v.into())
      }
      HWInventoryByLocation::HWInvByLocCabinet(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocCabinet((*v).into())
      }
      HWInventoryByLocation::HWInvByLocChassis(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocChassis((*v).into())
      }
      HWInventoryByLocation::HWInvByLocComputeModule(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocComputeModule(v.into())
      }
      HWInventoryByLocation::HWInvByLocDrive(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocDrive(v.into())
      }
      HWInventoryByLocation::HWInvByLocHSNBoard(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocHSNBoard(v.into())
      }
      HWInventoryByLocation::HWInvByLocHSNNIC(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocHSNNIC(v.into())
      }
      HWInventoryByLocation::HWInvByLocMemory(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocMemory(v.into())
      }
      HWInventoryByLocation::HWInvByLocMgmtHLSwitch(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocMgmtHLSwitch(v.into())
      }
      HWInventoryByLocation::HWInvByLocMgmtSwitch(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocMgmtSwitch(v.into())
      }
      HWInventoryByLocation::HWInvByLocNode(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocNode(v.into())
      }
      HWInventoryByLocation::HWInvByLocNodeAccel(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocNodeAccel(v.into())
      }
      HWInventoryByLocation::HWInvByLocNodeAccelRiser(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocNodeAccelRiser(v.into())
      }
      HWInventoryByLocation::HWInvByLocNodeBMC(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocNodeBMC(v.into())
      }
      HWInventoryByLocation::HWInvByLocNodeEnclosure(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocNodeEnclosure(v.into())
      }
      HWInventoryByLocation::HWInvByLocNodePowerSupply(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocNodePowerSupply(v.into())
      }
      HWInventoryByLocation::HWInvByLocOutlet(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocOutlet(v.into())
      }
      HWInventoryByLocation::HWInvByLocPDU(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocPDU(v.into())
      }
      HWInventoryByLocation::HWInvByLocProcessor(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocProcessor(v.into())
      }
      HWInventoryByLocation::HWInvByLocRouterBMC(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocRouterBMC(v.into())
      }
      HWInventoryByLocation::HWInvByLocRouterModule(v) => {
        FrontEndHWInventoryByLocation::HWInvByLocRouterModule(v.into())
      }
    }
  }
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
