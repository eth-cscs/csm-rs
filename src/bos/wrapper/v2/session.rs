//! Wrapper for `/bos/v2/sessions`. Replaces
//! `src/bos/session/http_client/v2/mod.rs`.
//!
//! Routed through the progenitor-generated client:
//! - *(none)* — every generated v2 session method returns the strict
//!   `types::V2Session{,Array}` / `types::V2SessionCreate` shapes
//!   (newtype-wrapped `name: Option<V2SessionName>`, enum
//!   `operation: Option<V2SessionOperation>`, `status: Option<V2SessionStatus>`
//!   with regex-validated start/end-time newtypes). csm-rs's public
//!   [`BosSession`] is the looser hand-written shape (plain
//!   `Option<String>` for name/limit/components, free-form `Status`
//!   with stringly-typed `start_time`/`end_time`, `Operation` enum
//!   that does not derive `Copy` and is `#[non_exhaustive]`) and is
//!   re-exported at `crate::bos::BosSession`, consumed by
//!   `backend_connector::bos`, `dispatcher_conv.rs`, and the
//!   `manta-backend-dispatcher` trait impls. Adopting the generated
//!   types here would force a structural change across all those
//!   consumers (and the public `bos::BosSession` API) so this wave
//!   keeps everything on raw `reqwest`. A follow-up commit can
//!   migrate individual methods once a generated->hand-written
//!   conversion layer (or a swap of `BosSession` to the generated
//!   type) lands.
//!
//! Stays on raw `reqwest` because the generated surface doesn't
//! cover what the existing public API needs:
//!
//! - `bos_session_v2_post` — request body type is the public
//!   hand-written `BosSession`; the generated `post_v2_session`
//!   takes `&types::V2SessionCreate` and returns `types::V2Session`
//!   (different field shape, see above), so adopting it would change
//!   the public input *and* output types.
//! - `bos_session_v2_get` — collapses two distinct generated
//!   operations into one method. With `id_opt = None` it must call
//!   `get_v2_sessions` (returns `V2SessionArray`); with `id_opt =
//!   Some(id)` it must call `get_v2_session` (returns a single
//!   `V2Session`) and re-wrap as a one-element `Vec`. Both generated
//!   methods return `V2Session{,Array}` rather than the hand-written
//!   `BosSession`, so adopting them would change the public return
//!   type. The generated `get_v2_session` also takes `session_id:
//!   &V2SessionName` (regex-validated newtype around `String`); the
//!   public method takes `id_opt: Option<&str>` and would have to
//!   surface a new validation error path.
//! - `bos_session_v2_delete` — generated `delete_v2_session` takes
//!   `session_id: &V2SessionName` (regex-validated newtype); the
//!   public method takes `bos_session_id: &str` and currently has no
//!   such validation. Routing through progenitor would either swallow
//!   the validation error (lossy) or introduce a new failure mode at
//!   the wrapper boundary.
//!
//! The `gen_client` / `map_err` / `run` helpers in
//! `crate::bos::wrapper` are retained so a future spec revision can
//! be migrated incrementally without a second scaffolding pass.

use crate::{
  ShastaClient,
  bos::session::http_client::v2::types::BosSession,
  common::http,
  error::Error,
};

impl ShastaClient {
  /// `POST /bos/v2/sessions` — create a BOS session.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bos_session_v2_post(
    &self,
    token: &str,
    bos_session: BosSession,
  ) -> Result<BosSession, Error> {
    log::debug!(
      "Create BOS session '{}'",
      bos_session.name.as_deref().unwrap_or("unknown")
    );
    log::debug!("Create BOS session request:\n{bos_session:#?}");

    let api_url = format!("{}/bos/v2/sessions", self.base_url());
    let created: BosSession =
      http::post_json(self.http(), &api_url, token, &bos_session).await?;

    log::debug!(
      "BOS session '{}' created successfully",
      created.name.as_deref().unwrap_or("unknown")
    );
    Ok(created)
  }

  /// `GET /bos/v2/sessions` (or `/bos/v2/sessions/{id}` if `id_opt` is
  /// supplied) — list sessions or fetch one by ID.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bos_session_v2_get(
    &self,
    token: &str,
    id_opt: Option<&str>,
  ) -> Result<Vec<BosSession>, Error> {
    log::debug!("Get BOS sessions '{}'", id_opt.unwrap_or("all available"));

    let api_url = if let Some(id) = id_opt {
      format!("{}/bos/v2/sessions/{}", self.base_url(), id)
    } else {
      format!("{}/bos/v2/sessions", self.base_url())
    };

    if id_opt.is_some() {
      let single: BosSession =
        http::get_json(self.http(), &api_url, token).await?;
      Ok(vec![single])
    } else {
      http::get_json(self.http(), &api_url, token).await
    }
  }

  /// `DELETE /bos/v2/sessions/{id}` — delete a BOS session.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bos_session_v2_delete(
    &self,
    token: &str,
    bos_session_id: &str,
  ) -> Result<(), Error> {
    let api_url =
      format!("{}/bos/v2/sessions/{}", self.base_url(), bos_session_id);
    http::delete(self.http(), &api_url, token).await
  }
}
