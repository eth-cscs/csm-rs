//! Helpers shared across the CFS resource modules (e.g. service health).

use serde_json::Value;

use crate::{ShastaClient, common::http, error::Error};

impl ShastaClient {
  /// Probe CFS for liveness.
  ///
  /// `GET /cfs/healthz`. Returns the raw JSON body produced by CFS;
  /// success only means the service was reachable and authenticated.
  pub async fn cfs_health_check(&self, token: &str) -> Result<Value, Error> {
    let api_url = format!("{}/cfs/healthz", self.base_url());
    http::get_json(self.http(), &api_url, token).await
  }
}

pub async fn health_check(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Value, Error> {
  crate::ShastaClient::new(
    shasta_base_url,
    shasta_root_cert.to_vec(),
    socks5_proxy.map(str::to_owned),
  )?
  .cfs_health_check(shasta_token)
  .await
}
