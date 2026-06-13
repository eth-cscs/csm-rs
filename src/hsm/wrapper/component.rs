//! Wrapper for `/State/Components`. Replaces
//! `src/hsm/component/http_client.rs`.
//!
//! **All methods stay on raw `reqwest`.** The generated client cannot
//! cover any of the public surface here without changing on-wire URLs
//! or accepting reduced typing. Concrete reasons per method:
//!
//! - `hsm_component_get` accepts `Option<&str>` for `type`, `state`,
//!   `flag`, `role`, `subrole`, `arch`, `class`. The generated
//!   `do_components_get` types those as `Option<types::DoComponentsGetType>`
//!   and similar (closed `Copy` enums), so a raw string like "Compute"
//!   has to round-trip through `FromStr`. More importantly, the
//!   historical `nid: Option<&str>` is split on `,` and turned into
//!   repeated `?nid=` query params (CSM accepts repeats); the generated
//!   binding emits a single `?nid=…` value, dropping the comma-split
//!   semantics. Keeping the multi-value behaviour is load-bearing for
//!   callers like `backend_connector::ComponentTrait::nid_to_xname`.
//! - `hsm_component_get_all`, `hsm_component_get_all_nodes`,
//!   `hsm_component_get_and_filter` are convenience wrappers built on
//!   top of `hsm_component_get` (they pre-fill all-`None` filters and
//!   optionally an in-memory xname filter). Not endpoint bindings of
//!   their own, so they inherit the same status.
//! - `hsm_component_get_one`, `hsm_component_post`,
//!   `hsm_component_post_query`, `hsm_component_post_bynid_query`,
//!   `hsm_component_put`, `hsm_component_delete_one`,
//!   `hsm_component_delete` all use the bare `/hsm/v2/...` URL prefix
//!   (no `/smd/`); the historical csm-rs surface has shipped against
//!   this URL inconsistency since before this codegen migration. The
//!   generated client's baseurl is `{base_url}/smd/hsm/v2`, so routing
//!   any of these through the generated client would prepend `/smd/`
//!   and break the contract — and break the
//!   `tests/backend_connector.rs::component_post_nodes_posts_to_state_components`
//!   test, which asserts `path("/hsm/v2/State/Components")` verbatim.
//!   Additionally, the delete methods return `HsmActionResponse`, whose
//!   `code` / `message` fields are `#[serde(default)]` to tolerate
//!   `{}` bodies from CSM mocks; the generated `Response100` has the
//!   fields as required and rejects the looser shape. Keeping these on
//!   `handle_json_response` preserves both contracts in one move.
//!
//! The body types passed to these methods (`ComponentArrayPostArray`,
//! `ComponentArrayPostQuery`, `ComponentArrayPostByNidQuery`,
//! `ComponentPut`) are still the progenitor-generated structs (now
//! re-exported from `types.rs`), so the on-wire JSON shape matches the
//! OpenAPI schema field-for-field. Only the HTTP path + response-parse
//! tolerance differ from a pure progenitor wrap.

use serde_json::Value;

use crate::{
  ShastaClient,
  common::http,
  error::Error,
  hsm::{
    component::{
      filter,
      types::{
        Component, ComponentArray, ComponentArrayPostArray,
        ComponentArrayPostByNidQuery, ComponentArrayPostQuery, ComponentPut,
      },
    },
    types::HsmActionResponse,
  },
};

