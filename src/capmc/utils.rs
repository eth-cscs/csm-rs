//! Helpers built on top of the CAPMC `ShastaClient` methods.

use core::time;
use serde_json::Value;
use std::io::Write;

use crate::{ShastaClient, error::Error};

/// Issue repeated CAPMC power-on requests, polling status every 3
/// seconds until every xname reports as "on" or the 60-attempt cap is
/// reached.
pub async fn wait_nodes_to_power_on(
  client: &ShastaClient,
  token: &str,
  xname_vec: Vec<String>,
  reason: Option<String>,
) -> Result<Value, Error> {
  let mut node_status_value: Value = client
    .capmc_node_power_status_post(token, &xname_vec)
    .await?;

  let mut node_off_vec: Vec<String> = node_status_value
    .get("off")
    .and_then(Value::as_array)
    .map(|node_status_off| {
      node_status_off
        .iter()
        .filter_map(|xname: &Value| xname.as_str().map(str::to_string))
        .collect()
    })
    .unwrap_or_default();

  // Check all nodes are OFF
  let mut i = 0;
  let max = 60;
  let delay_secs = 3;
  while i <= max && !node_off_vec.is_empty() {
    let _ = client
      .capmc_node_power_on_post(token, xname_vec.clone(), reason.clone())
      .await;

    tokio::time::sleep(time::Duration::from_secs(delay_secs)).await;

    node_status_value = client
      .capmc_node_power_status_post(token, &xname_vec)
      .await?;

    node_off_vec = node_status_value
      .get("off")
      .and_then(Value::as_array)
      .map(|node_status_off| {
        node_status_off
          .iter()
          .filter_map(|xname: &Value| xname.as_str().map(str::to_string))
          .collect()
      })
      .unwrap_or_default();

    log::info!(
      "\rWaiting nodes to power on. Trying again in {} seconds. Attempt {} of {}.",
      delay_secs,
      i + 1,
      max
    );
    std::io::stdout().flush().unwrap();

    i += 1;
  }

  Ok(node_status_value)
}

/// Issue repeated CAPMC power-off requests (graceful unless `force`),
/// polling status until every xname reports "off" or the 60-attempt
/// cap is reached.
pub async fn wait_nodes_to_power_off(
  client: &ShastaClient,
  token: &str,
  xname_vec: Vec<String>,
  reason_opt: Option<String>,
  force: bool,
) -> Result<Value, Error> {
  let mut node_off_vec: Vec<String> = Vec::new();
  let mut node_status_value: Value = serde_json::Value::Null;

  // Check all nodes are OFF
  let mut i = 0;
  let max = 60;
  let delay_secs = 3;
  while i <= max && xname_vec.iter().any(|xname| !node_off_vec.contains(xname))
  {
    let _ = client
      .capmc_node_power_off_post(
        token,
        xname_vec.clone(),
        reason_opt.clone(),
        force,
      )
      .await?;

    tokio::time::sleep(time::Duration::from_secs(delay_secs)).await;

    node_status_value = client
      .capmc_node_power_status_post(token, &xname_vec)
      .await?;

    node_off_vec = node_status_value
      .get("off")
      .and_then(Value::as_array)
      .map(|node_status_off| {
        node_status_off
          .iter()
          .map(|xname: &Value| xname.as_str().map(str::to_string).unwrap())
          .collect()
      })
      .unwrap_or_default();

    log::info!(
      "\rWaiting nodes to power off. Trying again in {} seconds. Attempt {} of {}.",
      delay_secs,
      i + 1,
      max
    );
    std::io::stdout().flush().unwrap();

    i += 1;
  }

  Ok(node_status_value)
}
