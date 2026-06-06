//! `ShastaClient` methods for `/bss/boot/v1/bootparameters`.

use core::result::Result;
use std::time::Instant;

use crate::{ShastaClient, common::http, error::Error};

use super::types::BootParameters;

impl ShastaClient {
  /// Get node boot params. Ref: <https://apidocs.svc.cscs.ch/iaas/bss/tag/bootparameters/paths/~1bootparameters/get/>.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bss_bootparameters_get(
    &self,
    token: &str,
    xnames: &[String],
  ) -> Result<Vec<BootParameters>, Error> {
    log::debug!("Get BSS bootparameters");

    let url_api = format!("{}/bss/boot/v1/bootparameters", self.base_url());

    let params: Vec<_> = xnames.iter().map(|xname| ("name", xname)).collect();

    let response = self
      .http()
      .get(url_api)
      .query(&params)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  /// `GET /bss/boot/v1/bootparameters` — fetch boot parameters for
  /// every node BSS knows about.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bss_bootparameters_get_all(
    &self,
    token: &str,
  ) -> Result<Vec<BootParameters>, Error> {
    self.bss_bootparameters_get(token, &[]).await
  }

  /// `GET /bss/boot/v1/bootparameters` for many xnames, parallelised
  /// in chunks of 30 with up to 10 concurrent batches.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bss_bootparameters_get_multiple(
    &self,
    token: &str,
    xnames: &[String],
  ) -> Result<Vec<BootParameters>, Error> {
    let start = Instant::now();

    let client = self.clone();
    let token = token.to_string();
    let boot_params_vec = http::parallel_batch(xnames, 30, 10, move |chunk| {
      let client = client.clone();
      let token = token.clone();
      async move { client.bss_bootparameters_get(&token, &chunk).await }
    })
    .await?;

    log::debug!(
      "Time elapsed to get BSS bootparameters is: {:?}",
      start.elapsed()
    );
    Ok(boot_params_vec)
  }

  /// Change nodes boot params. Ref: <https://apidocs.svc.cscs.ch/iaas/bss/tag/bootparameters/paths/~1bootparameters/put/>.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bss_bootparameters_put(
    &self,
    token: &str,
    boot_parameters: BootParameters,
  ) -> Result<BootParameters, Error> {
    let api_url = format!("{}/bss/boot/v1/bootparameters", self.base_url());

    log::debug!(
      "request payload:\n{}",
      serde_json::to_string_pretty(&boot_parameters)?
    );

    let response = self
      .http()
      .put(api_url)
      .json(&boot_parameters)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    if response.status().is_success() {
      Ok(response.json().await?)
    } else {
      Err(Error::Message(response.text().await?))
    }
  }

  /// POST a single set of BootParameters. Used to create new entries.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bss_bootparameters_post(
    &self,
    token: &str,
    boot_parameters: BootParameters,
  ) -> Result<(), Error> {
    let api_url = format!("{}/bss/boot/v1/bootparameters", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&boot_parameters)
      .send()
      .await
      .map_err(Error::NetError)?;

    if response.status().is_success() {
      Ok(())
    } else {
      Err(Error::Message(response.text().await?))
    }
  }

  /// `PATCH /bss/boot/v1/bootparameters` — partial update of an
  /// existing entry.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bss_bootparameters_patch(
    &self,
    token: &str,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    let api_url = format!("{}/bss/boot/v1/bootparameters", self.base_url());

    let response = self
      .http()
      .patch(api_url)
      .json(&boot_parameters)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    if response.status().is_success() {
      Ok(())
    } else {
      Err(Error::Message(response.text().await?))
    }
  }
}
