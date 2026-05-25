//! Liveness/readiness probes against the BOS service.

use serde_json::Value;

use crate::{ShastaClient, common::http, error::Error};

impl ShastaClient {
  pub async fn bos_health_check(&self) -> Result<Value, Error> {
    let api_url = format!("{}/bos/v2/healthz", self.base_url());
    http::get_json(self.http(), &api_url, self.token()).await
  }
}

pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Value, Error> {
  crate::ShastaClient::new(
    shasta_base_url,
    shasta_token,
    shasta_root_cert.to_vec(),
    socks5_proxy.map(str::to_owned),
  )?
  .bos_health_check()
  .await
}
