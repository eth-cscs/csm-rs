use serde_json::Value;

use std::collections::HashMap;

use crate::error::Error;

pub async fn validate_api_token(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
) -> Result<(), Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  // Build client
  let client = if std::env::var("SOCKS5").is_ok() {
    // socks5 proxy
    log::debug!("SOCKS5 enabled");
    let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5")?)?;
    client_builder.proxy(socks5proxy).build()?
  } else {
    client_builder.build()?
  };

  let api_url = shasta_base_url.to_owned() + "/cfs/healthz";

  log::info!("Validate CSM token against {}", api_url);

  let resp_rslt = client.get(api_url).bearer_auth(shasta_token).send().await;

  match resp_rslt {
    Ok(resp) => {
      return Ok(resp.error_for_status().map(|_| ())?);
    }
    Err(error) => Err(Error::Message(format!("Token is not valid: {}", error))),
  }
}

pub async fn get_token_from_shasta_endpoint(
  keycloak_base_url: &str,
  shasta_root_cert: &[u8],
  username: &str,
  password: &str,
) -> Result<String, Error> {
  let mut params = HashMap::new();
  params.insert("grant_type", "password");
  params.insert("client_id", "shasta");
  params.insert("username", username);
  params.insert("password", password);

  let client;

  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  // Build client
  if std::env::var("SOCKS5").is_ok() {
    // socks5 proxy
    let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5")?)?;
    client = client_builder.proxy(socks5proxy).build()?;
  } else {
    client = client_builder.build()?;
  }

  let api_url = format!(
    "{}/realms/shasta/protocol/openid-connect/token",
    keycloak_base_url
  );

  log::debug!("Request to fetch authentication token: {}", api_url);

  Ok(
    client
      .post(api_url)
      .form(&params)
      .send()
      .await?
      .error_for_status()?
      .json::<Value>()
      .await?
      .get("access_token")
      .and_then(Value::as_str)
      .map(str::to_string)
      .unwrap(),
  )
}
