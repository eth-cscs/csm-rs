pub mod types;

use std::{sync::Arc, time::Instant};

use tokio::sync::Semaphore;
use types::Component;

use crate::{common::http, error::Error};

pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  components_ids: Option<&str>,
  status: Option<&str>,
) -> Result<Vec<Component>, Error> {
  log::info!("Get CFS components");
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!("{}/cfs/v2/components", shasta_base_url);

  let response = client
    .get(api_url)
    .query(&[("ids", components_ids), ("status", status)])
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
) -> Result<Vec<Component>, Error> {
  get(shasta_token, shasta_base_url, shasta_root_cert, socks5_proxy, None, None).await
}

pub async fn get_single_component(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  component_id: &str,
) -> Result<Component, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url =
    format!("{}/cfs/v2/components/{}", shasta_base_url, component_id);

  let response = client
    .get(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  http::handle_json_or_text_response(response).await
}

/// Get components data.
/// Currently, CSM will throw an error if many xnames are sent in the request, therefore, this
/// method will paralelize multiple calls, each with a batch of xnames
pub async fn get_multiple(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  node_vec: &[String],
) -> Result<Vec<Component>, Error> {
  let start = Instant::now();

  let num_xnames_per_request = 60;
  let pipe_size = 15;

  log::debug!(
    "Number of nodes per request: {num_xnames_per_request}; Pipe size (semaphore): {pipe_size}"
  );

  let mut component_vec = Vec::new();

  let mut tasks = tokio::task::JoinSet::new();

  let sem = Arc::new(Semaphore::new(pipe_size)); // CSM 1.3.1 higher number of concurrent tasks won't

  let num_requests = (node_vec.len() / num_xnames_per_request) + 1;

  let mut i = 1;

  // Calculate number of digits of a number (used for pretty formatting console messages)
  let width = num_requests.checked_ilog10().unwrap_or(0) as usize + 1;

  for sub_node_list in node_vec.chunks(num_xnames_per_request) {
    let num_nodes_in_flight = sub_node_list.len();

    log::info!(
      "Getting CFS components: processing batch [{i:>width$}/{num_requests}] (batch size - {num_nodes_in_flight})"
    );

    let shasta_token_string = shasta_token.to_string();
    let shasta_base_url_string = shasta_base_url.to_string();
    let shasta_root_cert_vec = shasta_root_cert.to_vec();
    let socks5_proxy_opt = socks5_proxy.map(str::to_owned);

    let hsm_subgroup_nodes_string: String = sub_node_list.join(",");

    // Semaphore is never closed → acquire_owned cannot fail.
    let permit = sem
      .clone()
      .acquire_owned()
      .await
      .expect("semaphore not closed");

    tasks.spawn(async move {
      let _permit = permit; // Wait semaphore to allow new tasks https://github.com/tokio-rs/tokio/discussions/2648#discussioncomment-34885

      get(
        &shasta_token_string,
        &shasta_base_url_string,
        &shasta_root_cert_vec,
        socks5_proxy_opt.as_deref(),
        Some(&hsm_subgroup_nodes_string),
        None,
      )
      .await
    });

    i += 1;
  }

  while let Some(message) = tasks.join_next().await {
    component_vec.append(&mut message??);
  }

  let duration = start.elapsed();
  log::info!("Time elapsed to get CFS components is: {:?}", duration);

  Ok(component_vec)
}

/// Get components data.
/// Currently, CSM will throw an error if many xnames are sent in the request, therefore, this
/// method will paralelize multiple calls, each with a batch of xnames
pub async fn get_parallel(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  node_vec: &[String],
) -> Result<Vec<Component>, Error> {
  let start = Instant::now();

  let num_xnames_per_request = 60;
  let pipe_size = 15;

  log::debug!(
    "Number of nodes per request: {num_xnames_per_request}; Pipe size (semaphore): {pipe_size}"
  );

  let mut component_vec = Vec::new();

  let mut tasks = tokio::task::JoinSet::new();

  let sem = Arc::new(Semaphore::new(pipe_size)); // CSM 1.3.1 higher number of concurrent tasks won't

  let num_requests = (node_vec.len() / num_xnames_per_request) + 1;

  let mut i = 1;

  // Calculate number of digits of a number (used for pretty formatting console messages)
  let width = num_requests.checked_ilog10().unwrap_or(0) as usize + 1;

  for sub_node_list in node_vec.chunks(num_xnames_per_request) {
    let num_nodes_in_flight = sub_node_list.len();
    log::info!(
      "Getting CFS components: processing batch [{i:>width$}/{num_requests}] (batch size - {num_nodes_in_flight})"
    );

    let shasta_token_string = shasta_token.to_string();
    let shasta_base_url_string = shasta_base_url.to_string();
    let shasta_root_cert_vec = shasta_root_cert.to_vec();
    let socks5_proxy_opt = socks5_proxy.map(str::to_owned);

    let hsm_subgroup_nodes_string: String = sub_node_list.join(",");

    // Semaphore is never closed → acquire_owned cannot fail.
    let permit = sem
      .clone()
      .acquire_owned()
      .await
      .expect("semaphore not closed");

    tasks.spawn(async move {
      let _permit = permit; // Wait semaphore to allow new tasks https://github.com/tokio-rs/tokio/discussions/2648#discussioncomment-34885

      get_query(
        &shasta_token_string,
        &shasta_base_url_string,
        &shasta_root_cert_vec,
        socks5_proxy_opt.as_deref(),
        None,
        Some(&hsm_subgroup_nodes_string),
        None,
      )
      .await
    });

    i += 1;
  }

  while let Some(message) = tasks.join_next().await {
    component_vec.append(&mut message??);
  }

  let duration = start.elapsed();
  log::info!("Time elapsed to get CFS components is: {:?}", duration);

  Ok(component_vec)
}

pub async fn get_query(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  configuration_name: Option<&str>,
  components_ids: Option<&str>,
  status: Option<&str>,
) -> Result<Vec<Component>, Error> {
  let stupid_limit = 100000;

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!("{}/cfs/v2/components", shasta_base_url);

  let response = client
    .get(api_url)
    .query(&[
      ("ids", components_ids),
      ("config_name", configuration_name),
      ("status", status),
      ("limit", Some(&stupid_limit.to_string())),
    ])
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  http::handle_json_or_text_response(response).await
}

pub async fn put_component(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  component: Component,
) -> Result<Component, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let component_id = component.id.as_deref().ok_or_else(|| {
    Error::CfsComponentFieldNotDefined("id".to_string())
  })?;
  let api_url =
    format!("{}/cfs/v2/components/{}", shasta_base_url, component_id);
  http::put_json(&client, &api_url, shasta_token, &component).await
}

pub async fn put_component_list(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  component_list: Vec<Component>,
) -> Result<Vec<Component>, Error> {
  let mut result_vec: Vec<Result<Component, Error>> = Vec::new();

  for component in component_list {
    let result = put_component(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      component,
    )
    .await;
    result_vec.push(result);
  }

  // Convert from Vec<Result<Component, Error>> to Result<Vec<Component>, Error>>
  result_vec.into_iter().collect()
}

pub async fn delete_single_component(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  component_id: &str,
) -> Result<Component, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url =
    format!("{}/cfs/v2/components/{}", shasta_base_url, component_id);

  let response = client
    .delete(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  http::handle_json_or_text_response(response).await
}
