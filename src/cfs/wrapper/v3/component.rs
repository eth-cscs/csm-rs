//! Wrapper for `/cfs/v3/components`. Replaces
//! `src/cfs/component/http_client/v3/mod.rs`.
//!
//! Routed through the progenitor-generated client:
//! - *(none)* — every generated v3 component method returns the
//!   strict `types::V3ComponentData{,Collection,Array}` shape: typed
//!   `configuration_status` enum, `state: Vec<V3ConfigurationStateLayer>`
//!   without the looser `playbook` field, `error_count: Option<i64>`,
//!   etc. csm-rs's public `Component` is the hand-written shape
//!   (`id: Option<String>`, `state: Option<Vec<State>>` where each
//!   `State` carries the `playbook` field, `error_count: Option<u64>`,
//!   free-form `configuration_status: Option<String>`) and is
//!   re-exported from `cfs::v3`, consumed by `dispatcher_conv.rs`,
//!   `component/utils.rs`, `cleanup_session.rs`,
//!   `backend_connector/cfs.rs`, and the `manta-backend-dispatcher`
//!   trait impls. Adopting the generated types here would force a
//!   structural change across all those consumers (and the public
//!   `cfs::v3::Component` API) so this wave keeps everything on raw
//!   `reqwest`. A follow-up commit can migrate individual methods
//!   once a generated->hand-written conversion layer (or a swap of
//!   `Component` to the generated type) lands.
//!
//! Stays on raw `reqwest` because the generated surface doesn't
//! cover what the existing public API needs:
//!
//! - `cfs_component_v3_get_options` returns the tolerant
//!   `serde_json::Value`; the generated `get_options_v3` returns the
//!   strict `types::V3Options` struct, so any field the spec doesn't
//!   model would deserialize-fail today.
//! - `cfs_component_v3_get` accepts comma-joined ids and an arbitrary
//!   `status` string; the generated `get_components_v3` takes
//!   `status: Option<GetComponentsV3Status>` (typed enum), so passing
//!   a free-form status would require a fallible map. It also returns
//!   `V3ComponentDataCollection` — see the "routed via progenitor"
//!   section above for why we don't reach for it.
//! - `cfs_component_v3_get_single_by_id` returns the hand-written
//!   `Component`; the generated `get_component_v3` returns
//!   `V3ComponentData` (different field shape, see above).
//! - `cfs_component_v3_get_query_batch` is a chunking convenience
//!   wrapper over `cfs_component_v3_get_query` (60 ids per request,
//!   15 in flight), not an endpoint binding of its own.
//! - `cfs_component_v3_get_query` adds a `limit=100000` parameter that
//!   the spec exposes typed as `Option<NonZeroU64>` — workable, but
//!   the return type is `V3ComponentDataCollection`, see above.
//! - `cfs_component_v3_patch_component` takes the hand-written
//!   `Component` and returns the tolerant `Vec<Value>` body; the
//!   generated `patch_component_v3` takes `&V3ComponentData` and
//!   returns `V3ComponentData` (different shape both ways).
//! - `cfs_component_v3_patch_component_list` returns `()` on any 2xx
//!   via `response.status().is_success()` over the raw body; the
//!   generated `patch_components_v3` takes
//!   `body: &PatchComponentsV3Body` (a progenitor-emitted `oneOf`
//!   wrapper enum) and returns `V3ComponentIdCollection`, so the
//!   request and response shapes both diverge from the public
//!   `Vec<Component> -> ()` signature.
//! - `cfs_component_v3_put_component` takes the hand-written
//!   `Component`; the generated `put_component_v3` takes
//!   `&V3ComponentData` (different shape).
//! - `cfs_component_v3_put_component_list` is a sequential
//!   convenience wrapper over `cfs_component_v3_put_component`, not
//!   an endpoint binding of its own.
//! - `cfs_component_v3_delete_single_component` returns a `Component`
//!   body via the tolerant `handle_json_or_text_response` helper; the
//!   generated `delete_component_v3` is `Response = ()` on 204 only,
//!   so a caller relying on the deleted record's contents would lose
//!   data.

use std::time::Instant;

use serde_json::Value;

use crate::{
  ShastaClient,
  cfs::component::http_client::v3::types::{Component, ComponentVec},
  common::http,
  error::Error,
};

