//! BOS session templates v1 — `ShastaClient` methods for
//! `/bos/v1/sessiontemplate`.

pub(crate) mod types;

use crate::{
  ShastaClient, bos::template::http_client::v1::types::BosSessionTemplate,
  common::http, error::Error,
};

impl ShastaClient {
  /// Get BOS session templates. Ref: <https://apidocs.svc.cscs.ch/paas/bos/operation/get_v1_sessiontemplates/>.
  pub async fn bos_template_v1_get(
    &self,
    token: &str,
    bos_session_template_id_opt: Option<&String>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    log::info!(
      "Get BOS sessiontemplates '{}'",
      bos_session_template_id_opt.unwrap_or(&"all available".to_string())
    );

    let api_url = if let Some(id) = bos_session_template_id_opt {
      format!("{}/bos/v1/sessiontemplate/{}", self.base_url(), id)
    } else {
      format!("{}/bos/v1/sessiontemplate", self.base_url())
    };

    if bos_session_template_id_opt.is_none() {
      http::get_json(self.http(), &api_url, token).await
    } else {
      let single: BosSessionTemplate =
        http::get_json(self.http(), &api_url, token).await?;
      Ok(vec![single])
    }
  }

  /// `POST /bos/v1/sessiontemplate` — create a v1 BOS session
  /// template.
  pub async fn bos_template_v1_post(
    &self,
    token: &str,
    bos_template: &BosSessionTemplate,
  ) -> Result<String, Error> {
    log::info!("Create BOS sessiontemplate '{}'", bos_template.name);
    log::debug!(
      "Create BOS sessiontemplate request payload:\n{}",
      serde_json::to_string_pretty(bos_template)
        .unwrap_or_else(|e| format!("<serialize error: {}>", e))
    );

    let api_url = format!("{}/bos/v1/sessiontemplate", self.base_url());

    log::debug!("API URL request: {}", api_url);

    http::post_json(self.http(), &api_url, token, bos_template).await
  }
}
