//! Wrapper for `/Inventory/RedfishEndpoints`. Replaces
//! `src/hsm/hw_inventory/redfish_endpoint/http_client.rs`.
//!
//! **All seven methods stay on raw `reqwest`.** Routing each through
//! the generated client would either silently change the public type
//! surface, the on-wire URL, or the response-parse tolerance — none of
//! which are acceptable without a separate breaking-change PR. Per-method
//! rationale:
//!
//! - `hsm_redfish_get_query` and `hsm_redfish_get` return
//!   `RedfishEndpointArray` (the hand-written
//!   `super::types::RedfishEndpointArray`). The generated equivalents
//!   return `RedfishEndpointArrayRedfishEndpointArray` whose inner item
//!   is `RedfishEndpoint100RedfishEndpoint`, with nested
//!   `RedfishEndpoint100RedfishEndpointMacAddr` /
//!   `RedfishEndpoint100RedfishEndpointDiscoveryInfoLastStatus`
//!   newtypes/enums. Re-exporting the generated `RedfishEndpoint` over
//!   the hand-written one would cascade through `dispatcher_conv.rs`
//!   (16 field-by-field `From` impls) — same situation as Task 9,
//!   defer the type swap and keep the hand-written types instead.
//! - `hsm_redfish_get` historically passes
//!   `.query(&[id, fqdn, type, uuid, macaddr, ip_address, last_status])`
//!   (a `[Option<&str>; 7]`). The generated `do_redfish_endpoints_get`
//!   types `type_` as a closed enum and emits one query pair per
//!   `Some(_)` value; routing through it would change the on-wire query
//!   string, which is a contract change. Preserve the existing call.
//! - `hsm_redfish_get_one` returns the hand-written `RedfishEndpoint` —
//!   same nested-type cascade as above.
//! - `hsm_redfish_post` accepts a hand-written `RedfishEndpoint` body
//!   and returns `Vec<ResourceURI>` (capitalised `URI`). The generated
//!   `do_redfish_endpoints_post` accepts
//!   `RedfishEndpoint100RedfishEndpoint` and returns
//!   `Vec<ResourceUri100>` (the inner field is `Uri100` newtype). Both
//!   serialise to the same JSON, but the public Rust types differ;
//!   keeping raw avoids a `From`-impl bridge of its own.
//! - `hsm_redfish_put` issues the request against
//!   `/smd/hsm/v2/State/Components/{xname}` — **not** the
//!   `/Inventory/RedfishEndpoints/{xname}` URL the spec defines. This is
//!   an historical csm-rs quirk inherited from the previous
//!   hand-rolled client. The generated `do_redfish_endpoint_put` hits
//!   the spec URL, so routing through it would silently rewrite the on-
//!   wire path. Preserved verbatim.
//! - `hsm_redfish_delete_all` and `hsm_redfish_delete_one` return
//!   `HsmActionResponse` whose `code` / `message` are `#[serde(default)]`
//!   to tolerate `{}` bodies CSM (and integration test mocks) sometimes
//!   emit. The generated `do_redfish_endpoints_delete_all` /
//!   `do_redfish_endpoint_delete` decode into `Response100` whose
//!   `code`+`message` are required, so a `{}` body would fail with
//!   `InvalidResponsePayload`. The historical lenient parse keeps
//!   downstream callers + mocks portable.

use crate::{
  ShastaClient,
  common::http,
  error::Error,
  hsm::{
    hw_inventory::redfish_endpoint::types::{
      RedfishEndpoint, RedfishEndpointArray,
    },
    types::{HsmActionResponse, ResourceURI},
  },
};

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
  /// Note: the URL targets `/State/Components/{xname}` rather than
  /// the spec-defined `/Inventory/RedfishEndpoints/{xname}` — this
  /// is an historical csm-rs quirk preserved verbatim.
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
