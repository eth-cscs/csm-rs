use serde_json::{Value, json};

use crate::error::Error;

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

  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url = format!("{}{}", shasta_base_url, "/bos/v1/session");

  let response = client
    .post(api_url)
    .json(&payload)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(|error| Error::NetError(error))?;

  if response.status().is_success() {
    response
      .json()
      .await
      .map_err(|error| Error::NetError(error))
  } else {
    let payload = response
      .json::<Value>()
      .await
      .map_err(|error| Error::NetError(error))?;

    Err(Error::CsmError(payload))
  }
}
