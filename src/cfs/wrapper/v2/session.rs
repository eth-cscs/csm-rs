//! Wrapper for `/cfs/v2/sessions`. Replaces
//! `src/cfs/session/http_client/v2/mod.rs`.
//!
//! Routed through the progenitor-generated client:
//! - *(none)* — every generated v2 session method exchanges the strict
//!   `types::V2Session{,Array}` / `types::V2SessionCreate` shape:
//!     * `V2Session.name: Option<V2SessionName>` (newtype with the
//!       45-char/`^[a-z0-9]...$` pattern baked in) — csm-rs's
//!       hand-written `CfsSessionGetResponse.name` is `String`,
//!     * `V2Session.tags: HashMap<String, String>` (non-optional,
//!       defaulted) — hand-written is `Option<HashMap<String, String>>`,
//!     * `V2SessionCreate.name: V2SessionCreateName` (newtype),
//!       `ansible_limit: Option<V2SessionCreateAnsibleLimit>` (newtype
//!       with `^[^\s;]*$` pattern), `ansible_verbosity: i64`
//!       (non-optional, defaulted) — hand-written
//!       `CfsSessionPostRequest` is `name: String`,
//!       `ansible_limit: Option<String>`, `ansible_verbosity: Option<u8>`,
//!     * `V2SessionCreate.target: Option<SessionTargetSection>` and
//!       `V2Session.target: Option<SessionTargetSection>` — both use the
//!       generated `SessionTargetSection` shape; hand-written
//!       `CfsSessionPostRequest.target` is the local `Target` struct
//!       (`definition: Option<String>`, `groups: Option<Vec<Group>>`)
//!       and `CfsSessionGetResponse.target` is the same local `Target`.
//!   csm-rs's public `CfsSessionGetResponse` / `CfsSessionPostRequest`
//!   are re-exported from `cfs::v2`, consumed by `dispatcher_conv.rs`
//!   (`From` impls between hand-written types and the dispatcher
//!   mirrors), the SAT-file session translation in
//!   `cfs::session::utils`, `cleanup_session.rs`, and the
//!   `manta-backend-dispatcher` trait impls. Adopting the generated
//!   types here would force a structural change across all those
//!   consumers (and the public `cfs::v2::CfsSession*` API) so this
//!   wave keeps everything on raw `reqwest`. A follow-up commit can
//!   migrate individual methods once a generated->hand-written
//!   conversion layer (or a swap of the public types to the generated
//!   ones) lands.
//!
//! Stays on raw `reqwest` because the generated surface doesn't
//! cover what the existing public API needs:
//!
//! - `cfs_session_v2_get` multiplexes `GET /cfs/v2/sessions` and
//!   `GET /cfs/v2/sessions/{name}` and wraps the single-name response
//!   in a one-element `Vec` for uniform caller handling. The generated
//!   client splits these into `get_sessions_v2` (returns
//!   `V2SessionArray`) and `get_session_v2` (returns `V2Session`).
//!   The list variant accepts typed `GetSessionsV2Status` /
//!   `GetSessionsV2Succeeded` enums while the hand-written signature
//!   takes `status: Option<&String>` and `is_succeded_opt: Option<bool>`,
//!   so adopting the generated client would require fallible
//!   string-to-enum conversion. The hand-written code also sends the
//!   `succeded` query parameter (mis-spelled to match a long-standing
//!   CSM-side typo); the generated client sends the spec-canonical
//!   `succeeded`. Swapping spellings is a behavioural change that
//!   needs its own evaluation rather than riding along with codegen
//!   adoption.
//! - `cfs_session_v2_get_all` is a convenience wrapper over
//!   `cfs_session_v2_get` with every filter cleared, not an endpoint
//!   binding of its own.
//! - `cfs_session_v2_post` takes `&CfsSessionPostRequest` and returns
//!   `CfsSessionGetResponse`; the generated `create_session_v2` takes
//!   `&V2SessionCreate` (newtype-validated `name` / `ansible_limit`,
//!   non-optional `ansible_verbosity: i64`, generated
//!   `SessionTargetSection`) and returns `V2Session` (different
//!   `name`, `tags`, and `target` shapes — see the "routed via
//!   progenitor" section above).
//! - `cfs_session_v2_delete` returns `()` and matches the generated
//!   `delete_session_v2` signature on its own. We still keep it on raw
//!   `reqwest` for now to avoid leaving a single progenitor-routed
//!   method dangling in a module whose other three methods are blocked
//!   on the contract mismatches above; routing just `delete` would
//!   force the wrapper to mix two error/transport paths for no
//!   behavioural gain. Migrating delete on its own is safe to revisit
//!   alongside the future swap of the public session types.

