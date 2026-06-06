//! Liveness/readiness probes against the BOS service.

use serde_json::Value;

use crate::{ShastaClient, common::http, error::Error};

impl ShastaClient {
  /// `GET /bos/v2/healthz` — BOS liveness/readiness probe.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bos_health_check(&self, token: &str) -> Result<Value, Error> {
    let api_url = format!("{}/bos/v2/healthz", self.base_url());
    http::get_json(self.http(), &api_url, token).await
  }
}

/// Convenience: build a transient `ShastaClient` and run a BOS health
/// check in one call.
///
/// # Errors
///
/// Returns an [`Error`] variant on CSM, transport, or
/// deserialization failure; see the crate-level `Error` enum
/// for the full set.
pub async fn get(
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
  .bos_health_check(shasta_token)
  .await
}
