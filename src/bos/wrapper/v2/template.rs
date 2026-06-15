//! Wrapper for `/bos/v2/sessiontemplates`. Replaces
//! `src/bos/template/http_client/v2/mod.rs`.
//!
//! Routed through the progenitor-generated client:
//! - *(none)* — every generated v2 sessiontemplate method returns the
//!   strict `types::V2SessionTemplate{,Array}` shapes. The generated
//!   `V2SessionTemplate` has `boot_sets: HashMap<String, V2BootSet>`
//!   (not `Option`), `name: Option<V2SessionTemplateName>` (regex-
//!   validated newtype), `tenant: Option<V2TenantName>` (regex newtype),
//!   `description: Option<SessionTemplateDescription>` (newtype),
//!   `enable_cfs: Option<EnableCfs>` (bool newtype), `cfs:
//!   Option<V2CfsParameters>`, and `links: Option<LinkListReadOnly>`.
//!   The generated `V2BootSet` further requires `path:
//!   BootManifestPath` and `type_: BootSetType` (both non-`Option`
//!   newtypes) and uses the `V2BootSetArch` enum for `arch`. csm-rs's
//!   public [`BosSessionTemplate`] is the looser hand-written shape
//!   (plain `Option<String>` for every name/tenant/description/path/
//!   type/arch, `Option<HashMap<String, BootSet>>` for `boot_sets`) and
//!   is re-exported at `crate::bos::BosSessionTemplate`, consumed by
//!   `backend_connector::bos`, `cfs::configuration::utils`,
//!   `ims::image::utils`, `commands::migrate_backup`,
//!   `bos::template::utils`, the `dispatcher_conv.rs` `From` impls, and
//!   the `manta-backend-dispatcher` trait impls. Adopting the generated
//!   types here would force a structural change across all those
//!   consumers (and the public `bos::BosSessionTemplate` API) so this
//!   wave keeps everything on raw `reqwest`. A follow-up commit can
//!   migrate individual methods once a generated->hand-written
//!   conversion layer (or a swap of `BosSessionTemplate` to the
//!   generated type) lands.
//!
//! Stays on raw `reqwest` because the generated surface doesn't cover
//! what the existing public API needs:
//!
//! - `bos_template_v2_get` — collapses two distinct generated
//!   operations into one method. With `id_opt = None` it must call
//!   `get_v2_sessiontemplates` (returns `V2SessionTemplateArray`); with
//!   `id_opt = Some(id)` it must call `get_v2_sessiontemplate` (returns
//!   a single `V2SessionTemplate`) and re-wrap as a one-element `Vec`.
//!   Both generated methods return `V2SessionTemplate{,Array}` rather
//!   than the hand-written `BosSessionTemplate`, so adopting them would
//!   change the public return type. The generated `get_v2_sessiontemplate`
//!   also takes `session_template_id: &SessionTemplateName` (regex-
//!   validated newtype around `String`); the public method takes
//!   `bos_session_template_id_opt: Option<&str>` and would have to
//!   surface a new validation error path.
//! - `bos_template_v2_get_all` — convenience shim over
//!   `bos_template_v2_get(token, None)`; it inherits the same return-type
//!   coupling as above.
//! - `bos_template_v2_put` — request body type is the public hand-
//!   written `BosSessionTemplate`; the generated `put_v2_sessiontemplate`
//!   takes `body: &types::V2SessionTemplate` and returns
//!   `types::V2SessionTemplate` (different field shape, see above), so
//!   adopting it would change the public input *and* output types. The
//!   generated method also takes `session_template_id:
//!   &SessionTemplateName` (regex newtype); the public method takes
//!   `bos_template_name: &str` and would have to surface a new
//!   validation error path.
//! - `bos_template_v2_delete` — generated `delete_v2_sessiontemplate`
//!   takes `session_template_id: &SessionTemplateName` (regex-
//!   validated newtype); the public method takes `bos_template_id:
//!   &str` and currently has no such validation. Routing through
//!   progenitor would either swallow the validation error (lossy) or
//!   introduce a new failure mode at the wrapper boundary.
//!
//! The `gen_client` / `map_err` / `run` helpers in
//! `crate::bos::wrapper` are retained so a future spec revision can be
//! migrated incrementally without a second scaffolding pass.

use crate::{
  ShastaClient, bos::template::http_client::v2::types::BosSessionTemplate,
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
  pub async fn bos_template_v2_get(
    &self,
    token: &str,
    bos_session_template_id_opt: Option<&str>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    log::debug!("Get BOS sessiontemplate {bos_session_template_id_opt:?}");

    let api_url = if let Some(id) = bos_session_template_id_opt {
      format!("{}/bos/v2/sessiontemplates/{}", self.base_url(), id)
    } else {
      format!("{}/bos/v2/sessiontemplates", self.base_url())
    };

    if bos_session_template_id_opt.is_none() {
      http::get_json(self.http(), &api_url, token).await
    } else {
      let single: BosSessionTemplate =
        http::get_json(self.http(), &api_url, token).await?;
      Ok(vec![single])
    }
  }

  /// `GET /bos/v2/sessiontemplates` — list every BOS v2 session
  /// template.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bos_template_v2_get_all(
    &self,
    token: &str,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    self.bos_template_v2_get(token, None).await
  }

  /// `PUT /bos/v2/sessiontemplates/{name}` — create or replace a BOS
  /// v2 session template.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bos_template_v2_put(
    &self,
    token: &str,
    bos_template: &BosSessionTemplate,
    bos_template_name: &str,
  ) -> Result<BosSessionTemplate, Error> {
    log::debug!("Create BOS sessiontemplte '{bos_template_name}'");
    log::debug!(
      "Create BOS sessiontemplate request payload:\n{}",
      serde_json::to_string_pretty(bos_template)
        .unwrap_or_else(|e| format!("<serialize error: {e}>"))
    );

    let api_url = format!(
      "{}/bos/v2/sessiontemplates/{}",
      self.base_url(),
      bos_template_name
    );
    http::put_json(self.http(), &api_url, token, bos_template).await
  }

  /// Delete BOS session templates.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bos_template_v2_delete(
    &self,
    token: &str,
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
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?
      .error_for_status()
      .map_err(Error::NetError)?;

    Ok(())
  }
}
