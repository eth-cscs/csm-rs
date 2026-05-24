pub mod types;

use crate::{
  cfs::configuration::http_client::v2::types::{
    cfs_configuration_request::CfsConfigurationRequest,
    cfs_configuration_response::CfsConfigurationResponse,
  },
  common::http,
  error::Error,
};

const STUPID_LIMIT: i64 = 100000;

pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  configuration_name_opt: Option<&str>,
) -> Result<Vec<CfsConfigurationResponse>, Error> {
  log::info!(
    "Get CFS configuration '{}'",
    configuration_name_opt.unwrap_or("all available")
  );

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;

  let api_url = if let Some(name) = configuration_name_opt {
    format!("{}/cfs/v2/configurations/{}", shasta_base_url, name)
  } else {
    format!("{}/cfs/v2/configurations", shasta_base_url)
  };

  if configuration_name_opt.is_some() {
    let payload: CfsConfigurationResponse = http::get_json_with_query(
      &client,
      &api_url,
      shasta_token,
      &[("limit", STUPID_LIMIT)],
    )
    .await?;
    Ok(vec![payload])
  } else {
    http::get_json_with_query(
      &client,
      &api_url,
      shasta_token,
      &[("limit", STUPID_LIMIT)],
    )
    .await
  }
}

pub async fn get_all(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Vec<CfsConfigurationResponse>, Error> {
  get(shasta_token, shasta_base_url, shasta_root_cert, socks5_proxy, None).await
}

pub async fn put(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  configuration: &CfsConfigurationRequest,
  configuration_name: &str,
) -> Result<CfsConfigurationResponse, Error> {
  log::info!("Create CFS configuration '{}'", configuration_name);
  log::debug!("Create CFS configuration request:\n{:#?}", configuration);

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!(
    "{}/cfs/v2/configurations/{}",
    shasta_base_url, configuration_name
  );

  let request_payload = serde_json::json!({ "layers": configuration.layers });

  log::debug!(
    "CFS configuration request payload:\n{}",
    serde_json::to_string_pretty(&request_payload)
      .unwrap_or_else(|e| format!("<serialize error: {}>", e))
  );

  http::put_json(&client, &api_url, shasta_token, &request_payload).await
}

pub async fn delete(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  configuration_id: &str,
) -> Result<(), Error> {
  log::info!("Delete CFS configuration {:?}", configuration_id);

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!(
    "{}/cfs/v2/configurations/{}",
    shasta_base_url, configuration_id
  );
  http::delete(&client, &api_url, shasta_token).await
}
