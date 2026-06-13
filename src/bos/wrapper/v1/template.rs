//! Wrapper for `/bos/v1/sessiontemplate`. Replaces
//! `src/bos/template/http_client/v1/mod.rs`.
//!
//! The upstream BOS spec is v2-only — there is no progenitor coverage
//! for v1, so this is a pure file relocation. All methods stay on raw
//! `reqwest`.
//!
//! Methods present:
//! - `bos_template_v1_get`
//! - `bos_template_v1_post`
//!
//! The hand-written wire-format types still live at
//! `crate::bos::template::http_client::v1::types` (kept where they
//! were to avoid churning consumers).

use crate::{
  ShastaClient, bos::template::http_client::v1::types::BosSessionTemplate,
  common::http, error::Error,
};

impl ShastaClient {
  /// Get BOS session templates. Ref: <https://apidocs.svc.cscs.ch/paas/bos/operation/get_v1_sessiontemplates/>.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bos_template_v1_get(
    &self,
    token: &str,
    bos_session_template_id_opt: Option<&String>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    log::debug!(
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
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bos_template_v1_post(
    &self,
    token: &str,
    bos_template: &BosSessionTemplate,
  ) -> Result<String, Error> {
    log::debug!("Create BOS sessiontemplate '{}'", bos_template.name);
    log::debug!(
      "Create BOS sessiontemplate request payload:\n{}",
      serde_json::to_string_pretty(bos_template)
        .unwrap_or_else(|e| format!("<serialize error: {e}>"))
    );

    let api_url = format!("{}/bos/v1/sessiontemplate", self.base_url());

    log::debug!("API URL request: {api_url}");

    http::post_json(self.http(), &api_url, token, bos_template).await
  }
}
