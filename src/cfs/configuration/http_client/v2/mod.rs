pub mod types;

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
  pub async fn cfs_configuration_v2_get(
    &self,
    configuration_name_opt: Option<&str>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    log::info!(
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
        self.token(),
        &[("limit", STUPID_LIMIT)],
      )
      .await?;
      Ok(vec![payload])
    } else {
      http::get_json_with_query(
        self.http(),
        &api_url,
        self.token(),
        &[("limit", STUPID_LIMIT)],
      )
      .await
    }
  }

  pub async fn cfs_configuration_v2_get_all(
    &self,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    self.cfs_configuration_v2_get(None).await
  }

  pub async fn cfs_configuration_v2_put(
    &self,
    configuration: &CfsConfigurationRequest,
    configuration_name: &str,
  ) -> Result<CfsConfigurationResponse, Error> {
    log::info!("Create CFS configuration '{}'", configuration_name);
    log::debug!("Create CFS configuration request:\n{:#?}", configuration);

    let api_url = format!(
      "{}/cfs/v2/configurations/{}",
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

    http::put_json(self.http(), &api_url, self.token(), &request_payload).await
  }

  pub async fn cfs_configuration_v2_delete(
    &self,
    configuration_id: &str,
  ) -> Result<(), Error> {
    log::info!("Delete CFS configuration {:?}", configuration_id);

    let api_url = format!(
      "{}/cfs/v2/configurations/{}",
      self.base_url(),
      configuration_id
    );
    http::delete(self.http(), &api_url, self.token()).await
  }
}
