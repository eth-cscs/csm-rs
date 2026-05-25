//! BOS session templates v2 — `ShastaClient` methods for
//! `/bos/v2/sessiontemplates`.

pub mod types;

use crate::{
  ShastaClient, bos::template::http_client::v2::types::BosSessionTemplate,
  common::http, error::Error,
};

impl ShastaClient {
  /// Get BOS session templates. Ref: <https://apidocs.svc.cscs.ch/paas/bos/operation/get_v1_sessiontemplates/>.
  pub async fn bos_template_v2_get(
    &self,
    bos_session_template_id_opt: Option<&str>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    log::info!("Get BOS sessiontemplate {:?}", bos_session_template_id_opt);

    let api_url = if let Some(id) = bos_session_template_id_opt {
      format!("{}/bos/v2/sessiontemplates/{}", self.base_url(), id)
    } else {
      format!("{}/bos/v2/sessiontemplates", self.base_url())
    };

    if bos_session_template_id_opt.is_none() {
      http::get_json(self.http(), &api_url, self.token()).await
    } else {
      let single: BosSessionTemplate =
        http::get_json(self.http(), &api_url, self.token()).await?;
      Ok(vec![single])
    }
  }

  pub async fn bos_template_v2_get_all(
    &self,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    self.bos_template_v2_get(None).await
  }

  pub async fn bos_template_v2_put(
    &self,
    bos_template: &BosSessionTemplate,
    bos_template_name: &str,
  ) -> Result<BosSessionTemplate, Error> {
    log::info!("Create BOS sessiontemplte '{}'", bos_template_name);
    log::debug!(
      "Create BOS sessiontemplate request payload:\n{}",
      serde_json::to_string_pretty(bos_template)
        .unwrap_or_else(|e| format!("<serialize error: {}>", e))
    );

    let api_url = format!(
      "{}/bos/v2/sessiontemplates/{}",
      self.base_url(),
      bos_template_name
    );
    http::put_json(self.http(), &api_url, self.token(), bos_template).await
  }

  /// Delete BOS session templates.
  pub async fn bos_template_v2_delete(
    &self,
    bos_template_id: &str,
  ) -> Result<(), Error> {
    let api_url = format!(
      "{}/bos/v2/sessiontemplates/{}",
      self.base_url(),
      bos_template_id
    );

    self
      .http()
      .delete(api_url)
      .bearer_auth(self.token())
      .send()
      .await
      .map_err(Error::NetError)?
      .error_for_status()
      .map_err(Error::NetError)?;

    Ok(())
  }
}
