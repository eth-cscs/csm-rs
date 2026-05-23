use serde_json::Value;

use crate::{common::http, error::Error};

/// Create GET HTTP request and returns its response
/// This function will create an http client and call CSM API endpoint "api_url"
/// Returns the same payload received from CSM API
pub async fn process_get_http_request(
  shasta_token: &str,
  api_url: String,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Value, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  http::get_json(&client, &api_url, shasta_token).await
}
