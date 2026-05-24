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
///
/// `ShastaClient` is the single entry point for every HTTP call csm-rs
/// makes. Construct it once per Shasta installation and reuse it; cloning
/// is cheap (the inner `reqwest::Client` is reference-counted).
///
/// # Method naming
///
/// Methods follow the convention `<module>_<resource>_<verb>`, optionally
/// suffixed with an API version. Examples:
///
/// - `ims_image_get_all` — GET /ims/v3/images
/// - `cfs_configuration_v2_put` — PUT /cfs/v2/configurations/{name}
/// - `hsm_group_delete_member` — DELETE /smd/hsm/v2/groups/{label}/members/{id}
/// - `pcs_transitions_post_block` — POST /power-control/v1/transitions and poll until completion
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), csm_rs::error::Error> {
/// let client = csm_rs::ShastaClient::new(
///     "https://api.shasta.example.com",
///     "your-bearer-token",
///     std::fs::read("/etc/shasta/ca.crt").unwrap(),
///     None,
/// )?;
///
/// // Single shared client across many calls:
/// let images = client.ims_image_get_all().await?;
/// let group = client.hsm_group_get_one("zinal").await?;
/// # Ok(())
/// # }
/// ```
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
  ///
  /// # Errors
  ///
  /// Returns [`Error::NetError`] if the proxy URL is malformed or
  /// `reqwest::Client::build` fails. Malformed PEM input is silently
  /// tolerated by `reqwest::Certificate::from_pem` (it just produces an
  /// empty trust chain), so this constructor does not surface PEM errors.
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

  /// The Shasta API base URL (e.g. `https://api.shasta.example.com`).
  pub fn base_url(&self) -> &str {
    &self.base_url
  }

  /// The bearer token used to authenticate every request.
  pub fn token(&self) -> &str {
    &self.token
  }

  /// The PEM-encoded root certificate trusted for HTTPS calls.
  pub fn root_cert(&self) -> &[u8] {
    &self.root_cert
  }

  /// The SOCKS5 proxy URL, if one was configured.
  pub fn socks5_proxy(&self) -> Option<&str> {
    self.socks5_proxy.as_deref()
  }

  pub(crate) fn http(&self) -> &reqwest::Client {
    &self.http
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Reuse the test PEM from common::http tests to avoid duplicating a cert
  // blob; both modules need a syntactically valid PEM to construct a client.
  const TEST_PEM: &str = "-----BEGIN CERTIFICATE-----\n\
MIIBhTCCASugAwIBAgIQIRi6zePL6mKjOipn+dNuaTAKBggqhkjOPQQDAjASMRAw\n\
DgYDVQQKEwdBY21lIENvMB4XDTE3MTAyMDE5NDMwNloXDTE4MTAyMDE5NDMwNlow\n\
EjEQMA4GA1UEChMHQWNtZSBDbzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABD0d\n\
7VNhbWvZLWPuj/RtHFjvtJBEwOkhbN/BnnE8rnZR8+sbwnc/KhCk3FhnpHZnQz7B\n\
5aETbbIgmuvewdjvSBSjYzBhMA4GA1UdDwEB/wQEAwICpDATBgNVHSUEDDAKBggr\n\
BgEFBQcDATAPBgNVHRMBAf8EBTADAQH/MCkGA1UdEQQiMCCCDmxvY2FsaG9zdDo1\n\
NDUzgg4xMjcuMC4wLjE6NTQ1MzAKBggqhkjOPQQDAgNIADBFAiEA2zpJEPQyz6/l\n\
Wf86aX6PepsntZv2GYlA5UpabfT2EZICICpJ5h/iI+i341gBmLiAFQOyTDT+/wQc\n\
6MF9+Yw1Yy0t\n\
-----END CERTIFICATE-----\n";

  #[test]
  fn new_with_valid_pem_and_no_proxy_succeeds() {
    let client = ShastaClient::new(
      "https://api.shasta.example.com",
      "tok",
      TEST_PEM.as_bytes().to_vec(),
      None,
    )
    .expect("client construction should succeed");

    assert_eq!(client.base_url(), "https://api.shasta.example.com");
    assert_eq!(client.token(), "tok");
    assert_eq!(client.root_cert(), TEST_PEM.as_bytes());
    assert!(client.socks5_proxy().is_none());
  }

  #[test]
  fn new_with_socks5_proxy_succeeds() {
    let client = ShastaClient::new(
      "https://api.example.com",
      "tok",
      TEST_PEM.as_bytes().to_vec(),
      Some("socks5://localhost:9050".to_string()),
    )
    .expect("client with proxy should succeed");

    assert_eq!(client.socks5_proxy(), Some("socks5://localhost:9050"));
  }

  #[test]
  fn new_with_invalid_proxy_url_fails() {
    let result = ShastaClient::new(
      "https://api.example.com",
      "tok",
      TEST_PEM.as_bytes().to_vec(),
      Some(":::not a url:::".to_string()),
    );
    assert!(result.is_err());
  }

  // NOTE: there is no test for "invalid PEM fails" because
  // `reqwest::Certificate::from_pem` is lenient — see the analogous comment
  // in `common::http::tests`. Garbage input returns Ok with an empty chain.

  #[test]
  fn clone_preserves_all_fields() {
    let client = ShastaClient::new(
      "https://api.example.com",
      "tok",
      TEST_PEM.as_bytes().to_vec(),
      None,
    )
    .unwrap();
    let cloned = client.clone();

    assert_eq!(client.base_url(), cloned.base_url());
    assert_eq!(client.token(), cloned.token());
    assert_eq!(client.root_cert(), cloned.root_cert());
    assert_eq!(client.socks5_proxy(), cloned.socks5_proxy());
  }

  #[test]
  fn accepts_owned_and_borrowed_strings_via_into() {
    // String
    let _ = ShastaClient::new(
      "https://api.example.com".to_string(),
      "tok".to_string(),
      TEST_PEM.as_bytes().to_vec(),
      None,
    )
    .unwrap();
    // &str
    let _ = ShastaClient::new(
      "https://api.example.com",
      "tok",
      TEST_PEM.as_bytes(),
      None,
    )
    .unwrap();
  }
}
