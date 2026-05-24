pub mod types;

use std::{sync::Arc, time::Instant};

use serde_json::Value;
use tokio::sync::Semaphore;
use types::ComponentVec;

use crate::{
  ShastaClient, cfs::component::http_client::v3::types::Component,
  common::http, error::Error,
};

impl ShastaClient {
  /// Get CFS options
  /// Retutns a JSON object with the options available in the CFS API
  pub async fn cfs_component_v3_get_options(&self) -> Result<Value, Error> {
    let api_url = format!("{}/cfs/v3/options", self.base_url());

    let response = self
      .http()
      .get(api_url)
      .bearer_auth(self.token())
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  pub async fn cfs_component_v3_get(
    &self,
    components_ids: Option<&str>,
    status: Option<&str>,
  ) -> Result<Vec<Component>, Error> {
    let api_url = format!("{}/cfs/v3/components", self.base_url());

    let response = self
      .http()
      .get(api_url)
      .query(&[("ids", components_ids), ("status", status)])
      .bearer_auth(self.token())
      .send()
      .await
      .map_err(Error::NetError)?;

    let payload: ComponentVec =
      http::handle_json_or_text_response(response).await?;
    Ok(payload.components)
  }

  pub async fn cfs_component_v3_get_single_by_id(
    &self,
    component_id: &str,
  ) -> Result<Component, Error> {
    let api_url =
      format!("{}/cfs/v3/components/{}", self.base_url(), component_id);

    let response = self
      .http()
      .get(api_url)
      .bearer_auth(self.token())
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  /// Get components data.
  /// Currently, CSM will throw an error if many xnames are sent in the request, therefore, this
  /// method will paralelize multiple calls, each with a batch of xnames
  pub async fn cfs_component_v3_get_parallel(
    &self,
    node_vec: &[String],
  ) -> Result<Vec<Component>, Error> {
    let start = Instant::now();

    let num_xnames_per_request = 60;
    let pipe_size = 15;

    log::debug!(
      "Number of nodes per request: {num_xnames_per_request}; Pipe size (semaphore): {pipe_size}"
    );

    let mut component_vec = Vec::new();
    let mut tasks = tokio::task::JoinSet::new();
    let sem = Arc::new(Semaphore::new(pipe_size));
    let num_requests = (node_vec.len() / num_xnames_per_request) + 1;
    let mut i = 1;
    let width = num_requests.checked_ilog10().unwrap_or(0) as usize + 1;

    for sub_node_list in node_vec.chunks(num_xnames_per_request) {
      let num_nodes_in_flight = sub_node_list.len();
      log::info!(
        "Getting CFS components: processing batch [{i:>width$}/{num_requests}] (batch size - {num_nodes_in_flight})"
      );

      let hsm_subgroup_nodes_string: String = sub_node_list.join(",");
      let client = self.clone();

      let permit = sem
        .clone()
        .acquire_owned()
        .await
        .expect("semaphore not closed");

      tasks.spawn(async move {
        let _permit = permit;
        client
          .cfs_component_v3_get_query(
            None,
            Some(&hsm_subgroup_nodes_string),
            None,
          )
          .await
      });

      i += 1;
    }

    while let Some(message) = tasks.join_next().await {
      component_vec.append(&mut message??);
    }

    let duration = start.elapsed();
    log::info!("Time elapsed to get CFS components is: {:?}", duration);

    Ok(component_vec)
  }

  pub async fn cfs_component_v3_get_query(
    &self,
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
      .bearer_auth(self.token())
      .send()
      .await
      .map_err(Error::NetError)?;

    let payload: ComponentVec =
      http::handle_json_or_text_response(response).await?;
    Ok(payload.components)
  }

  pub async fn cfs_component_v3_patch_component(
    &self,
    component: Component,
  ) -> Result<Vec<Value>, Error> {
    let component_id = component.id.as_deref().ok_or_else(|| {
      Error::CfsComponentFieldNotDefined("id".to_string())
    })?;
    let api_url =
      format!("{}/cfs/v3/components/{}", self.base_url(), component_id);

    let response = self
      .http()
      .patch(api_url)
      .bearer_auth(self.token())
      .json(&component)
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  pub async fn cfs_component_v3_patch_component_list(
    &self,
    component_list: Vec<Component>,
  ) -> Result<(), Error> {
    let api_url = format!("{}/cfs/v3/components", self.base_url());

    let response = self
      .http()
      .patch(api_url)
      .bearer_auth(self.token())
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

  pub async fn cfs_component_v3_put_component(
    &self,
    component: Component,
  ) -> Result<Component, Error> {
    let component_id = component.id.as_deref().ok_or_else(|| {
      Error::CfsComponentFieldNotDefined("id".to_string())
    })?;
    let api_url =
      format!("{}/cfs/v3/components/{}", self.base_url(), component_id);
    http::put_json(self.http(), &api_url, self.token(), &component).await
  }

  pub async fn cfs_component_v3_put_component_list(
    &self,
    component_list: Vec<Component>,
  ) -> Result<Vec<Component>, Error> {
    let mut result_vec: Vec<Result<Component, Error>> = Vec::new();

    for component in component_list {
      let result = self.cfs_component_v3_put_component(component).await;
      result_vec.push(result);
    }

    result_vec.into_iter().collect()
  }

  pub async fn cfs_component_v3_delete_single_component(
    &self,
    component_id: &str,
  ) -> Result<Component, Error> {
    let api_url =
      format!("{}/cfs/v3/components/{}", self.base_url(), component_id);

    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(self.token())
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }
}
