//! `ShastaClient` methods for CAPMC power transitions and status.

use crate::{
  ShastaClient,
  capmc::{
    types::{
      NodeStatus, PowerStatus, XnamePowerActionResponse, XnameStatusResponse,
    },
    utils::{wait_nodes_to_power_off, wait_nodes_to_power_on},
  },
  common::http,
  error::Error,
};

impl ShastaClient {
  /// Issue a CAPMC power-off request for the given xnames and return
  /// immediately with the parsed response (fire-and-forget).
  ///
  /// `POST /capmc/capmc/v1/xname_off`. Use
  /// [`Self::capmc_node_power_off_post_sync`] to also wait until the
  /// nodes report as `off`.
  pub async fn capmc_node_power_off_post(
    &self,
    token: &str,
    xname_vec: Vec<String>,
    reason_opt: Option<String>,
    force: bool,
  ) -> Result<XnamePowerActionResponse, Error> {
    log::debug!("Power OFF nodes: {:?}", xname_vec);

    let power_off = PowerStatus::new(reason_opt, xname_vec, force, None);
    let api_url = format!("{}/capmc/capmc/v1/xname_off", self.base_url());
    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&power_off)
      .send()
      .await
      .map_err(Error::NetError)?;
    http::handle_json_response(response).await
  }

  /// Power off the given xnames and wait until CAPMC reports each one
  /// as `off`.
  ///
  /// This wrapper around [`Self::capmc_node_power_off_post`] polls
  /// `xname_status` until every target is down before returning.
  pub async fn capmc_node_power_off_post_sync(
    &self,
    token: &str,
    xname_vec: Vec<String>,
    reason_opt: Option<String>,
    force: bool,
  ) -> Result<XnameStatusResponse, Error> {
    // Check Nodes are shutdown
    let _ = self.capmc_node_power_status_post(token, &xname_vec).await?;

    wait_nodes_to_power_off(self, token, xname_vec, reason_opt, force).await
  }

  /// Issue a CAPMC power-on request for the given xnames and return
  /// immediately (fire-and-forget).
  ///
  /// `POST /capmc/capmc/v1/xname_on`.
  pub async fn capmc_node_power_on_post(
    &self,
    token: &str,
    xname_vec: Vec<String>,
    reason: Option<String>,
  ) -> Result<XnamePowerActionResponse, Error> {
    let power_on = PowerStatus::new(reason, xname_vec, false, None);
    let api_url = format!("{}/capmc/capmc/v1/xname_on", self.base_url());
    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&power_on)
      .send()
      .await
      .map_err(Error::NetError)?;
    http::handle_json_response(response).await
  }

  /// Power on the given xnames and wait until CAPMC reports each one
  /// as `on`.
  pub async fn capmc_node_power_on_post_sync(
    &self,
    token: &str,
    xname_vec: Vec<String>,
    reason: Option<String>,
  ) -> Result<XnameStatusResponse, Error> {
    let _ = self.capmc_node_power_status_post(token, &xname_vec).await?;

    wait_nodes_to_power_on(self, token, xname_vec, reason).await
  }

  /// Issue a CAPMC reinit (power-cycle) request and return immediately.
  ///
  /// `POST /capmc/capmc/v1/xname_reinit`.
  pub async fn capmc_node_power_reset_post(
    &self,
    token: &str,
    xname_vec: Vec<String>,
    reason: Option<String>,
    force: bool,
  ) -> Result<XnamePowerActionResponse, Error> {
    let node_restart = PowerStatus::new(reason, xname_vec, force, None);
    let api_url = format!("{}/capmc/capmc/v1/xname_reinit", self.base_url());
    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&node_restart)
      .send()
      .await
      .map_err(Error::NetError)?;
    http::handle_json_response(response).await
  }

  /// Power-cycle the given xnames by powering off then back on
  /// synchronously, waiting for each transition to complete.
  ///
  /// Implemented as `power_off_post_sync` followed by
  /// `power_on_post_sync` for the same set of xnames.
  pub async fn capmc_node_power_reset_post_sync(
    &self,
    token: &str,
    xname_vec: Vec<String>,
    reason_opt: Option<String>,
    force: bool,
  ) -> Result<XnameStatusResponse, Error> {
    log::debug!("Power RESET node: {:?}", xname_vec);

    let _ = self
      .capmc_node_power_off_post_sync(
        token,
        xname_vec.clone(),
        reason_opt.clone(),
        force,
      )
      .await?;

    self
      .capmc_node_power_on_post_sync(token, xname_vec, reason_opt)
      .await
  }

  /// Power-cycle a set of xnames concurrently — one
  /// `power_reset_post_sync` is spawned per xname so each node is reset
  /// in parallel.
  ///
  /// Returns once every node is back on. Use this rather than
  /// [`Self::capmc_node_power_reset_post_sync`] when the target list is
  /// large and waiting for them serially would be too slow.
  pub async fn capmc_node_power_reset_post_sync_vec(
    &self,
    token: &str,
    xnames: Vec<String>,
    reason_opt: Option<String>,
    force: bool,
  ) -> Result<Vec<XnameStatusResponse>, Error> {
    let mut nodes_reseted = Vec::new();
    let mut tasks = tokio::task::JoinSet::new();

    for xname in xnames {
      let client = self.clone();
      let reason_cloned = reason_opt.clone();
      let token = token.to_string();

      tasks.spawn(async move {
        client
          .capmc_node_power_reset_post_sync(
            &token,
            vec![xname],
            reason_cloned,
            force,
          )
          .await
      });
    }

    while let Some(message) = tasks.join_next().await {
      nodes_reseted.push(message??);
    }

    Ok(nodes_reseted)
  }

  /// Query CAPMC for the current Redfish power state of the given
  /// xnames.
  ///
  /// `POST /capmc/capmc/v1/get_xname_status` with source `redfish`.
  pub async fn capmc_node_power_status_post(
    &self,
    token: &str,
    xnames: &Vec<String>,
  ) -> Result<XnameStatusResponse, Error> {
    log::debug!("Checking nodes status: {:?}", xnames);

    let node_status_payload =
      NodeStatus::new(None, Some(xnames.clone()), Some("redfish".to_string()));
    let url_api =
      format!("{}/capmc/capmc/v1/get_xname_status", self.base_url());
    let response = self
      .http()
      .post(url_api)
      .bearer_auth(token)
      .json(&node_status_payload)
      .send()
      .await
      .map_err(Error::NetError)?;
    http::handle_json_response(response).await
  }
}
