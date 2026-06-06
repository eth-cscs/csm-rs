//! CFS configurations v2 — `ShastaClient` methods for
//! `/cfs/v2/configurations`.

pub(crate) mod types;

use crate::{
  ShastaClient,
  cfs::configuration::http_client::v2::types::{
    cfs_configuration_request::CfsConfigurationRequest,
    cfs_configuration_response::CfsConfigurationResponse,
  },
  common::http,
  error::Error,
};

const STUPID_LIMIT: i64 = 100000;

impl ShastaClient {
  /// Fetch one CFS configuration by name, or every configuration when
  /// `configuration_name_opt` is `None`.
  ///
  /// `GET /cfs/v2/configurations[/{name}]`. Always returns a `Vec` for
  /// uniform handling at the call site — single-name lookups produce a
  /// one-element vector.
  pub async fn cfs_configuration_v2_get(
    &self,
    token: &str,
    configuration_name_opt: Option<&str>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    log::debug!(
      "Get CFS configuration '{}'",
      configuration_name_opt.unwrap_or("all available")
    );

    let api_url = if let Some(name) = configuration_name_opt {
      format!("{}/cfs/v2/configurations/{}", self.base_url(), name)
    } else {
      format!("{}/cfs/v2/configurations", self.base_url())
    };

    if configuration_name_opt.is_some() {
      let payload: CfsConfigurationResponse = http::get_json_with_query(
        self.http(),
        &api_url,
        token,
        &[("limit", STUPID_LIMIT)],
      )
      .await?;
      Ok(vec![payload])
    } else {
      http::get_json_with_query(
        self.http(),
        &api_url,
        token,
        &[("limit", STUPID_LIMIT)],
      )
      .await
    }
  }

  /// List every CFS configuration on the system.
  ///
  /// Convenience wrapper for `cfs_configuration_v2_get(None)`.
  pub async fn cfs_configuration_v2_get_all(
    &self,
    token: &str,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    self.cfs_configuration_v2_get(token, None).await
  }

  /// Create or replace a CFS configuration by name with the supplied
  /// layer list.
  ///
  /// `PUT /cfs/v2/configurations/{configuration_name}`. The request body
  /// is `{ "layers": configuration.layers }`.
  pub async fn cfs_configuration_v2_put(
    &self,
    token: &str,
    configuration: &CfsConfigurationRequest,
    configuration_name: &str,
  ) -> Result<CfsConfigurationResponse, Error> {
    log::debug!("Create CFS configuration '{}'", configuration_name);
    log::debug!("Create CFS configuration request:\n{:#?}", configuration);

    let api_url = format!(
      "{}/cfs/v2/configurations/{}",
      self.base_url(),
      configuration_name
    );

    let request_payload = serde_json::json!({ "layers": configuration.layers });

    log::debug!(
      "CFS configuration request payload:\n{}",
      serde_json::to_string_pretty(&request_payload)
        .unwrap_or_else(|e| format!("<serialize error: {}>", e))
    );

    http::put_json(self.http(), &api_url, token, &request_payload).await
  }

  /// Delete a CFS configuration by id.
  ///
  /// `DELETE /cfs/v2/configurations/{configuration_id}`. CFS rejects
  /// the delete if the configuration is still referenced by an image
  /// or runtime binding; that surfaces as an HTTP error.
  pub async fn cfs_configuration_v2_delete(
    &self,
    token: &str,
    configuration_id: &str,
  ) -> Result<(), Error> {
    log::debug!("Delete CFS configuration {:?}", configuration_id);

    let api_url = format!(
      "{}/cfs/v2/configurations/{}",
      self.base_url(),
      configuration_id
    );
    http::delete(self.http(), &api_url, token).await
  }
}
