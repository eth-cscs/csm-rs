//! Wrapper for `/cfs/v3/sessions`. Replaces
//! `src/cfs/session/http_client/v3/mod.rs`.
//!
//! Routed through the progenitor-generated client:
//! - *(none)* — every generated v3 session method exchanges the strict
//!   `types::V3SessionData{,Collection}` / `types::V3SessionCreate`
//!   shape:
//!     * `V3SessionData.name: Option<V3SessionDataName>` (newtype with
//!       the 45-char/`^[a-z0-9]...$` pattern baked in) — csm-rs's
//!       hand-written `CfsSessionGetResponse.name` is `String`,
//!     * `V3SessionData.tags: HashMap<String, String>` (non-optional,
//!       defaulted) — hand-written is `Option<HashMap<String, String>>`,
//!     * `V3SessionCreate.name: V3SessionCreateName` (newtype),
//!       `configuration_name: V3SessionCreateConfigurationName` (newtype),
//!       `ansible_limit: V3SessionCreateAnsibleLimit` (newtype with
//!       `^[^\s;]*$` pattern, non-optional with a `""` default),
//!       `ansible_passthrough: String` (non-optional, defaulted),
//!       `ansible_verbosity: i64` (non-optional, defaulted),
//!       `configuration_limit: String` (non-optional, defaulted) —
//!       hand-written `CfsSessionPostRequest` keeps `name: String`,
//!       `configuration_name: String`, `ansible_limit: Option<String>`,
//!       `ansible_passthrough: Option<String>`,
//!       `ansible_verbosity: Option<u8>`, and
//!       `configuration_limit: Option<String>`,
//!     * `V3SessionCreate.target: Option<SessionTargetSection>` and
//!       `V3SessionData.target: Option<SessionTargetSection>` — both use
//!       the generated `SessionTargetSection` shape; hand-written
//!       `CfsSessionPostRequest.target` is the local `Target` struct
//!       (non-optional, defaulted to empty, with
//!       `definition: Option<String>`, `groups: Option<Vec<Group>>`,
//!       `image_map: Option<Vec<ImageMap>>`) and
//!       `CfsSessionGetResponse.target` is the same local `Target`,
//!     * `V3SessionData.status: Option<V3SessionStatusSection>` — the
//!       hand-written `Status` carries `Option<Vec<Artifact>>` and a
//!       free-form `Option<Session>` whose `succeeded` is `Option<String>`
//!       (the `is_success` helper compares it to the literal `"true"`),
//!       not the generated typed shape.
//!   csm-rs's public `CfsSessionGetResponse` / `CfsSessionPostRequest`
//!   are re-exported from `cfs::v3`, consumed by `dispatcher_conv.rs`
//!   (`From` impls between hand-written types and the dispatcher
//!   mirrors), the SAT-file session translation in `cfs::session::utils`
//!   and `cfs::session::utils::yaml`, `cleanup_session.rs`,
//!   `cleanup.rs`, `backend_connector/cfs.rs`, and the
//!   `manta-backend-dispatcher` trait impls. Adopting the generated
//!   types here would force a structural change across all those
//!   consumers (and the public `cfs::v3::CfsSession*` API) so this wave
//!   keeps everything on raw `reqwest`. A follow-up commit can migrate
//!   individual methods once a generated->hand-written conversion layer
//!   (or a swap of the public types to the generated ones) lands.
//!
//! Stays on raw `reqwest` because the generated surface doesn't
//! cover what the existing public API needs:
//!
//! - `cfs_session_v3_get` multiplexes `GET /cfs/v3/sessions` and
//!   `GET /cfs/v3/sessions/{name}` and wraps the single-name response
//!   in a one-element `Vec` for uniform caller handling. The generated
//!   client splits these into `get_sessions_v3` (returns
//!   `V3SessionDataCollection`) and `get_session_v3` (returns
//!   `V3SessionData`). The list variant accepts typed
//!   `GetSessionsV3Status` / `GetSessionsV3Succeeded` enums and typed
//!   `GetSessionsV3MaxAge` / `GetSessionsV3MinAge` newtypes while the
//!   hand-written signature takes `status_opt: Option<String>`,
//!   `is_succeded_opt: Option<bool>`, and `Option<String>` ages, so
//!   adopting the generated client would require fallible
//!   string-to-enum / string-to-newtype conversions. The hand-written
//!   code also sends the `succeced` query parameter (mis-spelled to
//!   match a long-standing CSM-side typo carried over from v2); the
//!   generated client sends the spec-canonical `succeeded`. Swapping
//!   spellings is a behavioural change that needs its own evaluation
//!   rather than riding along with codegen adoption.
//! - `cfs_session_v3_post` takes `&CfsSessionPostRequest` and returns
//!   `CfsSessionGetResponse`; the generated `create_session_v3` takes
//!   `&V3SessionCreate` (newtype-validated `name` /
//!   `configuration_name` / `ansible_limit`, non-optional
//!   `ansible_verbosity: i64` / `ansible_passthrough: String` /
//!   `configuration_limit: String`, generated `SessionTargetSection`)
//!   and returns `V3SessionData` (different `name`, `tags`, `status`,
//!   and `target` shapes — see the "routed via progenitor" section
//!   above).
//! - `cfs_session_v3_delete` returns `()` and matches the generated
//!   `delete_session_v3` signature on its own. We still keep it on raw
//!   `reqwest` for now to avoid leaving a single progenitor-routed
//!   method dangling in a module whose other two methods are blocked
//!   on the contract mismatches above; routing just `delete` would
//!   force the wrapper to mix two error/transport paths for no
//!   behavioural gain. Migrating delete on its own is safe to revisit
//!   alongside the future swap of the public session types.

