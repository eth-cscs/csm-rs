//! Internal HTTP helpers shared by all `*::http_client` modules.
//!
//! Centralizes the three patterns that were duplicated across ~30 files:
//!   1. Building a `reqwest::Client` with the CSM root cert and optional SOCKS5 proxy.
//!   2. Issuing a bearer-authenticated request.
//!   3. Branching on response status: deserialize success body as `T`, or map
//!      a non-success status to `Error::CsmError(Value)`.
//!
//! This module is `pub(crate)` — it intentionally does not change csm-rs's
//! public API. Existing `pub async fn x(shasta_token, shasta_base_url, ...)`
//! free functions delegate here, but their signatures stay the same.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::Error;

/// Build a `reqwest::Client` configured with the CSM root certificate and an
/// optional SOCKS5 proxy. This is the per-request setup that used to be
/// inlined at every call site.
pub(crate) fn build_client(
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<reqwest::Client, Error> {
  let builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => builder.build()?,
  };

  Ok(client)
}

/// On a 2xx response, deserialize the body as `T`. On any other status,
/// deserialize the body as `serde_json::Value` and return `Error::CsmError`.
pub(crate) async fn handle_json_response<T: DeserializeOwned>(
  response: reqwest::Response,
) -> Result<T, Error> {
  if response.status().is_success() {
    response.json::<T>().await.map_err(Error::NetError)
  } else {
    let payload = response
      .json::<Value>()
      .await
      .map_err(Error::NetError)?;
    Err(Error::CsmError(payload))
  }
}

/// On a 2xx response, deserialize the body as `T`. On any other status,
/// read the body as text and return `Error::Message`. Used by endpoints
/// (mostly CFS v3 and BSS) whose error payloads are plain text, not JSON.
pub(crate) async fn handle_json_or_text_response<T: DeserializeOwned>(
  response: reqwest::Response,
) -> Result<T, Error> {
  if response.status().is_success() {
    response.json::<T>().await.map_err(Error::NetError)
  } else {
    let text = response.text().await.map_err(Error::NetError)?;
    Err(Error::Message(text))
  }
}

/// GET `url` with bearer auth, deserialize success body as `T`.
pub(crate) async fn get_json<T: DeserializeOwned>(
  client: &reqwest::Client,
  url: &str,
  shasta_token: &str,
) -> Result<T, Error> {
  let response = client
    .get(url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  handle_json_response(response).await
}

/// POST JSON `body` to `url` with bearer auth, deserialize success body as `T`.
pub(crate) async fn post_json<B, T>(
  client: &reqwest::Client,
  url: &str,
  shasta_token: &str,
  body: &B,
) -> Result<T, Error>
where
  B: Serialize + ?Sized,
  T: DeserializeOwned,
{
  let response = client
    .post(url)
    .json(body)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  handle_json_response(response).await
}

/// PUT JSON `body` to `url` with bearer auth, deserialize success body as `T`.
pub(crate) async fn put_json<B, T>(
  client: &reqwest::Client,
  url: &str,
  shasta_token: &str,
  body: &B,
) -> Result<T, Error>
where
  B: Serialize + ?Sized,
  T: DeserializeOwned,
{
  let response = client
    .put(url)
    .json(body)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  handle_json_response(response).await
}

/// GET `url` with bearer auth and a query string, deserialize success body as `T`.
/// `query` is anything `serde_urlencoded` can serialize, e.g. `&[("limit", 100000)]`.
pub(crate) async fn get_json_with_query<Q, T>(
  client: &reqwest::Client,
  url: &str,
  shasta_token: &str,
  query: &Q,
) -> Result<T, Error>
where
  Q: Serialize + ?Sized,
  T: DeserializeOwned,
{
  let response = client
    .get(url)
    .query(query)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  handle_json_response(response).await
}

/// On a 2xx response, deserialize the body as `T`. On `UNAUTHORIZED`, return
/// `Error::RequestError { response, payload: text }`. On any other status,
/// deserialize the body as JSON and return `Error::CsmError`. This is the
/// pattern used by HSM endpoints that distinguish auth failures from other
/// API errors.
pub(crate) async fn handle_json_or_request_error<T: DeserializeOwned>(
  response: reqwest::Response,
) -> Result<T, Error> {
  if let Err(e) = response.error_for_status_ref() {
    match response.status() {
      reqwest::StatusCode::UNAUTHORIZED => {
        let payload = response.text().await.map_err(Error::NetError)?;
        return Err(Error::RequestError {
          response: e,
          payload,
        });
      }
      _ => {
        let payload = response
          .json::<Value>()
          .await
          .map_err(Error::NetError)?;
        return Err(Error::CsmError(payload));
      }
    }
  }

  response.json().await.map_err(Error::NetError)
}

/// DELETE `url` with bearer auth. Returns unit on 2xx; otherwise
/// `Error::CsmError(json)`.
pub(crate) async fn delete(
  client: &reqwest::Client,
  url: &str,
  shasta_token: &str,
) -> Result<(), Error> {
  let response = client
    .delete(url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  if response.status().is_success() {
    Ok(())
  } else {
    let payload = response
      .json::<Value>()
      .await
      .map_err(Error::NetError)?;
    Err(Error::CsmError(payload))
  }
}
