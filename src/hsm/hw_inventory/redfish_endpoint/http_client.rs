//! `ShastaClient` methods for `/smd/hsm/v2/Inventory/RedfishEndpoints`.

use serde_json::Value;

use crate::{ShastaClient, common::http, error::Error};

use super::types::{RedfishEndpoint, RedfishEndpointArray};

impl ShastaClient {
  /// Query Redfish endpoints filtered by xname.
  ///
  /// `GET /smd/hsm/v2/Inventory/RedfishEndpoint/Query/{xname}`.
  pub async fn hsm_redfish_get_query(
    &self,
    xname: &str,
  ) -> Result<RedfishEndpointArray, Error> {
    let api_url = format!(
      "{}/smd/hsm/v2/Inventory/RedfishEndpoint/Query/{}",
      self.base_url(),
      xname
    );

    let response = self
      .http()
      .get(api_url)
      .query(&[xname])
      .bearer_auth(self.token())
      .send()
      .await?;

    http::handle_json_or_request_error(response).await
  }

  /// List Redfish endpoints with optional filters.
  ///
  /// `GET /smd/hsm/v2/Inventory/RedfishEndpoints`.
  #[allow(clippy::too_many_arguments)]
  pub async fn hsm_redfish_get(
    &self,
    id: Option<&str>,
    fqdn: Option<&str>,
    r#type: Option<&str>,
    uuid: Option<&str>,
    macaddr: Option<&str>,
    ip_address: Option<&str>,
    last_status: Option<&str>,
  ) -> Result<RedfishEndpointArray, Error> {
    let api_url =
      format!("{}/smd/hsm/v2/Inventory/RedfishEndpoints", self.base_url());

    let response = self
      .http()
      .get(api_url)
      .query(&[id, fqdn, r#type, uuid, macaddr, ip_address, last_status])
      .bearer_auth(self.token())
      .send()
      .await?;

    http::handle_json_or_request_error(response).await
  }

  /// Fetch one Redfish endpoint by xname.
  ///
  /// `GET /smd/hsm/v2/Inventory/RedfishEndpoints/{xname}`.
  pub async fn hsm_redfish_get_one(
    &self,
    xname: &str,
  ) -> Result<RedfishEndpoint, Error> {
    let api_url = format!(
      "{}/smd/hsm/v2/Inventory/RedfishEndpoints/{}",
      self.base_url(),
      xname
    );

    let response = self
      .http()
      .get(api_url)
      .bearer_auth(self.token())
      .send()
      .await?;
    http::handle_json_or_request_error(response).await
  }

  /// Create a Redfish endpoint.
  ///
  /// `POST /smd/hsm/v2/Inventory/RedfishEndpoints`.
  pub async fn hsm_redfish_post(
    &self,
    redfish_endpoint: RedfishEndpoint,
  ) -> Result<Value, Error> {
    let api_url =
      format!("{}/smd/hsm/v2/Inventory/RedfishEndpoints", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(self.token())
      .json(&redfish_endpoint)
      .send()
      .await?;

    http::handle_json_or_request_error(response).await
  }

  pub async fn hsm_redfish_put(
    &self,
    xname: &str,
    redfish_endpoint: RedfishEndpoint,
  ) -> Result<RedfishEndpoint, Error> {
    let api_url =
      format!("{}/smd/hsm/v2/State/Components/{}", self.base_url(), xname);

    let response = self
      .http()
      .put(api_url)
      .bearer_auth(self.token())
      .json(&redfish_endpoint)
      .send()
      .await?;

    http::handle_json_or_request_error(response).await
  }

  pub async fn hsm_redfish_delete_all(&self) -> Result<Value, Error> {
    let api_url =
      format!("{}/smd/hsm/v2/Inventory/RedfishEndpoints", self.base_url());

    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(self.token())
      .send()
      .await?;

    http::handle_json_or_request_error(response).await
  }

  pub async fn hsm_redfish_delete_one(
    &self,
    xname: &str,
  ) -> Result<Value, Error> {
    let api_url = format!(
      "{}/smd/hsm/v2/Inventory/RedfishEndpoints/{}",
      self.base_url(),
      xname
    );

    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(self.token())
      .send()
      .await?;

    http::handle_json_or_request_error(response).await
  }
}
