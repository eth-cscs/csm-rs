//! Wrapper for `/cfs/v2/components`. Replaces
//! `src/cfs/component/http_client/v2/mod.rs`.
//!
//! Routed through the progenitor-generated client:
//! - *(none)* — every generated v2 component method returns the
//!   strict `types::V2ComponentState{,Array}` shape (newtype `id:
//!   Option<ComponentId>`, enum `configuration_status`, `state:
//!   Vec<V2ConfigurationStateLayer>`, `error_count: Option<i64>`).
//!   csm-rs's public `Component` is the looser hand-written shape
//!   (`id: Option<String>`, `state: Option<Vec<State>>` with the
//!   `playbook` field, `error_count: Option<u64>`, etc.) and is
//!   re-exported from `cfs::v2`, consumed by `dispatcher_conv.rs`,
//!   `cleanup_session.rs`, `backend_connector/cfs.rs`, and the
//!   `manta-backend-dispatcher` trait impls. Adopting the generated
//!   types here would force a structural change across all those
//!   consumers (and the public `cfs::v2::Component` API) so this
//!   wave keeps everything on raw `reqwest`. A follow-up commit can
//!   migrate individual methods once a generated->hand-written
//!   conversion layer (or a swap of `Component` to the generated
//!   type) lands.
//!
//! Stays on raw `reqwest` because the generated surface doesn't
//! cover what the existing public API needs:
//!
//! - `cfs_component_v2_get` accepts comma-joined ids and an
//!   arbitrary `status` string; the generated `get_components_v2`
//!   takes `status: Option<GetComponentsV2Status>` (typed enum), so
//!   passing a free-form status would require a fallible map.
//! - `cfs_component_v2_get_all` returns the looser hand-written
//!   `Vec<Component>` shape — see the "routed via progenitor"
//!   section above for why we don't reach for `get_components_v2`.
//! - `cfs_component_v2_get_single_component` returns the
//!   hand-written `Component`; the generated `get_component_v2`
//!   returns `V2ComponentState` (different field shape, see above).
//! - `cfs_component_v2_get_multiple` is a chunking convenience
//!   wrapper over `cfs_component_v2_get`, not an endpoint binding
//!   of its own.
//! - `cfs_component_v2_get_parallel` is a chunking convenience
//!   wrapper over `cfs_component_v2_get_query`, not an endpoint
//!   binding of its own.
//! - `cfs_component_v2_get_query` adds a `limit=100000` parameter
//!   that the spec/generated client doesn't expose.
//! - `cfs_component_v2_put_component` takes the hand-written
//!   `Component`; the generated `put_component_v2` takes
//!   `&V2ComponentState` (different shape).
//! - `cfs_component_v2_put_component_list` is a sequential
//!   convenience wrapper over `cfs_component_v2_put_component`, not
//!   an endpoint binding of its own.
//! - `cfs_component_v2_delete_single_component` returns a
//!   `Component` body via the tolerant `handle_json_or_text_response`
//!   helper; the generated `delete_component_v2` is `Response = ()`
//!   on 204 only.

use std::time::Instant;

use crate::{
  ShastaClient,
  cfs::component::http_client::v2::types::Component,
  common::http,
  error::Error,
};

