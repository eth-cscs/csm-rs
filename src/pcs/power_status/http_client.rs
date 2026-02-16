use serde_json::{Value, json};

use crate::error::Error;

use super::types::PowerStatusAll;

pub async fn post(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
  xname_vec_opt: Option<&[&str]>,
  power_state_filter_opt: Option<&str>,
  management_state_filter_opt: Option<&str>,
) -> Result<PowerStatusAll, Error> {
  let client;

  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  // Build client
  if std::env::var("SOCKS5").is_ok() {
    // socks5 proxy
    log::debug!("SOCKS5 enabled");
    let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5")?)?;

    // rest client to authenticate
    client = client_builder.proxy(socks5proxy).build()?;
  } else {
    client = client_builder.build()?;
  }

  let api_url = format!("{}/power-control/v1/power-status", shasta_base_url);

  let body = json!({
      // TODO: this should probably be a struct that matches the expected schema
      // here "xname" is an array of xnames
      "xname": xname_vec_opt.map(|xname_vec| xname_vec.iter().map(|&x| x.to_string()).collect::<Vec<String>>()).unwrap_or_default(),
      "powerStateFilter": power_state_filter_opt.unwrap_or(""),
      "managementStateFilter": management_state_filter_opt.unwrap_or(""),
  });

  let response = client
    .post(&api_url)
    .json(&body) // Send the body as JSON
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(|error| {
      println!("Failed POST query: {:?}", error);
      Error::NetError(error)
    })?;

  if response.status().is_success() {
    println!("Response is success");
    response.json().await.map_err(|error| {
      println!("{:?}", error);
      Error::NetError(error)
    })
  } else {
    println!("Response is failure");
    let payload = response.json::<Value>().await.map_err(|error| {
      println!("{:?}", error);
      Error::NetError(error)
    })?;

    Err(Error::CsmError(payload))
  }
}
