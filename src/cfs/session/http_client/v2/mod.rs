pub mod types;

use crate::{ShastaClient, common::http, error::Error};

use super::v2::types::{CfsSessionGetResponse, CfsSessionPostRequest};

impl ShastaClient {
  /// Fetch CFS sessions ref --> https://apidocs.svc.cscs.ch/paas/cfs/operation/get_sessions/
  // FIX: change parameters types from '&String' to '&str'
  pub async fn cfs_session_v2_get(
    &self,
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
      let payload: CfsSessionGetResponse = http::get_json_with_query(
        self.http(),
        &api_url,
        self.token(),
        &query_params,
      )
      .await?;
      Ok(vec![payload])
    } else {
      http::get_json_with_query(
        self.http(),
        &api_url,
        self.token(),
        &query_params,
      )
      .await
    }
  }

  pub async fn cfs_session_v2_get_all(
    &self,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    self
      .cfs_session_v2_get(None, None, None, None, None)
      .await
  }

  pub async fn cfs_session_v2_post(
    &self,
    session: &CfsSessionPostRequest,
  ) -> Result<CfsSessionGetResponse, Error> {
    log::debug!("Session:\n{:#?}", session);

    let api_url = format!("{}/cfs/v2/sessions", self.base_url());
    http::post_json(self.http(), &api_url, self.token(), session).await
  }

  pub async fn cfs_session_v2_delete(
    &self,
    session_name: &str,
  ) -> Result<(), Error> {
    log::info!("Deleting CFS session id: {}", session_name);

    let api_url =
      format!("{}/cfs/v2/sessions/{}", self.base_url(), session_name);
    http::delete(self.http(), &api_url, self.token()).await
  }
}
