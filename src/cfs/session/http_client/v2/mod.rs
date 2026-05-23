pub mod types;

use crate::{common::http, error::Error};

use super::v2::types::{CfsSessionGetResponse, CfsSessionPostRequest};

/// Fetch CFS sessions ref --> https://apidocs.svc.cscs.ch/paas/cfs/operation/get_sessions/
// FIX: change parameters types from '&String' to '&str'
pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  min_age_opt: Option<&String>,
  max_age_opt: Option<&String>,
  status_opt: Option<&String>,
  session_name_opt: Option<&String>,
  is_succeded_opt: Option<bool>,
) -> Result<Vec<CfsSessionGetResponse>, Error> {
  log::info!(
    "Get CFS sessions '{}'",
    session_name_opt.unwrap_or(&"all available".to_string())
  );

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;

  let api_url = if let Some(session_name) = session_name_opt {
    format!("{}/cfs/v2/sessions/{}", shasta_base_url, session_name)
  } else {
    format!("{}/cfs/v2/sessions", shasta_base_url)
  };

  let mut query_params: Vec<(&str, String)> = Vec::new();
  if let Some(is_succeded) = is_succeded_opt {
    query_params.push(("succeced", is_succeded.to_string()));
  }
  if let Some(min_age) = min_age_opt {
    query_params.push(("min_age", min_age.to_string()));
  }
  if let Some(max_age) = max_age_opt {
    query_params.push(("max_age", max_age.to_string()));
  }
  if let Some(status) = status_opt {
    query_params.push(("status", status.to_string()));
  }

  if session_name_opt.is_some() {
    let payload: CfsSessionGetResponse =
      http::get_json_with_query(&client, &api_url, shasta_token, &query_params)
        .await?;
    Ok(vec![payload])
  } else {
    http::get_json_with_query(&client, &api_url, shasta_token, &query_params)
      .await
  }
}

pub async fn get_all(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Vec<CfsSessionGetResponse>, Error> {
  get(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    socks5_proxy,
    None,
    None,
    None,
    None,
    None,
  )
  .await
}

pub async fn post(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  session: &CfsSessionPostRequest,
) -> Result<CfsSessionGetResponse, Error> {
  log::debug!("Session:\n{:#?}", session);

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!("{}/cfs/v2/sessions", shasta_base_url);
  http::post_json(&client, &api_url, shasta_token, session).await
}

pub async fn delete(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  session_name: &str,
) -> Result<(), Error> {
  log::info!("Deleting CFS session id: {}", session_name);

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url =
    format!("{}/cfs/v2/sessions/{}", shasta_base_url, session_name);
  http::delete(&client, &api_url, shasta_token).await
}
