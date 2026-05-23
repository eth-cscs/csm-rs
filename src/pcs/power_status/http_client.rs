use serde_json::json;

use crate::{common::http, error::Error};

use super::types::PowerStatusAll;

pub async fn post(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname_vec_opt: Option<&[&str]>,
  power_state_filter_opt: Option<&str>,
  management_state_filter_opt: Option<&str>,
) -> Result<PowerStatusAll, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url = format!("{}/power-control/v1/power-status", shasta_base_url);

  let body = json!({
    "xname": xname_vec_opt
      .map(|v| v.iter().map(|&x| x.to_string()).collect::<Vec<String>>())
      .unwrap_or_default(),
    "powerStateFilter": power_state_filter_opt.unwrap_or(""),
    "managementStateFilter": management_state_filter_opt.unwrap_or(""),
  });

  http::post_json(&client, &url, shasta_token, &body).await
}
