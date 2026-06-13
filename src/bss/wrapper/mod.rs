//! Thin wrapper bridging the generated BSS client to the public
//! `ShastaClient` API. Mirrors `crate::hsm::wrapper` — see its
//! module-level docs for the design rationale.
//!
//! Responsibilities:
//!  - construct a per-call generated `Client` with bearer auth baked in;
//!  - override the spec's basePath with the URL shape csm-rs has always
//!    used (`{base_url}/bss`);
//!  - map `progenitor_client::Error<T>` into `crate::error::Error` (async,
//!    reads the body for UnexpectedResponse/ErrorResponse — same idiom
//!    as `crate::hsm::wrapper::map_err`).
//!
//! `impl ShastaClient { pub async fn bss_bootparameters_*() }` blocks
//! live in this file directly because BSS has only one resource
//! (bootparameters). Task 3 appends them here.

use crate::{ShastaClient, bss::generated, error::Error};

pub(crate) fn gen_client(
  client: &ShastaClient,
  token: &str,
) -> Result<generated::Client, Error> {
  let inner = crate::common::http::build_client_with_auth(
    client.root_cert(),
    client.socks5_proxy(),
    Some(token),
  )?;
  let baseurl = format!("{}/bss", client.base_url());
  Ok(generated::Client::new_with_client(&baseurl, inner))
}

#[allow(clippy::enum_glob_use, clippy::match_same_arms)]
pub(crate) async fn map_err<E: std::fmt::Debug>(
  err: progenitor_client::Error<E>,
) -> Error {
  use progenitor_client::Error::*;
  match err {
    InvalidRequest(s) => Error::Message(format!("BSS invalid request: {s}")),
    CommunicationError(e) => Error::NetError(e),
    InvalidUpgrade(e) => Error::NetError(e),
    ErrorResponse(rv) => {
      let status = rv.status();
      Error::Message(format!(
        "BSS error response: status={status} body={:?}",
        rv.into_inner()
      ))
    }
    ResponseBodyError(e) => Error::NetError(e),
    InvalidResponsePayload(_, e) => Error::SerdeJsonError(e),
    UnexpectedResponse(resp) => {
      let status = resp.status();
      let url = resp.url().clone();
      let body = resp
        .text()
        .await
        .unwrap_or_else(|e| format!("<body read failed: {e}>"));
      Error::Message(format!(
        "BSS unexpected response: status={status} url={url} body={body}"
      ))
    }
    PreHookError(s) => Error::Message(format!("BSS pre-hook error: {s}")),
  }
}

pub(crate) async fn run<F, Fut, T, E>(
  client: &ShastaClient,
  token: &str,
  op: F,
) -> Result<T, Error>
where
  F: FnOnce(generated::Client) -> Fut,
  Fut: std::future::Future<
      Output = Result<
        progenitor_client::ResponseValue<T>,
        progenitor_client::Error<E>,
      >,
    >,
  E: std::fmt::Debug,
{
  let gc = gen_client(client, token)?;
  match op(gc).await {
    Ok(rv) => Ok(rv.into_inner()),
    Err(e) => Err(map_err(e).await),
  }
}

// Task 3 inserts an `impl ShastaClient { ... }` block here with all
// 6 `bss_bootparameters_*` wrapper methods.
