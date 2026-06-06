//! Helpers built on top of the CAPMC `ShastaClient` methods.

use std::time::Duration;

use crate::{
  ShastaClient,
  capmc::types::XnameStatusResponse,
  common::poll::{PollBackoff, poll_until_with_backoff},
  error::Error,
};

const POWER_TRANSITION_BACKOFF: PollBackoff = PollBackoff {
  initial_delay: Duration::from_secs(3),
  max_delay: Duration::from_secs(10),
  max_attempts: 40,
};

/// Issue repeated CAPMC power-on requests, polling status with
/// exponential backoff until every xname reports as "on" or the
/// attempt cap is reached.
///
/// # Errors
///
/// Returns an [`Error`] variant on CSM, transport, or
/// deserialization failure; see the crate-level `Error` enum
/// for the full set.
pub async fn wait_nodes_to_power_on(
  client: &ShastaClient,
  token: &str,
  xname_vec: Vec<String>,
  reason: Option<String>,
) -> Result<XnameStatusResponse, Error> {
  poll_until_with_backoff(
    POWER_TRANSITION_BACKOFF,
    || async {
      if let Err(e) = client
        .capmc_node_power_on_post(token, xname_vec.clone(), reason.clone())
        .await
      {
        log::warn!(
          "CAPMC power-on returned an error (continuing to poll status): {e}"
        );
      }
      client
        .capmc_node_power_status_post(token, &xname_vec)
        .await
    },
    |status| status.off.as_ref().is_none_or(|off| off.is_empty()),
  )
  .await
}

/// Issue repeated CAPMC power-off requests (graceful unless `force`),
/// polling status with exponential backoff until every xname reports
/// "off" or the attempt cap is reached.
///
/// # Errors
///
/// Returns an [`Error`] variant on CSM, transport, or
/// deserialization failure; see the crate-level `Error` enum
/// for the full set.
pub async fn wait_nodes_to_power_off(
  client: &ShastaClient,
  token: &str,
  xname_vec: Vec<String>,
  reason_opt: Option<String>,
  force: bool,
) -> Result<XnameStatusResponse, Error> {
  poll_until_with_backoff(
    POWER_TRANSITION_BACKOFF,
    || async {
      let _ = client
        .capmc_node_power_off_post(
          token,
          xname_vec.clone(),
          reason_opt.clone(),
          force,
        )
        .await?;
      client
        .capmc_node_power_status_post(token, &xname_vec)
        .await
    },
    |status| {
      let off = status.off.clone().unwrap_or_default();
      xname_vec.iter().all(|xname| off.contains(xname))
    },
  )
  .await
}
