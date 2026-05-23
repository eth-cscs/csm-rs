use serde_json::{Value, json};

use crate::{common::http, error::Error};

pub async fn post(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  bos_template_name: &String,
  operation: &str,
) -> core::result::Result<Value, Error> {
  let payload = json!({
      "operation": operation,
      "templateName": bos_template_name,
  });

  log::info!("Create BOS session v1");
  log::debug!("Create BOS session v1 payload:\n{:#?}", payload);

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url = format!("{}/bos/v1/session", shasta_base_url);
  http::post_json(&client, &url, shasta_token, &payload).await
}