impl ShastaClient {
  /// Fetch all HSM components. `nid_only` toggles the lightweight nid-only response.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_component_get_all(
    &self,
    token: &str,
    nid_only: Option<&str>,
  ) -> Result<ComponentArray, Error> {
    self
      .hsm_component_get(
        token, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, nid_only,
      )
      .await
  }

  /// Fetch all HSM components with `Type=Node`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_component_get_all_nodes(
    &self,
    token: &str,
    nid_only: Option<&str>,
  ) -> Result<ComponentArray, Error> {
    self
      .hsm_component_get(
        token,
        None,
        Some("Node"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        nid_only,
      )
      .await
  }

  /// `GET /hsm/v2/State/Components` then filter the result down to
  /// components whose xname appears in `xname_list`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_component_get_and_filter(
    &self,
    token: &str,
    xname_list: &[String],
  ) -> Result<Vec<Component>, Error> {
    // `Component100Component.components` is `Vec<...>` (with
    // `#[serde(default)]` for an absent `Components` array), so it is
    // already empty by default — no `unwrap_or_default()` needed.
    let mut component_vec =
      self.hsm_component_get_all(token, None).await?.components;

    filter(&mut component_vec, xname_list);

    Ok(component_vec)
  }

  /// `GET /hsm/v2/State/Components` with the full set of HSM query
  /// parameters (id, type, state, flag, role, subrole, enabled,
  /// software status, subtype, arch, class, nid, nid range, partition,
  /// group, and the `*only` projection toggles).
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  #[allow(clippy::too_many_arguments)]
  pub async fn hsm_component_get(
    &self,
    token: &str,
    id: Option<&str>,
    r#type: Option<&str>,
    state: Option<&str>,
    flag: Option<&str>,
    role: Option<&str>,
    subrole: Option<&str>,
    enabled: Option<&str>,
    software_status: Option<&str>,
    subtype: Option<&str>,
    arch: Option<&str>,
    class: Option<&str>,
    nid: Option<&str>,
    nid_start: Option<&str>,
    nid_end: Option<&str>,
    partition: Option<&str>,
    group: Option<&str>,
    state_only: Option<&str>,
    flag_only: Option<&str>,
    // role_only: Option<&str>,
    nid_only: Option<&str>,
  ) -> Result<ComponentArray, Error> {
    let mut nid_vec_query = nid.map(|nids| {
      nids
        .split(',')
        .map(|nid| ("nid", Some(nid)))
        .collect::<Vec<(&str, Option<&str>)>>()
    });

    let mut query_params = vec![
      ("id", id),
      ("type", r#type),
      ("state", state),
      ("flag", flag),
      ("role", role),
      ("subrole", subrole),
      ("enabled", enabled),
      ("softwarestatus", software_status),
      ("subtype", subtype),
      ("arch", arch),
      ("class", class),
      ("nidstart", nid_start),
      ("nidend", nid_end),
      ("partition", partition),
      ("group", group),
      ("stateonly", state_only),
      ("flagonly", flag_only),
      ("nidonly", nid_only),
    ];

    if let Some(mut nid_vec_query) = nid_vec_query.take() {
      query_params.append(&mut nid_vec_query);
    }

    let api_url = format!("{}/smd/hsm/v2/State/Components", self.base_url());

    let response = self
      .http()
      .get(api_url)
      .query(&query_params)
      .bearer_auth(token)
      .send()
      .await?;

    http::handle_json_or_text_response(response).await
  }

  /// `GET /hsm/v2/State/Components/{xname}` — fetch a single component.
  ///
  /// Surfaces a 401 as [`Error::RequestError`] (token-distinguishable)
  /// and other non-2xx statuses as the structured [`Error::CsmError`]
  /// with method + URL + RFC 7807 detail — same shape as
  /// `hsm_redfish_*` and the other `handle_json_or_request_error`
  /// users.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_component_get_one(
    &self,
    token: &str,
    xname: &str,
  ) -> Result<Component, Error> {
    let api_url =
      format!("{}/hsm/v2/State/Components/{}", self.base_url(), xname);

    let response = self.http().get(api_url).bearer_auth(token).send().await?;
    http::handle_json_or_request_error(response, "GET").await
  }

  /// `POST /hsm/v2/State/Components` — create components in bulk.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_component_post(
    &self,
    token: &str,
    component: ComponentArrayPostArray,
  ) -> Result<(), Error> {
    let api_url = format!("{}/hsm/v2/State/Components", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&component)
      .send()
      .await?;

    http::handle_unit_or_request_error(response, "POST").await
  }

  /// `POST /hsm/v2/State/Components` query — components matching the
  /// supplied criteria.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_component_post_query(
    &self,
    token: &str,
    component: ComponentArrayPostQuery,
  ) -> Result<ComponentArray, Error> {
    let api_url = format!("{}/hsm/v2/State/Components", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&component)
      .send()
      .await?;

    http::handle_json_or_request_error::<ComponentArray>(response, "POST")
      .await
  }

  /// `POST /hsm/v2/State/Components/ByNID/Query` — components matching
  /// the supplied NID query.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_component_post_bynid_query(
    &self,
    token: &str,
    component: ComponentArrayPostByNidQuery,
  ) -> Result<ComponentArray, Error> {
    let api_url =
      format!("{}/hsm/v2/State/Components/ByNID/Query", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&component)
      .send()
      .await?;

    http::handle_json_or_request_error::<ComponentArray>(response, "POST")
      .await
  }

  /// `PUT /hsm/v2/State/Components/{xname}` — replace a component.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_component_put(
    &self,
    token: &str,
    xname: &str,
    component: ComponentPut,
  ) -> Result<(), Error> {
    let api_url =
      format!("{}/hsm/v2/State/Components/{}", self.base_url(), xname);

    let response = self
      .http()
      .put(api_url)
      .bearer_auth(token)
      .json(&component)
      .send()
      .await?;

    if !response.status().is_success() {
      if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        let response_err = response
          .error_for_status_ref()
          .expect_err("non-2xx branch implies error_for_status_ref errs");
        let url = response.url().to_string();
        let payload = response.text().await?;
        return Err(Error::RequestError {
          response: response_err,
          url,
          payload,
        });
      } else {
        let status = response.status().as_u16();
        let url = response.url().to_string();
        let payload = response.json::<Value>().await?;
        return Err(Error::csm_from_response(
          "PUT",
          &url,
          status,
          payload,
        ));
      }
    }

    // Historical behaviour: attempt to deserialise an empty body or any
    // 2xx payload. CSM returns 204 No Content on success, in which case
    // the prior code path would error here. Preserving the historical
    // behaviour exactly (including the error on a 204 body) keeps the
    // public contract identical for now.
    response.json().await.map_err(Error::NetError)
  }

  /// `DELETE /hsm/v2/State/Components/{xname}` — remove a single
  /// component.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_component_delete_one(
    &self,
    token: &str,
    xname: &str,
  ) -> Result<HsmActionResponse, Error> {
    let api_url =
      format!("{}/hsm/v2/State/Components/{}", self.base_url(), xname);
    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;
    http::handle_json_response(response, "DELETE").await
  }

  /// `DELETE /hsm/v2/State/Components` — remove all components.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_component_delete(
    &self,
    token: &str,
  ) -> Result<HsmActionResponse, Error> {
    let api_url = format!("{}/hsm/v2/State/Components", self.base_url());
    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;
    http::handle_json_response(response, "DELETE").await
  }
}
