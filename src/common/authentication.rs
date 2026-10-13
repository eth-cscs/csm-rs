//! Keycloak / OIDC bearer-token acquisition for Shasta.

use serde_json::Value;

use std::collections::HashMap;

use crate::error::Error;

/// Validate a CSM bearer token by issuing `GET /cfs/healthz` and
/// checking the response status.
pub async fn validate_api_token(
  shasta_base_url: &str,
  shasta_token: &str,
  shasta_root_cert: &[u8],
) -> Result<(), Error> {
  let client = crate::common::http::build_client(shasta_root_cert)?;

  let api_url = shasta_base_url.to_owned() + "/cfs/healthz";

  log::debug!("Validate CSM token against {api_url}");

  let resp_rslt = client.get(api_url).bearer_auth(shasta_token).send().await;

  match resp_rslt {
    Ok(resp) => Ok(resp.error_for_status().map(|_| ())?),
    Err(error) => Err(Error::Message(format!("Token is not valid: {error}"))),
  }
}

/// Exchange Keycloak username/password credentials for a CSM bearer
/// token via the `password` grant.
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

  let client = crate::common::http::build_client(shasta_root_cert)?;

  let api_url =
    format!("{keycloak_base_url}/realms/shasta/protocol/openid-connect/token");

  log::debug!("Request to fetch authentication token: {api_url}");

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
    .ok_or_else(|| {
      Error::Message(
        "Keycloak token response is missing 'access_token'".to_string(),
      )
    })
}

/// Per-request deadline for the token exchange. Every manta request
/// waits on it, so it is much shorter than the CSM API's
/// `HTTP_REQUEST_TIMEOUT`.
const TOKEN_EXCHANGE_TIMEOUT: std::time::Duration =
  std::time::Duration::from_secs(30);

/// Client credentials for exchanging a token from an external identity
/// provider (the CSCS Keycloak) for a CSM token at the site's Keycloak
/// (`shasta` realm), using RFC 8693 token exchange.
#[derive(Clone)]
pub struct TokenExchange {
  /// Keycloak client allowed to exchange tokens, e.g.
  /// `cscs-token-exchanger`.
  pub client_id: String,
  /// Secret of the confidential `client_id`.
  pub client_secret: String,
  /// Alias of the external identity provider in the `shasta` realm,
  /// e.g. `cscs`. Sent as `subject_issuer`.
  pub subject_issuer: String,
}

impl std::fmt::Debug for TokenExchange {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TokenExchange")
      .field("client_id", &self.client_id)
      .field("client_secret", &"<redacted>")
      .field("subject_issuer", &self.subject_issuer)
      .finish()
  }
}

/// Exchange `subject_token` (issued by the external identity provider
/// named in `exchange.subject_issuer`) for a CSM access token at
/// `{keycloak_base_url}/realms/shasta`.
///
/// # Errors
///
/// - [`Error::Unauthorized`] when Keycloak rejects the subject token
///   (`invalid_token`: expired, bad signature, unknown issuer).
/// - [`Error::Forbidden`] when Keycloak answers 403 (the exchange is
///   not permitted for this user or client).
/// - [`Error::Message`] for any other 4xx, including a wrong client
///   secret (`invalid_client`), or a success body without
///   `access_token`.
/// - [`Error::CsmText`] for a 5xx and [`Error::NetError`] when Keycloak
///   is unreachable or times out.
pub async fn exchange_token(
  client: &reqwest::Client,
  keycloak_base_url: &str,
  exchange: &TokenExchange,
  subject_token: &str,
) -> Result<String, Error> {
  let api_url =
    format!("{keycloak_base_url}/realms/shasta/protocol/openid-connect/token");

  log::debug!("Exchange token at {api_url}");

  let params = [
    (
      "grant_type",
      "urn:ietf:params:oauth:grant-type:token-exchange",
    ),
    ("subject_token", subject_token),
    ("subject_issuer", exchange.subject_issuer.as_str()),
    (
      "subject_token_type",
      "urn:ietf:params:oauth:token-type:access_token",
    ),
    (
      "requested_token_type",
      "urn:ietf:params:oauth:token-type:access_token",
    ),
  ];

  let resp = client
    .post(&api_url)
    .basic_auth(&exchange.client_id, Some(&exchange.client_secret))
    .form(&params)
    .timeout(TOKEN_EXCHANGE_TIMEOUT)
    .send()
    .await?;

  let status = resp.status();
  if status.is_success() {
    return resp
      .json::<Value>()
      .await?
      .get("access_token")
      .and_then(Value::as_str)
      .map(str::to_string)
      .ok_or_else(|| {
        Error::Message(
          "Keycloak token-exchange response is missing 'access_token'"
            .to_string(),
        )
      });
  }

  let payload = resp.text().await.unwrap_or_default();
  if status.is_server_error() {
    return Err(Error::CsmText {
      method: "POST".to_string(),
      url: api_url,
      status: status.as_u16(),
      payload,
    });
  }

  // OAuth error body: {"error": "...", "error_description": "..."}.
  let body: Value = serde_json::from_str(&payload).unwrap_or(Value::Null);
  let error = body.get("error").and_then(Value::as_str).unwrap_or("");
  let description = body
    .get("error_description")
    .and_then(Value::as_str)
    .unwrap_or(&payload);
  let detail = format!(
    "token exchange at {api_url} failed: status={status} \
     error={error} {description}"
  );

  Err(match (status.as_u16(), error) {
    (_, "invalid_token") => Error::Unauthorized(detail),
    (403, _) => Error::Forbidden(detail),
    _ => Error::Message(detail),
  })
}
