use manta_backend_dispatcher::{
  error::Error,
  interfaces::pcs::PCSTrait,
  types::pcs::{
    power_status::types::PowerStatusAll as FrontEndPowerStatusAll,
    transitions::types::TransitionResponse,
  },
};

use crate::pcs;

use super::Csm;

impl PCSTrait for Csm {
  async fn power_on_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
  ) -> Result<TransitionResponse, Error> {
    let operation = "on";

    pcs::transitions::http_client::post_block(
      &self.base_url,
      auth_token,
      &self.root_cert,
      operation,
      &nodes.to_vec(),
    )
    .await
    .map(|transition| transition.into())
    .map_err(|e: crate::error::Error| Error::Message(e.to_string()))
  }

  async fn power_off_sync(
    &self,
    auth_token: &str,
    nodes: &[String],
    force: bool,
  ) -> Result<TransitionResponse, Error> {
    let operation = if force { "force-off" } else { "soft-off" };

    pcs::transitions::http_client::post_block(
      &self.base_url,
      auth_token,
      &self.root_cert,
      operation,
      &nodes.to_vec(),
    )
    .await
    .map(|transition| transition.into())
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

    pcs::transitions::http_client::post_block(
      &self.base_url,
      auth_token,
      &self.root_cert,
      operation,
      &nodes.to_vec(),
    )
    .await
    .map(|transition| transition.into())
    .map_err(|e: crate::error::Error| Error::Message(e.to_string()))
  }

  async fn power_status(
    &self,
    auth_token: &str,
    nodes: &[String],
    power_state_filter: Option<&str>,
    management_state_filter: Option<&str>,
  ) -> Result<FrontEndPowerStatusAll, Error> {
    // Convert &[String] to Vec<&str> and wrap in Some
    let nodes_str: Vec<&str> = nodes.iter().map(|s| s.as_str()).collect();
    let nodes_opt = Some(nodes_str.as_slice());

    pcs::power_status::http_client::post(
      &self.base_url,
      auth_token,
      &self.root_cert,
      nodes_opt,
      power_state_filter,
      management_state_filter,
    )
    .await
    .map(|status| {
      println!("return value from async fn power_status : {:?}", status);
      status.into()
    })
    .map_err(|e| Error::Message(e.to_string()))
  }
}
