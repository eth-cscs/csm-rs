use std::str::FromStr;
use std::time;

use crate::{
  common::http,
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
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url = format!("{}/power-control/v1/transitions", shasta_base_url);

  let list: TransitionResponseList =
    http::get_json(&client, &url, shasta_token).await?;
  Ok(list.transitions)
}

pub async fn get_by_id(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  id: &str,
) -> Result<TransitionResponse, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url =
    format!("{}/power-control/v1/transitions/{}", shasta_base_url, id);

  let transition: TransitionResponse =
    http::get_json(&client, &url, shasta_token).await?;
  log::debug!("PCS transition details\n{:#?}", transition);
  Ok(transition)
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

  let location_vec: Vec<Location> = xname_vec
    .iter()
    .map(|xname| Location {
      xname: xname.to_string(),
      deputy_key: None,
    })
    .collect();

  let request_payload = Transition {
    operation: Operation::from_str(operation)?,
    task_deadline_minutes: None,
    location: location_vec,
  };

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url = format!("{}/power-control/v1/transitions", shasta_base_url);

  http::post_json(&client, &url, shasta_token, &request_payload).await
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
