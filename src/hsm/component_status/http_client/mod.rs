use reqwest::Url;
use serde_json::Value;

use crate::{common::http, error::Error};

pub async fn get_raw(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname_vec: &[String],
) -> Result<Vec<Value>, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;

  let url_params: Vec<_> =
    xname_vec.iter().map(|xname| ("id", xname)).collect();

  let api_url = Url::parse_with_params(
    &format!("{}/smd/hsm/v2/State/Components", shasta_base_url),
    &url_params,
  )
  .map_err(|e| {
    Error::Message(format!(
      "Could not build HSM components URL from base '{}': {}",
      shasta_base_url, e
    ))
  })?;

  let response: Value =
    http::get_json(&client, api_url.as_str(), shasta_token).await?;

  Ok(
    response
      .get("Components")
      .and_then(Value::as_array)
      .cloned()
      .unwrap_or_default(),
  )
}

/// Fetches nodes/compnents details using HSM v2 ref --> https://apidocs.svc.cscs.ch/iaas/hardware-state-manager/operation/doComponentsGet/
pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname_vec: &[String],
) -> Result<Vec<Value>, Error> {
  let chunk_size = 30;

  let mut hsm_component_status_vec: Vec<Value> = Vec::new();

  let mut tasks = tokio::task::JoinSet::new();

  for sub_node_list in xname_vec.chunks(chunk_size) {
    let shasta_token_string = shasta_token.to_string();
    let shasta_base_url_string = shasta_base_url.to_string();
    let shasta_root_cert_vec = shasta_root_cert.to_vec();
    let socks5_proxy_opt = socks5_proxy.map(str::to_owned);

    let node_vec = sub_node_list.to_vec();

    tasks.spawn(async move {
      get_raw(
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
    hsm_component_status_vec.append(&mut message??);
  }

  Ok(hsm_component_status_vec)
}
