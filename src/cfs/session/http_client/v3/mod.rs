//! CFS sessions v3 — `ShastaClient` methods for `/cfs/v3/sessions`.

pub(crate) mod types;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// CFS v3 session mirror types. Gated behind the `manta-dispatcher`
/// Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;

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
