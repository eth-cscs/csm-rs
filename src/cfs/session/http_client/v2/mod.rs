//! CFS sessions v2 — `ShastaClient` methods for `/cfs/v2/sessions`.

pub(crate) mod types;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// CFS v2 session mirror types. Gated behind the `manta-dispatcher`
/// Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;

use crate::{ShastaClient, common::http, error::Error};

use super::v2::types::{CfsSessionGetResponse, CfsSessionPostRequest};

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
  pub async fn cfs_session_v2_get(
    &self,
    token: &str,
    min_age_opt: Option<&String>,
    max_age_opt: Option<&String>,
    status_opt: Option<&String>,
    session_name_opt: Option<&String>,
    is_succeded_opt: Option<bool>,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    log::info!(
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
      query_params.push(("min_age", min_age.to_string()));
    }
    if let Some(max_age) = max_age_opt {
      query_params.push(("max_age", max_age.to_string()));
    }
    if let Some(status) = status_opt {
      query_params.push(("status", status.to_string()));
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
  pub async fn cfs_session_v2_post(
    &self,
    token: &str,
    session: &CfsSessionPostRequest,
  ) -> Result<CfsSessionGetResponse, Error> {
    log::debug!("Session:\n{:#?}", session);

    let api_url = format!("{}/cfs/v2/sessions", self.base_url());
    http::post_json(self.http(), &api_url, token, session).await
  }

  /// Delete a CFS session by name.
  ///
  /// `DELETE /cfs/v2/sessions/{session_name}`.
  pub async fn cfs_session_v2_delete(
    &self,
    token: &str,
    session_name: &str,
  ) -> Result<(), Error> {
    log::info!("Deleting CFS session id: {}", session_name);

    let api_url =
      format!("{}/cfs/v2/sessions/{}", self.base_url(), session_name);
    http::delete(self.http(), &api_url, token).await
  }
}
