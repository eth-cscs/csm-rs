use serde_json::Value;

use crate::error::Error;

/// Create GET HTTP request and returns its response
/// This function will create an http client and call CSM API endpoint "api_url"
/// Returns the same payload received from CSM API
pub async fn process_get_http_request(
  shasta_token: &str,
  api_url: String,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Value, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  // Build client
  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  // Call to CSM API
  let response = client
    .get(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(|e| Error::NetError(e))?; // Map network errors

  // Error handleling. Check for errors from the CSM API processing the request
  match response.status().is_success() {
    true => response.json().await.map_err(|e| Error::NetError(e)), // Map error during marshalling
    false => {
      let e: Value = response.json().await.map_err(|e| Error::NetError(e))?; // Map error during

      Err(Error::CsmError(e))
    }
  }
}
