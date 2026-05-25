//! BOS sessions v1 — `ShastaClient` methods for `/bos/v1/session`.

use serde_json::{Value, json};

use crate::{ShastaClient, common::http, error::Error};

impl ShastaClient {
  pub async fn bos_session_v1_post(
    &self,
    token: &str,
    bos_template_name: &str,
    operation: &str,
  ) -> Result<Value, Error> {
    let payload = json!({
      "operation": operation,
      "templateName": bos_template_name,
    });

    log::info!("Create BOS session v1");
    log::debug!("Create BOS session v1 payload:\n{:#?}", payload);

    let url = format!("{}/bos/v1/session", self.base_url());
    http::post_json(self.http(), &url, token, &payload).await
  }
}
