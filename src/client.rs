//! [`ShastaClient`] — the single entry point for talking to a Shasta CSM API.
//!
//! Bundles the four parameters every HTTP call used to take individually
//! (`shasta_token`, `shasta_base_url`, `shasta_root_cert`, `socks5_proxy`)
//! into one struct, and holds a pre-built `reqwest::Client` so we don't
//! re-construct it per request.
//!
//! All API calls hang off this type as methods. There are no free
//! `pub async fn x(token, base_url, ...)` functions anymore — those were
//! removed in 0.107.0. Callers should construct one `ShastaClient` per
//! Shasta installation and reuse it across calls.

use crate::common::http;
use crate::error::Error;

/// Connection details + a reusable `reqwest::Client` for one Shasta CSM
/// installation.
#[derive(Debug, Clone)]
pub struct ShastaClient {
  pub(crate) base_url: String,
  pub(crate) token: String,
  pub(crate) root_cert: Vec<u8>,
  pub(crate) socks5_proxy: Option<String>,
  pub(crate) http: reqwest::Client,
}

impl ShastaClient {
  /// Build a new client. Constructs the underlying `reqwest::Client` once,
  /// applying the CSM root cert and (optionally) a SOCKS5 proxy.
  pub fn new(
    base_url: impl Into<String>,
    token: impl Into<String>,
    root_cert: impl Into<Vec<u8>>,
    socks5_proxy: Option<String>,
  ) -> Result<Self, Error> {
    let root_cert = root_cert.into();
    let http = http::build_client(&root_cert, socks5_proxy.as_deref())?;
    Ok(Self {
      base_url: base_url.into(),
      token: token.into(),
      root_cert,
      socks5_proxy,
      http,
    })
  }

  pub fn base_url(&self) -> &str {
    &self.base_url
  }

  pub fn token(&self) -> &str {
    &self.token
  }

  pub fn root_cert(&self) -> &[u8] {
    &self.root_cert
  }

  pub fn socks5_proxy(&self) -> Option<&str> {
    self.socks5_proxy.as_deref()
  }

  pub(crate) fn http(&self) -> &reqwest::Client {
    &self.http
  }
}
