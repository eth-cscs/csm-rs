//! `PCSTrait` impl for [`Csm`](super::Csm).

use manta_backend_dispatcher::{
  error::Error,
  interfaces::pcs::PCSTrait,
  types::pcs::{
    power_status::types::PowerStatusAll as FrontEndPowerStatusAll,
    transitions::types::TransitionResponse,
  },
};

use super::Csm;

impl PCSTrait for Csm {
  async fn power_on_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
  ) -> Result<TransitionResponse, Error> {
    self
      .shasta_client()
      .pcs_transitions_post_block(auth_token, "on", nodes)
      .await
      .map(Into::into)
      .map_err(|e: crate::error::Error| Error::Message(e.to_string()))
  }

  async fn power_off_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
    force: bool,
  ) -> Result<TransitionResponse, Error> {
    let operation = if force { "force-off" } else { "soft-off" };

    self
      .shasta_client()
      .pcs_transitions_post_block(auth_token, operation, nodes)
      .await
      .map(Into::into)
      .map_err(|e: crate::error::Error| Error::Message(e.to_string()))
  }

  async fn power_reset_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
    force: bool,
  ) -> Result<TransitionResponse, Error> {
    let operation = if force {
      "hard-restart"
    } else {
      "soft-restart"
    };

    self
      .shasta_client()
      .pcs_transitions_post_block(auth_token, operation, nodes)
      .await
      .map(Into::into)
      .map_err(|e: crate::error::Error| Error::Message(e.to_string()))
  }

  async fn power_status(
    &self,
    auth_token: &str,
    nodes: &[String],
    power_state_filter: Option<&str>,
    management_state_filter: Option<&str>,
  ) -> Result<FrontEndPowerStatusAll, Error> {
    let nodes_str: Vec<&str> = nodes.iter().map(String::as_str).collect();

    self
      .shasta_client()
      .pcs_power_status_post(
        auth_token,
        Some(nodes_str.as_slice()),
        power_state_filter,
        management_state_filter,
      )
      .await
      .map(|status| {
        log::info!("return value from async fn power_status : {:?}", status);
        status.into()
      })
      .map_err(|e| Error::Message(e.to_string()))
  }
}
