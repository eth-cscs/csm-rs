use serde_json::Value;

use crate::{ShastaClient, common::http, error::Error};

impl ShastaClient {
  /// Get one user public key in IMS.
  /// Returns None if public key not found or multiple fould
  pub async fn ims_public_keys_v3_get_single(
    &self,
    username_opt: &str,
  ) -> Result<Option<Value>, Error> {
    let public_key_value_list =
      self.ims_public_keys_v3_get(Some(username_opt)).await?;

    if public_key_value_list.len() == 1 {
      Ok(public_key_value_list.first().cloned())
    } else {
      Ok(None)
    }
  }

  /// Fetch IMS image ref --> https://apidocs.svc.cscs.ch/paas/ims/operation/get_v3_image/
  pub async fn ims_public_keys_v3_get(
    &self,
    username_opt: Option<&str>,
  ) -> Result<Vec<Value>, Error> {
    let api_url = format!("{}/ims/v3/public-keys", self.base_url());

    let json_response: Value =
      http::get_json(self.http(), &api_url, self.token()).await?;

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
