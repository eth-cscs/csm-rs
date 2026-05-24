use tokio::sync::Semaphore;

use core::result::Result;
use std::{sync::Arc, time::Instant};

use crate::{ShastaClient, common::http, error::Error};

use super::types::BootParameters;

impl ShastaClient {
  /// Get node boot params, ref --> https://apidocs.svc.cscs.ch/iaas/bss/tag/bootparameters/paths/~1bootparameters/get/
  pub async fn bss_bootparameters_get(
    &self,
    xnames: &[String],
  ) -> Result<Vec<BootParameters>, Error> {
    log::info!("Get BSS bootparameters");

    let url_api = format!("{}/bss/boot/v1/bootparameters", self.base_url());

    let params: Vec<_> = xnames.iter().map(|xname| ("name", xname)).collect();

    let response = self
      .http()
      .get(url_api)
      .query(&params)
      .bearer_auth(self.token())
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  pub async fn bss_bootparameters_get_all(
    &self,
  ) -> Result<Vec<BootParameters>, Error> {
    self.bss_bootparameters_get(&[]).await
  }

  pub async fn bss_bootparameters_get_multiple(
    &self,
    xnames: &[String],
  ) -> Result<Vec<BootParameters>, Error> {
    let start = Instant::now();

    let chunk_size = 30;

    let mut boot_params_vec = Vec::new();

    let mut tasks = tokio::task::JoinSet::new();

    let sem = Arc::new(Semaphore::new(10)); // CSM 1.3.1 higher number of concurrent tasks won't

    for sub_node_list in xnames.chunks(chunk_size) {
      let permit = Arc::clone(&sem).acquire_owned().await;

      let node_vec = sub_node_list.to_vec();
      let client = self.clone();

      tasks.spawn(async move {
        let _permit = permit; // Wait semaphore to allow new tasks https://github.com/tokio-rs/tokio/discussions/2648#discussioncomment-34885

        client.bss_bootparameters_get(&node_vec).await
      });
    }

    while let Some(message) = tasks.join_next().await {
      boot_params_vec.append(&mut message??);
    }

    let duration = start.elapsed();
    log::info!("Time elapsed to get BSS bootparameters is: {:?}", duration);

    Ok(boot_params_vec)
  }

  /// Change nodes boot params, ref --> https://apidocs.svc.cscs.ch/iaas/bss/tag/bootparameters/paths/~1bootparameters/put/
  pub async fn bss_bootparameters_put(
    &self,
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
      .bearer_auth(self.token())
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
  pub async fn bss_bootparameters_post(
    &self,
    boot_parameters: BootParameters,
  ) -> Result<(), Error> {
    let api_url = format!("{}/bss/boot/v1/bootparameters", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(self.token())
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

  pub async fn bss_bootparameters_patch(
    &self,
    boot_parameters: &BootParameters,
  ) -> Result<(), Error> {
    let api_url = format!("{}/bss/boot/v1/bootparameters", self.base_url());

    let response = self
      .http()
      .patch(api_url)
      .json(&boot_parameters)
      .bearer_auth(self.token())
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

