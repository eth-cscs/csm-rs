use tokio::sync::Semaphore;

use core::result::Result;
use std::{sync::Arc, time::Instant};

use crate::{common::http, error::Error};

use super::types::BootParameters;

/// Get node boot params, ref --> https://apidocs.svc.cscs.ch/iaas/bss/tag/bootparameters/paths/~1bootparameters/get/
pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xnames: &[String],
) -> Result<Vec<BootParameters>, Error> {
  log::info!("Get BSS bootparameters");

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url_api = format!("{}/bss/boot/v1/bootparameters", shasta_base_url);

  let params: Vec<_> = xnames.iter().map(|xname| ("name", xname)).collect();

  let response = client
    .get(url_api)
    .query(&params)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  http::handle_json_or_text_response(response).await
}

pub async fn get_all(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Vec<BootParameters>, Error> {
  get(shasta_token, shasta_base_url, shasta_root_cert, socks5_proxy, &[]).await
}

pub async fn get_multiple(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xnames: &[String],
) -> Result<Vec<BootParameters>, Error> {
  let start = Instant::now();

  let chunk_size = 30;

  let mut boot_params_vec = Vec::new();

  let mut tasks = tokio::task::JoinSet::new();

  let sem = Arc::new(Semaphore::new(10)); // CSM 1.3.1 higher number of concurrent tasks won't

  for sub_node_list in xnames.chunks(chunk_size) {
    let shasta_token_string = shasta_token.to_string();
    let shasta_base_url_string = shasta_base_url.to_string();
    let shasta_root_cert_vec = shasta_root_cert.to_vec();
    let socks5_proxy_opt = socks5_proxy.map(str::to_owned);

    let permit = Arc::clone(&sem).acquire_owned().await;

    let node_vec = sub_node_list.to_vec();

    tasks.spawn(async move {
      let _permit = permit; // Wait semaphore to allow new tasks https://github.com/tokio-rs/tokio/discussions/2648#discussioncomment-34885

      get(
        &shasta_token_string,
        &shasta_base_url_string,
        &shasta_root_cert_vec,
        socks5_proxy_opt.as_deref(),
        &node_vec,
      )
      .await
    });
  }

  while let Some(message) = tasks.join_next().await {
    boot_params_vec.append(&mut message??);
  }

  let duration = start.elapsed();
  log::info!("Time elapsed to get BSS bootparameters is: {:?}", duration);

  Ok(boot_params_vec)
}

pub fn post(
  base_url: &str,
  auth_token: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  boot_parameters: BootParameters,
) -> Result<(), Error> {
  // NOTE: this is the only blocking (non-async) call in bss; the helper module
  // is async-only, so we keep the inline reqwest::blocking client.
  let client_builder = reqwest::blocking::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url = format!("{}/boot/v1/bootparameters", base_url);

  let response = client
    .post(api_url)
    .bearer_auth(auth_token)
    .json(&boot_parameters)
    .send()
    .map_err(Error::NetError)?;

  if response.status().is_success() {
    Ok(())
  } else {
    Err(Error::Message(response.text()?))
  }
}

/// Change nodes boot params, ref --> https://apidocs.svc.cscs.ch/iaas/bss/tag/bootparameters/paths/~1bootparameters/put/
pub async fn put(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  boot_parameters: BootParameters,
) -> Result<BootParameters, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!("{}/bss/boot/v1/bootparameters", shasta_base_url);

  log::debug!(
    "request payload:\n{}",
    serde_json::to_string_pretty(&boot_parameters)?
  );

  let response = client
    .put(api_url)
    .json(&boot_parameters)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  if response.status().is_success() {
    Ok(response.json().await?)
  } else {
    Err(Error::Message(response.text().await?))
  }
}

pub async fn patch(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  boot_parameters: &BootParameters,
) -> Result<(), Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!("{}/bss/boot/v1/bootparameters", shasta_base_url);

  let response = client
    .patch(api_url)
    .json(&boot_parameters)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  if response.status().is_success() {
    Ok(())
  } else {
    Err(Error::Message(response.text().await?))
  }
}