use crate::{
  ShastaClient,
  cfs::session::http_client::v3::types::{
    CfsSessionGetResponse, CfsSessionGetResponseList, CfsSessionPostRequest,
  },
  common::http,
  error::Error,
};

impl ShastaClient {
  /// Fetch CFS sessions via the v3 API, optionally filtered by name,
  /// age, status, name substring, success flag, and tags.
  ///
  /// `GET /cfs/v3/sessions[/{name}]`. When `session_name_opt` is set
  /// the returned `Vec` always has at most one element. `limit_opt` and
  /// `after_id_opt` paginate the list form.
  ///
  /// See <https://apidocs.svc.cscs.ch/paas/cfs/operation/get_sessions/>
  /// for the underlying REST contract.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  #[allow(clippy::too_many_arguments)]
  pub async fn cfs_session_v3_get(
    &self,
    token: &str,
    session_name_opt: Option<&String>,
    limit_opt: Option<u8>,
    after_id_opt: Option<String>,
    min_age_opt: Option<String>,
    max_age_opt: Option<String>,
    status_opt: Option<String>,
    name_contains_opt: Option<String>,
    is_succeded_opt: Option<bool>,
    tags_opt: Option<String>,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    let api_url = if let Some(session_name) = session_name_opt {
      format!("{}/cfs/v3/sessions/{}", self.base_url(), session_name)
    } else {
      format!("{}/cfs/v3/sessions", self.base_url())
    };

    let mut query_params: Vec<(&str, String)> = Vec::new();
    if let Some(limit) = limit_opt {
      query_params.push(("limit", limit.to_string()));
    }
    if let Some(after_id) = after_id_opt {
      query_params.push(("after_id", after_id));
    }
    if let Some(min_age) = min_age_opt {
      query_params.push(("min_age", min_age));
    }
    if let Some(max_age) = max_age_opt {
      query_params.push(("max_age", max_age));
    }
    if let Some(status) = status_opt {
      query_params.push(("status", status));
    }
    if let Some(name_contains) = name_contains_opt {
      query_params.push(("name_contains", name_contains));
    }
    if let Some(is_succeded) = is_succeded_opt {
      query_params.push(("succeced", is_succeded.to_string()));
    }
    if let Some(tags) = tags_opt {
      query_params.push(("tags", tags));
    }

    if session_name_opt.is_some() {
      let payload: CfsSessionGetResponse =
        http::get_json_with_query(self.http(), &api_url, token, &query_params)
          .await?;
      Ok(vec![payload])
    } else {
      let payload: CfsSessionGetResponseList =
        http::get_json_with_query(self.http(), &api_url, token, &query_params)
          .await?;
      Ok(payload.sessions)
    }
  }

  /// Create a new CFS session via the v3 API.
  ///
  /// `POST /cfs/v3/sessions`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_session_v3_post(
    &self,
    token: &str,
    session: &CfsSessionPostRequest,
  ) -> Result<CfsSessionGetResponse, Error> {
    log::debug!("Session:\n{session:#?}");

    let api_url = format!("{}/cfs/v3/sessions", self.base_url());
    http::post_json(self.http(), &api_url, token, session).await
  }

  /// Delete a CFS session by name via the v3 API.
  ///
  /// `DELETE /cfs/v3/sessions/{session_name}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_session_v3_delete(
    &self,
    token: &str,
    session_name: &str,
  ) -> Result<(), Error> {
    log::debug!("Deleting CFS session id: {session_name}");

    let api_url =
      format!("{}/cfs/v3/sessions/{}", self.base_url(), session_name);
    http::delete(self.http(), &api_url, token).await
  }
}
