use std::time;

use serde_json::Value;

use crate::{
  error::Error,
  pcs::transitions::types::{
    Location, Operation, TransitionResponse, TransitionResponseList,
  },
};

use super::types::Transition;

pub async fn get(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
) -> Result<Vec<TransitionResponse>, Error> {
  let client;

  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  // Build client
  if std::env::var("SOCKS5").is_ok() {
    // socks5 proxy
    log::debug!("SOCKS5 enabled");
    let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5")?)?;
    client = client_builder.proxy(socks5proxy).build()?;
  } else {
    client = client_builder.build()?;
  }

  let api_url = format!("{}/power-control/v1/transitions", shasta_base_url);

  let response = client
    .get(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(|error| Error::NetError(error))?;

  if response.status().is_success() {
    response
      .json::<TransitionResponseList>()
      .await
      .map(|transition_list| transition_list.transitions)
      .map_err(|error| Error::NetError(error))
  } else {
    let error_payload = response
      .json()
      .await
      .map_err(|error| Error::NetError(error))?;

    Err(Error::CsmError(error_payload))
  }
}

pub async fn get_by_id(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  id: &str,
) -> Result<TransitionResponse, Error> {
  let client;

  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  // Build client
  if std::env::var("SOCKS5").is_ok() {
    // socks5 proxy
    log::debug!("SOCKS5 enabled");
    let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5")?)?;
    client = client_builder.proxy(socks5proxy).build()?;
  } else {
    client = client_builder.build()?;
  }

  let api_url =
    format!("{}/power-control/v1/transitions/{}", shasta_base_url, id);

  let response = client
    .get(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(|error| Error::NetError(error))?;

  if response.status().is_success() {
    let payload = response
      .json()
      .await
      .map_err(|error| Error::NetError(error));

    log::debug!("PCS transition details\n{:#?}", payload);

    payload
  } else {
    let error_payload = response
      .json()
      .await
      .map_err(|error| Error::NetError(error))?;

    Err(Error::CsmError(error_payload))
  }
}

pub async fn post(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  operation: &str,
  xname_vec: &Vec<String>,
) -> Result<TransitionResponse, Error> {
  log::info!("Create PCS transition '{}' on {:?}", operation, xname_vec);

  //Create request payload
  //
  // Create 'location' list with all the xnames to operate
  let mut location_vec: Vec<Location> = Vec::new();

  for xname in xname_vec {
    let location: Location = Location {
      xname: xname.to_string(),
      deputy_key: None,
    };

    location_vec.push(location);
  }

  // Create 'transition'
  let request_payload = Transition {
    operation: Operation::from_str(operation)?,
    task_deadline_minutes: None,
    location: location_vec,
  };

  // Build http client
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  let client = if let Ok(socks5_env) = std::env::var("SOCKS5") {
    // socks5 proxy
    log::debug!("SOCKS5 enabled");
    let socks5proxy = reqwest::Proxy::all(socks5_env)?;

    // rest client to authenticate
    client_builder.proxy(socks5proxy).build()?
  } else {
    client_builder.build()?
  };

  let api_url = shasta_base_url.to_owned() + "/power-control/v1/transitions";

  // Submit call to http api
  let response = client
    .post(api_url)
    .json(&request_payload)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(|error| Error::NetError(error))?;

  if response.status().is_success() {
    Ok(
      response
        .json::<TransitionResponse>()
        .await
        .map_err(|e| Error::NetError(e))?,
    )
  } else {
    let error_payload =
      response.json().await.map_err(|e| Error::NetError(e))?;

    Err(Error::CsmError(error_payload))
  }
}

// Creates a task on CSM for power management nodes.
// Returns a serde_json::Value with the power task management
pub async fn post_block(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  operation: &str,
  xname_vec: &Vec<String>,
) -> Result<TransitionResponse, Error> {
  let power_transition = post(
    shasta_base_url,
    shasta_token,
    shasta_root_cert,
    operation,
    xname_vec,
  )
  .await?;

  log::info!("PCS transition ID: {}", power_transition.transition_id);

  let power_management_status: TransitionResponse = wait_to_complete(
    shasta_base_url,
    shasta_token,
    shasta_root_cert,
    &power_transition.transition_id,
  )
  .await?;

  Ok(power_management_status)
}

pub async fn wait_to_complete(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  transition_id: &str,
) -> Result<TransitionResponse, Error> {
  let mut transition: TransitionResponse = get_by_id(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    transition_id,
  )
  .await?;

  let mut i = 1;
  let max_attempt = 300;

  while i <= max_attempt && transition.transition_status != "completed" {
    // Check PCS transition status
    transition = get_by_id(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      transition_id,
    )
    .await?;

    eprintln!(
      "Power '{}' summary - status: {}, failed: {}, in-progress: {}, succeeded: {}, total: {}. Attempt {} of {}",
      transition.operation,
      transition.transition_status,
      transition.task_counts.failed,
      transition.task_counts.in_progress,
      transition.task_counts.succeeded,
      transition.task_counts.total,
      i,
      max_attempt
    );

    tokio::time::sleep(time::Duration::from_secs(3)).await;
    i += 1;
  }

  Ok(transition)
}
