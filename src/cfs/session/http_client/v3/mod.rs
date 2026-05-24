pub mod types;

use crate::{
  ShastaClient,
  cfs::session::http_client::v3::types::{
    CfsSessionGetResponse, CfsSessionGetResponseList, CfsSessionPostRequest,
  },
  common::http,
  error::Error,
};

impl ShastaClient {
  /// Fetch CFS sessions ref --> https://apidocs.svc.cscs.ch/paas/cfs/operation/get_sessions/
  #[allow(clippy::too_many_arguments)]
  pub async fn cfs_session_v3_get(
    &self,
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
      let payload: CfsSessionGetResponse = http::get_json_with_query(
        self.http(),
        &api_url,
        self.token(),
        &query_params,
      )
      .await?;
      Ok(vec![payload])
    } else {
      let payload: CfsSessionGetResponseList = http::get_json_with_query(
        self.http(),
        &api_url,
        self.token(),
        &query_params,
      )
      .await?;
      Ok(payload.sessions)
    }
  }

  pub async fn cfs_session_v3_post(
    &self,
    session: &CfsSessionPostRequest,
  ) -> Result<CfsSessionGetResponse, Error> {
    log::debug!("Session:\n{:#?}", session);

    let api_url = format!("{}/cfs/v3/sessions", self.base_url());
    http::post_json(self.http(), &api_url, self.token(), session).await
  }

  pub async fn cfs_session_v3_delete(
    &self,
    session_name: &str,
  ) -> Result<(), Error> {
    log::info!("Deleting CFS session id: {}", session_name);

    let api_url =
      format!("{}/cfs/v3/sessions/{}", self.base_url(), session_name);
    http::delete(self.http(), &api_url, self.token()).await
  }
}
