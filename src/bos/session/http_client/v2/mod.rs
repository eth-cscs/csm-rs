pub mod types;

use types::BosSession;

use crate::{common::http, error::Error};

pub async fn post(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  bos_session: BosSession,
) -> Result<BosSession, Error> {
  log::info!(
    "Create BOS session '{}'",
    bos_session.name.as_deref().unwrap_or("unknown")
  );
  log::debug!("Create BOS session request:\n{:#?}", bos_session);

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!("{}/bos/v2/sessions", shasta_base_url);
  let created: BosSession =
    http::post_json(&client, &api_url, shasta_token, &bos_session).await?;

  log::info!(
    "BOS session '{}' created successfully",
    created.name.as_deref().unwrap_or("unknown")
  );
  Ok(created)
}

pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  id_opt: Option<&str>,
) -> Result<Vec<BosSession>, Error> {
  log::info!("Get BOS sessions '{}'", id_opt.unwrap_or("all available"));

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;

  let api_url = if let Some(id) = id_opt {
    format!("{}/bos/v2/sessions/{}", shasta_base_url, id)
  } else {
    format!("{}/bos/v2/sessions", shasta_base_url)
  };

  if id_opt.is_some() {
    let single: BosSession =
      http::get_json(&client, &api_url, shasta_token).await?;
    Ok(vec![single])
  } else {
    http::get_json(&client, &api_url, shasta_token).await
  }
}

pub async fn delete(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  bos_session_id: &str,
) -> Result<(), Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url =
    format!("{}/bos/v2/sessions/{}", shasta_base_url, bos_session_id);
  http::delete(&client, &api_url, shasta_token).await
}
