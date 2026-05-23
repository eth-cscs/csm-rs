pub mod types;

use crate::{
  bos::template::http_client::v2::types::BosSessionTemplate,
  common::http,
  error::Error,
};

/// Get BOS session templates. Ref --> https://apidocs.svc.cscs.ch/paas/bos/operation/get_v1_sessiontemplates/
pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  bos_session_template_id_opt: Option<&str>,
) -> Result<Vec<BosSessionTemplate>, Error> {
  log::info!("Get BOS sessiontemplate {:?}", bos_session_template_id_opt);

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;

  let api_url = if let Some(id) = bos_session_template_id_opt {
    format!("{}/bos/v2/sessiontemplates/{}", shasta_base_url, id)
  } else {
    format!("{}/bos/v2/sessiontemplates", shasta_base_url)
  };

  if bos_session_template_id_opt.is_none() {
    http::get_json(&client, &api_url, shasta_token).await
  } else {
    let single: BosSessionTemplate =
      http::get_json(&client, &api_url, shasta_token).await?;
    Ok(vec![single])
  }
}

pub async fn get_all(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Vec<BosSessionTemplate>, Error> {
  get(shasta_token, shasta_base_url, shasta_root_cert, socks5_proxy, None).await
}

pub async fn put(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  bos_template: &BosSessionTemplate,
  bos_template_name: &str,
) -> Result<BosSessionTemplate, Error> {
  log::info!("Create BOS sessiontemplte '{}'", bos_template_name);
  log::debug!(
    "Create BOS sessiontemplate request payload:\n{}",
    serde_json::to_string_pretty(bos_template).unwrap()
  );

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!(
    "{}/bos/v2/sessiontemplates/{}",
    shasta_base_url, bos_template_name
  );
  http::put_json(&client, &api_url, shasta_token, bos_template).await
}

/// Delete BOS session templates.
pub async fn delete(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  bos_template_id: &str,
) -> Result<(), Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url =
    format!("{}/bos/v2/sessiontemplates/{}", shasta_base_url, bos_template_id);

  // NOTE: existing behavior — `error_for_status` is called but its result is
  // discarded, so DELETE failures are silently ignored. Preserving for now.
  let _ = client
    .delete(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await?
    .error_for_status();

  Ok(())
}
