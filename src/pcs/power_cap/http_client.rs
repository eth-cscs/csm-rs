use crate::{common::http, error::Error};

use super::types::{PowerCapComponent, PowerCapTaskInfo};

pub async fn get(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<PowerCapTaskInfo, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url = format!("{}/power-control/v1/power-cap", shasta_base_url);
  http::get_json(&client, &url, shasta_token).await
}

pub async fn get_task_id(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  task_id: &str,
) -> Result<PowerCapTaskInfo, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url =
    format!("{}/power-control/v1/power-cap/{}", shasta_base_url, task_id);
  http::get_json(&client, &url, shasta_token).await
}

pub async fn post_snapshot(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname_vec: Vec<&str>,
) -> Result<PowerCapTaskInfo, Error> {
  log::info!("Create PCS power snapshot for nodes:\n{:?}", xname_vec);
  log::debug!("Create PCS power snapshot for nodes:\n{:?}", xname_vec);

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url =
    format!("{}/power-control/v1/power-cap/snapshot", shasta_base_url);
  let body = serde_json::json!({ "xnames": xname_vec });
  http::put_json(&client, &url, shasta_token, &body).await
}

pub async fn patch(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  power_cap: Vec<PowerCapComponent>,
) -> Result<PowerCapTaskInfo, Error> {
  log::info!("Create PCS power cap:\n{:#?}", power_cap);
  log::debug!("Create PCS power cap:\n{:#?}", power_cap);

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url =
    format!("{}/power-control/v1/power-cap/snapshot", shasta_base_url);
  http::put_json(&client, &url, shasta_token, &power_cap).await
}
