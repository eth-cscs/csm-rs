//! Thin wrapper bridging the generated HSM client to the public
//! `ShastaClient` API. Responsibilities:
//!  - construct a per-call generated `Client` with bearer auth baked in;
//!  - override the spec's basePath with the URL shape csm-rs has always used;
//!  - map `progenitor_client::Error<T>` into `crate::error::Error`.
//!
//! Per-resource wrapper files (`group.rs`, `memberships.rs`, …) hold
//! `impl ShastaClient { pub async fn hsm_*() }` blocks that delegate
//! to the generated client.

use crate::{ShastaClient, error::Error, hsm::generated};

/// Build a generated HSM `Client` bound to the caller's token. Cheap to
/// recreate per call: `reqwest::Client` clones are `Arc`-internal, and
/// the generated `Client` is a thin newtype around it.
pub(crate) fn gen_client(client: &ShastaClient, token: &str) -> generated::Client {
  // Inject the bearer token as a default header on a fresh reqwest::Client
  // built from the configured one. This avoids depending on progenitor's
  // (version-sensitive) middleware story.
  let mut headers = reqwest::header::HeaderMap::new();
  let auth = format!("Bearer {token}");
  let mut value = reqwest::header::HeaderValue::from_str(&auth)
    .expect("bearer token contained header-invalid bytes");
  value.set_sensitive(true);
  headers.insert(reqwest::header::AUTHORIZATION, value);

  // Rebuild a reqwest::Client carrying the bearer header and the same
  // TLS / proxy config as the shared one.
  let inner = reqwest::Client::builder()
    .default_headers(headers)
    .add_root_certificate(
      reqwest::Certificate::from_pem(client.root_cert()).expect("invalid root cert"),
    );
  let inner = match client.socks5_proxy() {
    Some(p) => inner.proxy(reqwest::Proxy::all(p).expect("invalid socks5 proxy")),
    None => inner,
  };
  let inner = inner.build().expect("reqwest client build failed");

  // Override spec basePath: csm-rs's `base_url` already ends in `/apis`.
  // Use `new_with_client` (not `new`) so we keep our own timeout/TLS
  // config; `Client::new` bakes in a hard-coded 15s timeout.
  let baseurl = format!("{}/smd/hsm/v2", client.base_url());
  generated::Client::new_with_client(&baseurl, inner)
}

/// Map a generated `Error` into the crate's `Error` enum.
///
/// The arm coverage is intentionally exhaustive (no `_` catch-all) so a
/// future progenitor bump that adds a variant forces an explicit decision
/// here instead of silently collapsing to a generic message. The variant
/// names below are taken verbatim from `progenitor_client::Error<E>` in
/// progenitor-client 0.8 (see Section C of the progenitor output
/// reference doc captured in Task 0).
///
/// With the spec patches applied (Section F of the same doc), error
/// responses are stripped, so generated methods return `Error<()>` and
/// non-2xx bodies surface as `UnexpectedResponse(reqwest::Response)`
/// rather than `ErrorResponse`. We can't synchronously read the response
/// body here (this is a sync mapper), so `UnexpectedResponse` is folded
/// into `Error::Message` carrying status + URL; per-call wrappers that
/// need the body should branch on the variant themselves before calling
/// `map_err`.
pub(crate) fn map_err<E: std::fmt::Debug>(
  err: progenitor_client::Error<E>,
) -> Error {
  use progenitor_client::Error::*;
  match err {
    InvalidRequest(s) => Error::Message(format!("HSM invalid request: {s}")),
    CommunicationError(e) => Error::NetError(e),
    InvalidUpgrade(e) => Error::NetError(e),
    ErrorResponse(rv) => {
      Error::Message(format!("HSM error response: status={} body={:?}", rv.status(), rv.into_inner()))
    }
    ResponseBodyError(e) => Error::NetError(e),
    InvalidResponsePayload(_, e) => Error::SerdeJsonError(e),
    UnexpectedResponse(resp) => Error::Message(format!(
      "HSM unexpected response: status={} url={}",
      resp.status(),
      resp.url()
    )),
    PreHookError(s) => Error::Message(format!("HSM pre-hook error: {s}")),
  }
}
