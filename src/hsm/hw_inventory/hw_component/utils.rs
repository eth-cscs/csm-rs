use serde_json::Value;

pub fn get_list_processor_model_from_hw_inventory_value(
  hw_inventory: &Value,
) -> Option<Vec<String>> {
  hw_inventory
    .pointer("/Nodes/0/Processors")
    /* .get("Nodes")
    .and_then(Value::as_array)
    .and_then(|nodes| nodes.first())
    .and_then(|first_node| first_node.get("Processors")) */
    .and_then(Value::as_array)
    .map(|processor_list: &Vec<Value>| {
      processor_list
        .iter()
        .map(|processor| {
          processor
            .pointer("/PopulatedFRU/ProcessorFRUInfo/Model")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap()
        })
        .collect::<Vec<String>>()
    })
}

pub fn get_list_accelerator_model_from_hw_inventory_value(
  hw_inventory: &Value,
) -> Option<Vec<String>> {
  hw_inventory
    .pointer("/Nodes/0/NodeAccels")
    /* .get("Nodes")
    .and_then(Value::as_array)
    .and_then(|nodes| nodes.first())
    .and_then(|first_node| first_node.get("NodeAccels")) */
    .and_then(Value::as_array)
    .map(|accelerator_list| {
      accelerator_list
        .iter()
        .map(|accelerator| {
          accelerator
            .pointer("/PopulatedFRU/NodeAccelFRUInfo/Model")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap()
        })
        .collect::<Vec<String>>()
    })
}

pub fn get_list_hsn_nics_model_from_hw_inventory_value(
  hw_inventory: &Value,
) -> Option<Vec<String>> {
  hw_inventory
    .pointer("/Nodes/0/NodeHsnNics")
    /* .get("Nodes")
    .and_then(Value::as_array)
    .and_then(|nodes| nodes.first())
    .and_then(|first_node| first_node.get("NodeHsnNics")) */
    .and_then(Value::as_array)
    .map(|hsn_nic_list| {
      hsn_nic_list
        .iter()
        .map(|hsn_nic| {
          hsn_nic
            .pointer("/NodeHsnNicLocationInfo/Description")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap()
        })
        .collect::<Vec<String>>()
    })
}

pub fn get_list_memory_capacity_from_hw_inventory_value(
  hw_inventory: &Value,
) -> Option<Vec<u64>> {
  hw_inventory
    .pointer("/Nodes/0/Memory")
    /* .get("Nodes")
    .and_then(Value::as_array)
    .and_then(|nodes| nodes.first())
    .and_then(|first_node| first_node.get("Memory")) */
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