impl ShastaClient {
  /// Fetch CFS components, optionally filtered by a comma-separated
  /// `components_ids` list and/or a `status`.
  ///
  /// `GET /cfs/v2/components`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v2_get(
    &self,
    token: &str,
    components_ids: Option<&str>,
    status: Option<&str>,
  ) -> Result<Vec<Component>, Error> {
    log::debug!("Get CFS components");
    let api_url = format!("{}/cfs/v2/components", self.base_url());

    let response = self
      .http()
      .get(api_url)
      .query(&[("ids", components_ids), ("status", status)])
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  /// List every CFS component.
  ///
  /// Convenience wrapper for `cfs_component_v2_get(None, None)`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v2_get_all(
    &self,
    token: &str,
  ) -> Result<Vec<Component>, Error> {
    self.cfs_component_v2_get(token, None, None).await
  }

  /// Fetch one component by id.
  ///
  /// `GET /cfs/v2/components/{component_id}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v2_get_single_component(
    &self,
    token: &str,
    component_id: &str,
  ) -> Result<Component, Error> {
    let api_url =
      format!("{}/cfs/v2/components/{}", self.base_url(), component_id);

    let response = self
      .http()
      .get(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  /// Fetch CFS components for an arbitrarily large list of xnames by
  /// batching into requests of 60 ids and running up to 15 in flight at
  /// a time.
  ///
  /// Works around the CSM-side limit on how many ids a single
  /// `cfs_component_v2_get` request will accept. Order of returned
  /// components is not preserved.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v2_get_multiple(
    &self,
    token: &str,
    node_vec: &[String],
  ) -> Result<Vec<Component>, Error> {
    let start = Instant::now();

    let client = self.clone();
    let token = token.to_string();
    let component_vec = http::parallel_batch(node_vec, 60, 15, move |chunk| {
      let client = client.clone();
      let token = token.clone();
      async move {
        let ids = chunk.join(",");
        client.cfs_component_v2_get(&token, Some(&ids), None).await
      }
    })
    .await?;

    log::debug!(
      "Time elapsed to get CFS components is: {:?}",
      start.elapsed()
    );
    Ok(component_vec)
  }

  /// Same batching strategy as [`Self::cfs_component_v2_get_multiple`],
  /// but each batch goes through [`Self::cfs_component_v2_get_query`]
  /// so callers can also filter by configuration name / status.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v2_get_parallel(
    &self,
    token: &str,
    node_vec: &[String],
  ) -> Result<Vec<Component>, Error> {
    let start = Instant::now();

    let client = self.clone();
    let token = token.to_string();
    let component_vec = http::parallel_batch(node_vec, 60, 15, move |chunk| {
      let client = client.clone();
      let token = token.clone();
      async move {
        let ids = chunk.join(",");
        client
          .cfs_component_v2_get_query(&token, None, Some(&ids), None)
          .await
      }
    })
    .await?;

    let duration = start.elapsed();
    log::debug!("Time elapsed to get CFS components is: {duration:?}");

    Ok(component_vec)
  }

  /// Fetch CFS components with a richer filter set than
  /// [`Self::cfs_component_v2_get`].
  ///
  /// `GET /cfs/v2/components` with `config_name`, `ids`, and `status`
  /// query parameters.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v2_get_query(
    &self,
    token: &str,
    configuration_name: Option<&str>,
    components_ids: Option<&str>,
    status: Option<&str>,
  ) -> Result<Vec<Component>, Error> {
    let stupid_limit = 100000;

    let api_url = format!("{}/cfs/v2/components", self.base_url());

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

    http::handle_json_or_text_response(response).await
  }

  /// Replace one CFS component record.
  ///
  /// `PUT /cfs/v2/components/{component.id}`. Returns
  /// [`Error::CfsComponentFieldNotDefined`] if `component.id` is `None`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v2_put_component(
    &self,
    token: &str,
    component: Component,
  ) -> Result<Component, Error> {
    let component_id = component
      .id
      .as_deref()
      .ok_or_else(|| Error::CfsComponentFieldNotDefined("id".to_string()))?;
    let api_url =
      format!("{}/cfs/v2/components/{}", self.base_url(), component_id);
    http::put_json(self.http(), &api_url, token, &component).await
  }

  /// Replace many CFS component records sequentially. Stops at the
  /// first error (the partial results before that error are dropped).
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v2_put_component_list(
    &self,
    token: &str,
    component_list: Vec<Component>,
  ) -> Result<Vec<Component>, Error> {
    let mut result_vec: Vec<Result<Component, Error>> = Vec::new();

    for component in component_list {
      let result = self.cfs_component_v2_put_component(token, component).await;
      result_vec.push(result);
    }

    result_vec.into_iter().collect()
  }

  /// Delete a CFS component by id.
  ///
  /// `DELETE /cfs/v2/components/{component_id}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_component_v2_delete_single_component(
    &self,
    token: &str,
    component_id: &str,
  ) -> Result<Component, Error> {
    let api_url =
      format!("{}/cfs/v2/components/{}", self.base_url(), component_id);

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
