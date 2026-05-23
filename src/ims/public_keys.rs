pub mod http_client {

  pub mod v3 {
    use serde_json::Value;

    use crate::{common::http, error::Error};

    /// Get one user public key in IMS is can find
    /// Returns None if public key not found or multiple fould
    pub async fn get_single(
      shasta_token: &str,
      shasta_base_url: &str,
      shasta_root_cert: &[u8],
      socks5_proxy: Option<&str>,
      username_opt: &str,
    ) -> Result<Option<Value>, Error> {
      let public_key_value_list = get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        socks5_proxy,
        Some(username_opt),
      )
      .await?;

      if public_key_value_list.len() == 1 {
        Ok(public_key_value_list.first().cloned())
      } else {
        Ok(None)
      }
    }

    /// Fetch IMS image ref --> https://apidocs.svc.cscs.ch/paas/ims/operation/get_v3_image/
    pub async fn get(
      shasta_token: &str,
      shasta_base_url: &str,
      shasta_root_cert: &[u8],
      socks5_proxy: Option<&str>,
      username_opt: Option<&str>,
    ) -> Result<Vec<Value>, Error> {
      let client = http::build_client(shasta_root_cert, socks5_proxy)?;
      let api_url = format!("{}/ims/v3/public-keys", shasta_base_url);

      let json_response: Value =
        http::get_json(&client, &api_url, shasta_token).await?;

      let public_key_value_list = json_response.as_array().unwrap().to_vec();

      Ok(match username_opt {
        Some(username) => public_key_value_list
          .into_iter()
          .filter(|ssh_key_value| {
            ssh_key_value
              .get("name")
              .and_then(Value::as_str)
              .is_some_and(|v| v.eq(username))
          })
          .collect(),
        None => public_key_value_list,
      })
    }
  }
}
