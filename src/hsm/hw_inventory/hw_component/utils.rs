//! Helpers built on top of `ShastaClient::hsm_hw_component_*` methods.

use serde_json::Value;

/// Extract processor model names from a node's HSM HW Inventory Value
/// (path `/Nodes/0/Processors[*]/PopulatedFRU/ProcessorFRUInfo/Model`).
pub fn get_list_processor_model_from_hw_inventory_value(
  hw_inventory: &Value,
) -> Option<Vec<String>> {
  hw_inventory
    .pointer("/Nodes/0/Processors")
    .and_then(Value::as_array)
    .map(|processor_list: &Vec<Value>| {
      processor_list
        .iter()
        .filter_map(|processor| {
          processor
            .pointer("/PopulatedFRU/ProcessorFRUInfo/Model")
            .and_then(Value::as_str)
            .map(str::to_string)
        })
        .collect::<Vec<String>>()
    })
}

/// Extract accelerator (GPU) model names from a node's HSM HW
/// Inventory Value (path
/// `/Nodes/0/NodeAccels[*]/PopulatedFRU/NodeAccelFRUInfo/Model`).
pub fn get_list_accelerator_model_from_hw_inventory_value(
  hw_inventory: &Value,
) -> Option<Vec<String>> {
  hw_inventory
    .pointer("/Nodes/0/NodeAccels")
    .and_then(Value::as_array)
    .map(|accelerator_list| {
      accelerator_list
        .iter()
        .filter_map(|accelerator| {
          accelerator
            .pointer("/PopulatedFRU/NodeAccelFRUInfo/Model")
            .and_then(Value::as_str)
            .map(str::to_string)
        })
        .collect::<Vec<String>>()
    })
}

/// Extract HSN NIC descriptions from a node's HSM HW Inventory Value
/// (path `/Nodes/0/NodeHsnNics[*]/NodeHsnNicLocationInfo/Description`).
pub fn get_list_hsn_nics_model_from_hw_inventory_value(
  hw_inventory: &Value,
) -> Option<Vec<String>> {
  hw_inventory
    .pointer("/Nodes/0/NodeHsnNics")
    .and_then(Value::as_array)
    .map(|hsn_nic_list| {
      hsn_nic_list
        .iter()
        .filter_map(|hsn_nic| {
          hsn_nic
            .pointer("/NodeHsnNicLocationInfo/Description")
            .and_then(Value::as_str)
            .map(str::to_string)
        })
        .collect::<Vec<String>>()
    })
}

/// Extract per-DIMM memory capacities (MiB) from a node's HSM HW
/// Inventory Value (path
/// `/Nodes/0/Memory[*]/PopulatedFRU/MemoryFRUInfo/CapacityMiB`).
pub fn get_list_memory_capacity_from_hw_inventory_value(
  hw_inventory: &Value,
) -> Option<Vec<u64>> {
  hw_inventory
    .pointer("/Nodes/0/Memory")
    .and_then(Value::as_array)
    .map(|memory_list| {
      memory_list
        .iter()
        .map(|memory| {
          memory
            .pointer("/PopulatedFRU/MemoryFRUInfo/CapacityMiB")
            .and_then(Value::as_u64)
            .unwrap_or(0)
        })
        .collect::<Vec<u64>>()
    })
}
