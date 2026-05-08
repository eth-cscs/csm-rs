use serde_json::{Value, json};

use crate::error::Error;

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
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url = format!("{}/power-control/v1/power-status", shasta_base_url);

  let body = json!({
      "xname": xname_vec_opt.map(|xname_vec| xname_vec.iter().map(|&x| x.to_string()).collect::<Vec<String>>()).unwrap_or_default(),
      "powerStateFilter": power_state_filter_opt.unwrap_or(""),
      "managementStateFilter": management_state_filter_opt.unwrap_or(""),
  });

  let response = client
    .post(&api_url)
    .json(&body)
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
