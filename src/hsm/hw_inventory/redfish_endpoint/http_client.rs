//! `ShastaClient` methods for `/smd/hsm/v2/Inventory/RedfishEndpoints`.

use crate::{
  ShastaClient,
  common::http,
  error::Error,
  hsm::types::{HsmActionResponse, ResourceURI},
};

use super::types::{RedfishEndpoint, RedfishEndpointArray};

impl ShastaClient {
  /// Query Redfish endpoints filtered by xname.
  ///
  /// `GET /smd/hsm/v2/Inventory/RedfishEndpoint/Query/{xname}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_redfish_get_query(
    &self,
    token: &str,
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
      .bearer_auth(token)
      .send()
      .await?;

    http::handle_json_or_request_error(response, "GET").await
  }

  /// List Redfish endpoints with optional filters.
  ///
  /// `GET /smd/hsm/v2/Inventory/RedfishEndpoints`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  #[allow(clippy::too_many_arguments)]
  pub async fn hsm_redfish_get(
    &self,
    token: &str,
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
      .bearer_auth(token)
      .send()
      .await?;

    http::handle_json_or_request_error(response, "GET").await
  }

  /// Fetch one Redfish endpoint by xname.
  ///
  /// `GET /smd/hsm/v2/Inventory/RedfishEndpoints/{xname}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_redfish_get_one(
    &self,
    token: &str,
    xname: &str,
  ) -> Result<RedfishEndpoint, Error> {
    let api_url = format!(
      "{}/smd/hsm/v2/Inventory/RedfishEndpoints/{}",
      self.base_url(),
      xname
    );

    let response = self.http().get(api_url).bearer_auth(token).send().await?;
    http::handle_json_or_request_error(response, "GET").await
  }

  /// Create a Redfish endpoint. Returns the array of created resource
  /// URIs (typically one entry per posted endpoint).
  ///
  /// `POST /smd/hsm/v2/Inventory/RedfishEndpoints`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_redfish_post(
    &self,
    token: &str,
    redfish_endpoint: RedfishEndpoint,
  ) -> Result<Vec<ResourceURI>, Error> {
    let api_url =
      format!("{}/smd/hsm/v2/Inventory/RedfishEndpoints", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&redfish_endpoint)
      .send()
      .await?;

    http::handle_json_or_request_error(response, "POST").await
  }

  /// `PUT /hsm/v2/State/Components/{xname}` — replace a Redfish
  /// endpoint definition.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_redfish_put(
    &self,
    token: &str,
    xname: &str,
    redfish_endpoint: RedfishEndpoint,
  ) -> Result<RedfishEndpoint, Error> {
    let api_url =
      format!("{}/smd/hsm/v2/State/Components/{}", self.base_url(), xname);

    let response = self
      .http()
      .put(api_url)
      .bearer_auth(token)
      .json(&redfish_endpoint)
      .send()
      .await?;

    http::handle_json_or_request_error(response, "PUT").await
  }

  /// `DELETE /hsm/v2/Inventory/RedfishEndpoints` — remove every Redfish
  /// endpoint.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_redfish_delete_all(
    &self,
    token: &str,
  ) -> Result<HsmActionResponse, Error> {
    let api_url =
      format!("{}/smd/hsm/v2/Inventory/RedfishEndpoints", self.base_url());

    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(token)
      .send()
      .await?;

    http::handle_json_or_request_error(response, "DELETE").await
  }

  /// `DELETE /hsm/v2/Inventory/RedfishEndpoints/{xname}` — remove one
  /// Redfish endpoint.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_redfish_delete_one(
    &self,
    token: &str,
    xname: &str,
  ) -> Result<HsmActionResponse, Error> {
    let api_url = format!(
      "{}/smd/hsm/v2/Inventory/RedfishEndpoints/{}",
      self.base_url(),
      xname
    );

    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(token)
      .send()
      .await?;

    http::handle_json_or_request_error(response, "DELETE").await
  }
}
