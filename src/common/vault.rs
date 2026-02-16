pub mod http_client {

  use crate::error::Error;
  use serde_json::{Value, json};

  pub async fn auth_oidc_jwt(
    vault_base_url: &str,
    // vault_role_id: &str,
    shasta_token: &str,
    site_name: &str,
  ) -> Result<String, Error> {
    let role = "manta";

    let client;

    let client_builder = reqwest::Client::builder();

    // Build client
    if std::env::var("SOCKS5").is_ok() {
      // socks5 proxy
      let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5")?)?;
      client = client_builder.proxy(socks5proxy).build()?;
    } else {
      client = client_builder.build()?;
    }

    let api_url =
      format!("{}/v1/auth/jwt-manta-{}/login", vault_base_url, site_name);

    log::debug!("Accessing/login to {}", api_url);

    let request_payload = json!({
            "jwt": shasta_token,
            "role": role});

    let resp = client
      .post(api_url)
      .header("X-Vault-Request", "true")
      .json(&request_payload)
      .send()
      .await?;

    match resp.error_for_status() {
      Ok(resp) => {
        let resp_value = resp.json::<Value>().await?;
        return Ok(
          resp_value
            .get("auth")
            .and_then(|auth| auth.get("client_token"))
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| {
              Error::Message("ERROR - JWT auth token not valid".to_string())
            })?,
        );
      }
      Err(e) => {
        return Err(Error::NetError(e));
      }
    }
  }

  pub async fn fetch_secret(
    vault_auth_token: &str,
    vault_base_url: &str,
    secret_path: &str,
  ) -> Result<Value, Error> {
    let client;

    let client_builder = reqwest::Client::builder();

    // Build client
    if std::env::var("SOCKS5").is_ok() {
      // socks5 proxy
      let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5")?)?;
      client = client_builder.proxy(socks5proxy).build()?;
    } else {
      client = client_builder.build()?;
    }

    let api_url = vault_base_url.to_owned() + secret_path;

    log::debug!("Vault url to fetch VCS secrets is '{}'", api_url);

    let resp = client
      .get(api_url)
      .header("X-Vault-Token", vault_auth_token)
      .send()
      .await?;

    match resp.error_for_status() {
      Ok(resp) => {
        let secret_value: Value = resp.json().await?;
        return Ok(secret_value["data"].clone());
      }
      Err(e) => {
        return Err(Error::NetError(e));
      }
    }
  }

  pub async fn fetch_shasta_vcs_token(
    shasta_token: &str,
    vault_base_url: &str,
    site_name: &str,
    // vault_role_id: &str,
    // secret_path: &str,
  ) -> Result<String, Error> {
    let vault_token =
      auth_oidc_jwt(vault_base_url, shasta_token, site_name).await?;

    let vault_secret_path = format!("manta/data/{}", site_name);

    let vault_secret = fetch_secret(
      &vault_token,
      vault_base_url,
      &format!("/v1/{}/vcs", vault_secret_path),
    )
    .await?; // this works for hashicorp-vault for fulen may need /v1/secret/data/shasta/vcs

    Ok(
      vault_secret
        .get("data")
        .and_then(|data| data.get("token"))
        .and_then(Value::as_str)
        .map(String::from)
        .ok_or_else(|| {
          Error::Message(
            "ERROR - VCS token not found in vault secret".to_string(),
          )
        })?,
    ) // this works for vault v1.12.0 for older versions may need vault_secret["data"]["token"]
  }

  pub async fn fetch_shasta_k8s_secrets_from_vault(
    vault_base_url: &str,
    // vault_role_id: &str,
    shasta_token: &str,
    // secret_path: &str,
    site_name: &str,
  ) -> Result<Value, Error> {
    log::info!("Fetching k8s secrets from vault");
    let vault_token =
      auth_oidc_jwt(vault_base_url, shasta_token, site_name).await?;

    let vault_secret_path = format!("manta/data/{}", site_name);

    fetch_secret(
      &vault_token,
      vault_base_url,
      &format!("/v1/{}/k8s", vault_secret_path),
    )
    .await
    .map(|secret| secret["data"].clone())
  }
}
