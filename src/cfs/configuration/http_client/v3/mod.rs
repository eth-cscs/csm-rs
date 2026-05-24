pub mod types;

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
  pub async fn cfs_configuration_v3_get(
    &self,
    configuration_name_opt: Option<&str>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    log::info!("Get CFS configuration {:?}", configuration_name_opt);

    let api_url = if let Some(name) = configuration_name_opt {
      format!("{}/cfs/v3/configurations/{}", self.base_url(), name)
    } else {
      format!("{}/cfs/v3/configurations", self.base_url())
    };

    let response = self
      .http()
      .get(api_url)
      .query(&[("limit", STUPID_LIMIT)])
      .bearer_auth(self.token())
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

  // This function enforces a new CFS configuration to be created. First, checks
  // if CFS configuration with same name already exists in CSM, if that is the
  // case, it will return an error, otherwise creates a new CFS configuration
  pub async fn cfs_configuration_v3_put(
    &self,
    configuration: &CfsConfigurationRequest,
    configuration_name: &str,
  ) -> Result<CfsConfigurationResponse, Error> {
    // Check if CFS configuration already exists
    log::info!("Check CFS configuration '{}' exists", configuration_name);

    let cfs_configuration_rslt =
      self.cfs_configuration_v3_get(Some(configuration_name)).await;

    if cfs_configuration_rslt
      .is_ok_and(|cfs_configuration_vec| !cfs_configuration_vec.is_empty())
    {
      return Err(Error::Message(format!(
        "CFS configuration '{}' already exists.",
        configuration_name
      )));
    }

    log::info!(
      "CFS configuration '{}' does not exists, creating new CFS configuration",
      configuration_name
    );

    log::info!("Create CFS configuration '{}'", configuration_name);
    log::debug!("Create CFS configuration request:\n{:#?}", configuration);

    let api_url = format!(
      "{}/cfs/v3/configurations/{}",
      self.base_url(),
      configuration_name
    );

    let request_payload =
      serde_json::json!({ "layers": configuration.layers });
    log::debug!(
      "CFS configuration request payload:\n{}",
      serde_json::to_string_pretty(&request_payload)
        .unwrap_or_else(|e| format!("<serialize error: {}>", e))
    );

    let response = self
      .http()
      .put(api_url)
      .json(&request_payload)
      .bearer_auth(self.token())
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  pub async fn cfs_configuration_v3_delete(
    &self,
    configuration_id: &str,
  ) -> Result<(), Error> {
    log::info!("Delete CFS configuration '{}'", configuration_id);

    let api_url = format!(
      "{}/cfs/v3/configurations/{}",
      self.base_url(),
      configuration_id
    );
    http::delete(self.http(), &api_url, self.token()).await
  }
}