use crate::{ShastaClient, common::http, error::Error};

use crate::cfs::session::http_client::v2::types::{
  CfsSessionGetResponse, CfsSessionPostRequest,
};

impl ShastaClient {
  /// Fetch CFS sessions, optionally by name or filtered by age / status
  /// / success.
  ///
  /// `GET /cfs/v2/sessions[/{name}]`. When `session_name_opt` is set,
  /// the returned `Vec` always has at most one element. The age filters
  /// accept the CSM duration syntax (e.g. `"24h"`, `"7d"`).
  ///
  /// See <https://apidocs.svc.cscs.ch/paas/cfs/operation/get_sessions/>
  /// for the underlying REST contract.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_session_v2_get(
    &self,
    token: &str,
    min_age_opt: Option<&String>,
    max_age_opt: Option<&String>,
    status_opt: Option<&String>,
    session_name_opt: Option<&String>,
    is_succeded_opt: Option<bool>,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    log::debug!(
      "Get CFS sessions '{}'",
      session_name_opt.unwrap_or(&"all available".to_string())
    );

    let api_url = if let Some(session_name) = session_name_opt {
      format!("{}/cfs/v2/sessions/{}", self.base_url(), session_name)
    } else {
      format!("{}/cfs/v2/sessions", self.base_url())
    };

    let mut query_params: Vec<(&str, String)> = Vec::new();
    if let Some(is_succeded) = is_succeded_opt {
      query_params.push(("succeced", is_succeded.to_string()));
    }
    if let Some(min_age) = min_age_opt {
      query_params.push(("min_age", min_age.clone()));
    }
    if let Some(max_age) = max_age_opt {
      query_params.push(("max_age", max_age.clone()));
    }
    if let Some(status) = status_opt {
      query_params.push(("status", status.clone()));
    }

    if session_name_opt.is_some() {
      let payload: CfsSessionGetResponse =
        http::get_json_with_query(self.http(), &api_url, token, &query_params)
          .await?;
      Ok(vec![payload])
    } else {
      http::get_json_with_query(self.http(), &api_url, token, &query_params)
        .await
    }
  }

  /// List every CFS session.
  ///
  /// Convenience wrapper for `cfs_session_v2_get` with all filters
  /// unset.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_session_v2_get_all(
    &self,
    token: &str,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    self
      .cfs_session_v2_get(token, None, None, None, None, None)
      .await
  }

  /// Create a new CFS session.
  ///
  /// `POST /cfs/v2/sessions`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_session_v2_post(
    &self,
    token: &str,
    session: &CfsSessionPostRequest,
  ) -> Result<CfsSessionGetResponse, Error> {
    log::debug!("Session:\n{session:#?}");

    let api_url = format!("{}/cfs/v2/sessions", self.base_url());
    http::post_json(self.http(), &api_url, token, session).await
  }

  /// Delete a CFS session by name.
  ///
  /// `DELETE /cfs/v2/sessions/{session_name}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_session_v2_delete(
    &self,
    token: &str,
    session_name: &str,
  ) -> Result<(), Error> {
    log::debug!("Deleting CFS session id: {session_name}");

    let api_url =
      format!("{}/cfs/v2/sessions/{}", self.base_url(), session_name);
    http::delete(self.http(), &api_url, token).await
  }
}
