//! Helpers built on top of the CAPMC `ShastaClient` methods.

use core::time;

use crate::{
  ShastaClient, capmc::types::XnameStatusResponse, error::Error,
};

/// Issue repeated CAPMC power-on requests, polling status every 3
/// seconds until every xname reports as "on" or the 60-attempt cap is
/// reached.
pub async fn wait_nodes_to_power_on(
  client: &ShastaClient,
  token: &str,
  xname_vec: Vec<String>,
  reason: Option<String>,
) -> Result<XnameStatusResponse, Error> {
  let mut status = client
    .capmc_node_power_status_post(token, &xname_vec)
    .await?;
  let mut node_off_vec: Vec<String> =
    status.off.clone().unwrap_or_default();

  let mut i = 0;
  let max = 60;
  let delay_secs = 3;
  while i <= max && !node_off_vec.is_empty() {
    let _ = client
      .capmc_node_power_on_post(token, xname_vec.clone(), reason.clone())
      .await;

    tokio::time::sleep(time::Duration::from_secs(delay_secs)).await;

    status = client
      .capmc_node_power_status_post(token, &xname_vec)
      .await?;
    node_off_vec = status.off.clone().unwrap_or_default();

    log::info!(
      "Waiting nodes to power on. Trying again in {} seconds. Attempt {} of {}.",
      delay_secs,
      i + 1,
      max
    );

    i += 1;
  }

  Ok(status)
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
) -> Result<XnameStatusResponse, Error> {
  let mut node_off_vec: Vec<String> = Vec::new();
  let mut status = XnameStatusResponse::default();

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

    status = client
      .capmc_node_power_status_post(token, &xname_vec)
      .await?;
    node_off_vec = status.off.clone().unwrap_or_default();

    log::info!(
      "Waiting nodes to power off. Trying again in {} seconds. Attempt {} of {}.",
      delay_secs,
      i + 1,
      max
    );

    i += 1;
  }

  Ok(status)
}
