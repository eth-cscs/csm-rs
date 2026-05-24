pub mod types;

use crate::{
  bos::template::http_client::v1::types::BosSessionTemplate,
  common::http,
  error::Error,
};

/// Get BOS session templates. Ref --> https://apidocs.svc.cscs.ch/paas/bos/operation/get_v1_sessiontemplates/
pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  bos_session_template_id_opt: Option<&String>,
) -> Result<Vec<BosSessionTemplate>, Error> {
  log::info!(
    "Get BOS sessiontemplates '{}'",
    bos_session_template_id_opt.unwrap_or(&"all available".to_string())
  );

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;

  let api_url = if let Some(id) = bos_session_template_id_opt {
    format!("{}/bos/v1/sessiontemplate/{}", shasta_base_url, id)
  } else {
    format!("{}/bos/v1/sessiontemplate", shasta_base_url)
  };

  if bos_session_template_id_opt.is_none() {
    http::get_json(&client, &api_url, shasta_token).await
  } else {
    let single: BosSessionTemplate =
      http::get_json(&client, &api_url, shasta_token).await?;
    Ok(vec![single])
  }
}

pub async fn post(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  bos_template: &BosSessionTemplate,
) -> Result<String, Error> {
  log::info!("Create BOS sessiontemplate '{}'", bos_template.name);
  log::debug!(
    "Create BOS sessiontemplate request payload:\n{}",
    serde_json::to_string_pretty(bos_template)
      .unwrap_or_else(|e| format!("<serialize error: {}>", e))
  );

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!("{}/bos/v1/sessiontemplate", shasta_base_url);

  log::debug!("API URL request: {}", api_url);

  http::post_json(&client, &api_url, shasta_token, bos_template).await
}
