//! CFS configurations v3 — `ShastaClient` methods for
//! `/cfs/v3/configurations`.

pub(crate) mod types;

use crate::{
  ShastaClient,
  cfs::configuration::http_client::v3::types::{
    cfs_configuration_request::CfsConfigurationRequest,
    cfs_configuration_response::{
      CfsConfigurationResponse, CfsConfigurationVecResponse,
    },
  },
  common::http,
  error::Error,
};

const STUPID_LIMIT: i64 = 100000;

impl ShastaClient {
  /// Fetch one CFS configuration by name, or every configuration when
  /// `configuration_name_opt` is `None`, using the v3 API.
  ///
  /// `GET /cfs/v3/configurations[/{name}]`. CFS v3 returns plain-text
  /// error bodies and a different success shape for single vs. list
  /// lookups; both are normalised here.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_configuration_v3_get(
    &self,
    token: &str,
    configuration_name_opt: Option<&str>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    log::debug!("Get CFS configuration {:?}", configuration_name_opt);

    let api_url = if let Some(name) = configuration_name_opt {
      format!("{}/cfs/v3/configurations/{}", self.base_url(), name)
    } else {
      format!("{}/cfs/v3/configurations", self.base_url())
    };

    let response = self
      .http()
      .get(api_url)
      .query(&[("limit", STUPID_LIMIT)])
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    // CFS v3 returns plain-text errors on failure (not JSON), and a different
    // success shape depending on whether a single config was requested.
    if configuration_name_opt.is_some() {
      let payload: CfsConfigurationResponse =
        http::handle_json_or_text_response(response).await?;
      Ok(vec![payload])
    } else {
      let payload: CfsConfigurationVecResponse =
        http::handle_json_or_text_response(response).await?;
      Ok(payload.configurations)
    }
  }

  /// Create a CFS configuration by name, refusing to overwrite if one
  /// already exists.
  ///
  /// `PUT /cfs/v3/configurations/{configuration_name}`. Unlike a bare
  /// `PUT`, this checks first via [`Self::cfs_configuration_v3_get`]
  /// and returns [`Error::Message`] if a configuration with the same
  /// name is already present.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_configuration_v3_put(
    &self,
    token: &str,
    configuration: &CfsConfigurationRequest,
    configuration_name: &str,
  ) -> Result<CfsConfigurationResponse, Error> {
    // Check if CFS configuration already exists
    log::debug!("Check CFS configuration '{}' exists", configuration_name);

    let cfs_configuration_rslt = self
      .cfs_configuration_v3_get(token, Some(configuration_name))
      .await;

    if cfs_configuration_rslt
      .is_ok_and(|cfs_configuration_vec| !cfs_configuration_vec.is_empty())
    {
      return Err(Error::ConfigurationAlreadyExists(
        configuration_name.to_string(),
      ));
    }

    log::debug!(
      "CFS configuration '{}' does not exists, creating new CFS configuration",
      configuration_name
    );

    log::debug!("Create CFS configuration '{}'", configuration_name);
    log::debug!("Create CFS configuration request:\n{:#?}", configuration);

    let api_url = format!(
      "{}/cfs/v3/configurations/{}",
      self.base_url(),
      configuration_name
    );

    let request_payload = serde_json::json!({ "layers": configuration.layers });
    log::debug!(
      "CFS configuration request payload:\n{}",
      serde_json::to_string_pretty(&request_payload)
        .unwrap_or_else(|e| format!("<serialize error: {}>", e))
    );

    let response = self
      .http()
      .put(api_url)
      .json(&request_payload)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  /// Delete a CFS configuration by id via the v3 API.
  ///
  /// `DELETE /cfs/v3/configurations/{configuration_id}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_configuration_v3_delete(
    &self,
    token: &str,
    configuration_id: &str,
  ) -> Result<(), Error> {
    log::debug!("Delete CFS configuration '{}'", configuration_id);

    let api_url = format!(
      "{}/cfs/v3/configurations/{}",
      self.base_url(),
      configuration_id
    );
    http::delete(self.http(), &api_url, token).await
  }
}
