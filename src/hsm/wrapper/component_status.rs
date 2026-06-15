//! Wrapper for chunked `GET /smd/hsm/v2/State/Components`. Replaces
//! `src/hsm/component_status/http_client/mod.rs`.
//!
//! Strategy: callers may pass an arbitrary-length xname list; CSM rejects
//! a single GET with more than ~30 ids, so `hsm_component_status_get`
//! chunks the input and fans out the per-batch GETs through
//! [`crate::common::http::parallel_batch`]. Order of the merged results
//! is not preserved.
//!
//! **Both methods stay on raw `reqwest`.** Routing decisions per method:
//!
//! - `hsm_component_status_get_raw` issues `GET /State/Components` with a
//!   repeated `?id=` query parameter per xname (CSM accepts repeats).
//!   The generated `do_components_get` types `id` as `Option<&str>` — a
//!   single value — so wrapping it would either drop ids on the floor or
//!   require N sequential calls per chunk, defeating the point. Public
//!   return type is `Vec<serde_json::Value>` extracted from the
//!   `Components` field of the response; the generated binding returns
//!   the strongly-typed `ComponentArrayComponentArray`, so a switch
//!   would also be a load-bearing public-API break (consumers index into
//!   the value with `.get("State")` etc., see
//!   `src/commands/apply_session.rs`).
//! - `hsm_component_status_get` is a pure chunking wrapper over
//!   `hsm_component_status_get_raw` — no endpoint of its own, so it
//!   inherits the same status.

use reqwest::Url;
use serde_json::Value;

use crate::{ShastaClient, common::http, error::Error};

impl ShastaClient {
  /// Fetch the HSM `Components` block for the given xnames in a single
  /// request and return the raw JSON array.
  ///
  /// `GET /smd/hsm/v2/State/Components?id=…&id=…`. Use
  /// [`Self::hsm_component_status_get`] instead when the list might be
  /// large — CSM rejects requests with more than ~30 ids.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_component_status_get_raw(
    &self,
    token: &str,
    xname_vec: &[String],
  ) -> Result<Vec<Value>, Error> {
    let url_params: Vec<_> =
      xname_vec.iter().map(|xname| ("id", xname)).collect();

    let api_url = Url::parse_with_params(
      &format!("{}/smd/hsm/v2/State/Components", self.base_url()),
      &url_params,
    )
    .map_err(|e| {
      Error::Message(format!(
        "Could not build HSM components URL from base '{}': {}",
        self.base_url(),
        e
      ))
    })?;

    let response: Value =
      http::get_json(self.http(), api_url.as_str(), token).await?;

    Ok(
      response
        .get("Components")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default(),
    )
  }

  /// Fetch HSM component state for an arbitrarily large xname list,
  /// chunking into requests of 30 and running them concurrently.
  ///
  /// Wraps [`Self::hsm_component_status_get_raw`] to work around the
  /// per-request id limit on `GET /smd/hsm/v2/State/Components`. Order
  /// of the returned values is not preserved.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_component_status_get(
    &self,
    token: &str,
    xname_vec: &[String],
  ) -> Result<Vec<Value>, Error> {
    let client = self.clone();
    let token = token.to_string();
    // No semaphore in the original code — pick a high cap.
    http::parallel_batch(xname_vec, 30, 1024, move |chunk| {
      let client = client.clone();
      let token = token.clone();
      async move { client.hsm_component_status_get_raw(&token, &chunk).await }
    })
    .await
  }
}
