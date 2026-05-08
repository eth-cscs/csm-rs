use std::time;

use crate::{
  error::Error,
  pcs::transitions::types::{
    Location, Operation, TransitionResponse, TransitionResponseList,
    TransitionStartOutput,
  },
};

use super::types::Transition;

pub async fn get(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Vec<TransitionResponse>, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

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
  socks5_proxy: Option<&str>,
  id: &str,
) -> Result<TransitionResponse, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

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
  socks5_proxy: Option<&str>,
  operation: &str,
  xname_vec: &Vec<String>,
) -> Result<TransitionStartOutput, Error> {
  log::info!("Create PCS transition '{}' on {:?}", operation, xname_vec);

  let mut location_vec: Vec<Location> = Vec::new();

  for xname in xname_vec {
    let location: Location = Location {
      xname: xname.to_string(),
      deputy_key: None,
    };

    location_vec.push(location);
  }

  let request_payload = Transition {
    operation: Operation::from_str(operation)?,
    task_deadline_minutes: None,
    location: location_vec,
  };

  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url = shasta_base_url.to_owned() + "/power-control/v1/transitions";

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
        .json::<TransitionStartOutput>()
        .await
        .map_err(|e| Error::NetError(e))?,
    )
  } else {
    let error_payload =
      response.json().await.map_err(|e| Error::NetError(e))?;

    Err(Error::CsmError(error_payload))
  }
}

pub async fn post_block(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  operation: &str,
  xname_vec: &Vec<String>,
) -> Result<TransitionResponse, Error> {
  let power_transition = post(
    shasta_base_url,
    shasta_token,
    shasta_root_cert,
    socks5_proxy,
    operation,
    xname_vec,
  )
  .await?;

  log::info!("PCS transition ID: {}", power_transition.transition_id);

  let power_management_status: TransitionResponse = wait_to_complete(
    shasta_base_url,
    shasta_token,
    shasta_root_cert,
    socks5_proxy,
    &power_transition.transition_id,
  )
  .await?;

  Ok(power_management_status)
}

pub async fn wait_to_complete(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  transition_id: &str,
) -> Result<TransitionResponse, Error> {
  let mut transition: TransitionResponse = get_by_id(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    socks5_proxy,
    transition_id,
  )
  .await?;

  let mut i = 1;
  let max_attempt = 300;

  while i <= max_attempt && transition.transition_status != "completed" {
    transition = get_by_id(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
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
