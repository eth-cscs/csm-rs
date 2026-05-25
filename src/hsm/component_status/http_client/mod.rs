//! `ShastaClient` methods for `/smd/hsm/v2/State/Components`.

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
