//! CFS components v2 — `ShastaClient` methods for `/cfs/v2/components`.

pub(crate) mod types;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// CFS v2 component mirror types. Gated behind the `manta-dispatcher`
/// Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;

use std::time::Instant;

use types::Component;

use crate::{ShastaClient, common::http, error::Error};

impl ShastaClient {
  /// Fetch CFS components, optionally filtered by a comma-separated
  /// `components_ids` list and/or a `status`.
  ///
  /// `GET /cfs/v2/components`.
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
  pub async fn cfs_component_v2_get_all(
    &self,
    token: &str,
  ) -> Result<Vec<Component>, Error> {
    self.cfs_component_v2_get(token, None, None).await
  }

  /// Fetch one component by id.
  ///
  /// `GET /cfs/v2/components/{component_id}`.
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
    log::debug!("Time elapsed to get CFS components is: {:?}", duration);

    Ok(component_vec)
  }

  /// Fetch CFS components with a richer filter set than
  /// [`Self::cfs_component_v2_get`].
  ///
  /// `GET /cfs/v2/components` with `config_name`, `ids`, and `status`
  /// query parameters.
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