impl ShastaClient {
  /// Fetch CFS options.
  ///
  /// `GET /cfs/v3/options`. Returns the raw JSON object of CFS
  /// service-level options.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v3_get_options(
    &self,
    token: &str,
  ) -> Result<Value, Error> {
    let api_url = format!("{}/cfs/v3/options", self.base_url());

    let response = self
      .http()
      .get(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  /// Fetch CFS components, optionally filtered by id list and status.
  ///
  /// `GET /cfs/v3/components`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v3_get(
    &self,
    token: &str,
    components_ids: Option<&str>,
    status: Option<&str>,
  ) -> Result<Vec<Component>, Error> {
    let api_url = format!("{}/cfs/v3/components", self.base_url());

    let response = self
      .http()
      .get(api_url)
      .query(&[("ids", components_ids), ("status", status)])
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    let payload: ComponentVec =
      http::handle_json_or_text_response(response).await?;
    Ok(payload.components)
  }

  /// Fetch one CFS component by id.
  ///
  /// `GET /cfs/v3/components/{component_id}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v3_get_single_by_id(
    &self,
    token: &str,
    component_id: &str,
  ) -> Result<Component, Error> {
    let api_url =
      format!("{}/cfs/v3/components/{}", self.base_url(), component_id);

    let response = self
      .http()
      .get(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  /// Fetch CFS components for an arbitrarily large xname list by
  /// batching 60 ids per request, 15 requests in flight.
  ///
  /// Works around the CSM-side limit on a single GET; order of the
  /// returned components is not preserved.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v3_get_query_batch(
    &self,
    token: &str,
    configuration_name: Option<String>,
    node_vec: &[String],
    status: Option<String>,
  ) -> Result<Vec<Component>, Error> {
    let start = Instant::now();

    let client = self.clone();
    let token = token.to_string();
    let component_vec = http::parallel_batch(node_vec, 60, 15, move |chunk| {
      let client_clone = client.clone();
      let token_clone = token.clone();
      let config_name_clone = configuration_name.clone();
      let status_clone = status.clone();
      async move {
        let ids = chunk.join(",");
        client_clone
          .cfs_component_v3_get_query(
            &token_clone,
            config_name_clone.as_deref(),
            Some(&ids),
            status_clone.as_deref(),
          )
          .await
      }
    })
    .await?;

    log::debug!(
      "Time elapsed to get CFS components is: {:?}",
      start.elapsed()
    );
    Ok(component_vec)
  }

  /// Fetch CFS components with full query filters (configuration name,
  /// ids, status).
  ///
  /// `GET /cfs/v3/components` with `config_name`, `ids`, `status`,
  /// and a large `limit`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v3_get_query(
    &self,
    token: &str,
    configuration_name: Option<&str>,
    components_ids: Option<&str>,
    status: Option<&str>,
  ) -> Result<Vec<Component>, Error> {
    let stupid_limit = 100000;

    let api_url = format!("{}/cfs/v3/components", self.base_url());

    let response = self
      .http()
      .get(api_url)
      .query(&[
        ("ids", components_ids),
        ("config_name", configuration_name),
        ("status", status),
        ("limit", Some(&stupid_limit.to_string())),
      ])
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    let payload: ComponentVec =
      http::handle_json_or_text_response(response).await?;
    Ok(payload.components)
  }

  /// Apply a partial update to one CFS component.
  ///
  /// `PATCH /cfs/v3/components/{component.id}`. Returns
  /// [`Error::CfsComponentFieldNotDefined`] if `component.id` is `None`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v3_patch_component(
    &self,
    token: &str,
    component: Component,
  ) -> Result<Vec<Value>, Error> {
    let component_id = component
      .id
      .as_deref()
      .ok_or_else(|| Error::CfsComponentFieldNotDefined("id".to_string()))?;
    let api_url =
      format!("{}/cfs/v3/components/{}", self.base_url(), component_id);

    let response = self
      .http()
      .patch(api_url)
      .bearer_auth(token)
      .json(&component)
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  /// Bulk-patch many CFS components in a single request.
  ///
  /// `PATCH /cfs/v3/components` with the full list as JSON body.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v3_patch_component_list(
    &self,
    token: &str,
    component_list: Vec<Component>,
  ) -> Result<(), Error> {
    let api_url = format!("{}/cfs/v3/components", self.base_url());

    let response = self
      .http()
      .patch(api_url)
      .bearer_auth(token)
      .json(&component_list)
      .send()
      .await
      .map_err(Error::NetError)?;

    if response.status().is_success() {
      Ok(())
    } else {
      let payload = response.text().await.map_err(Error::NetError)?;
      Err(Error::Message(payload))
    }
  }

  /// Replace one CFS component record.
  ///
  /// `PUT /cfs/v3/components/{component.id}`. Returns
  /// [`Error::CfsComponentFieldNotDefined`] if `component.id` is `None`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v3_put_component(
    &self,
    token: &str,
    component: Component,
  ) -> Result<Component, Error> {
    let component_id = component
      .id
      .as_deref()
      .ok_or_else(|| Error::CfsComponentFieldNotDefined("id".to_string()))?;
    let api_url =
      format!("{}/cfs/v3/components/{}", self.base_url(), component_id);
    http::put_json(self.http(), &api_url, token, &component).await
  }

  /// Replace many CFS component records sequentially. Stops at the
  /// first error.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v3_put_component_list(
    &self,
    token: &str,
    component_list: Vec<Component>,
  ) -> Result<Vec<Component>, Error> {
    let mut out = Vec::with_capacity(component_list.len());
    for component in component_list {
      out.push(self.cfs_component_v3_put_component(token, component).await?);
    }
    Ok(out)
  }

  /// Delete a CFS component by id.
  ///
  /// `DELETE /cfs/v3/components/{component_id}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v3_delete_single_component(
    &self,
    token: &str,
    component_id: &str,
  ) -> Result<Component, Error> {
    let api_url =
      format!("{}/cfs/v3/components/{}", self.base_url(), component_id);

    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }
}
