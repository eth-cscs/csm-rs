//! CFS components v3 — `ShastaClient` methods for `/cfs/v3/components`.

pub(crate) mod types;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// CFS v3 component mirror types. Gated behind the `manta-dispatcher`
/// Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;

use std::time::Instant;

use serde_json::Value;
use types::ComponentVec;

use crate::{
  ShastaClient, cfs::component::http_client::v3::types::Component,
  common::http, error::Error,
};

impl ShastaClient {
  /// Fetch CFS options.
  ///
  /// `GET /cfs/v3/options`. Returns the raw JSON object of CFS
  /// service-level options.
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
  pub async fn cfs_component_v3_get_parallel(
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
          .cfs_component_v3_get_query(&token, None, Some(&ids), None)
          .await
      }
    })
    .await?;

    log::info!(
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
  pub async fn cfs_component_v3_put_component_list(
    &self,
    token: &str,
    component_list: Vec<Component>,
  ) -> Result<Vec<Component>, Error> {
    let mut result_vec: Vec<Result<Component, Error>> = Vec::new();

    for component in component_list {
      let result = self.cfs_component_v3_put_component(token, component).await;
      result_vec.push(result);
    }

    result_vec.into_iter().collect()
  }

  /// Delete a CFS component by id.
  ///
  /// `DELETE /cfs/v3/components/{component_id}`.
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
