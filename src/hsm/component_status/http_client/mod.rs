use reqwest::Url;
use serde_json::Value;

use crate::{ShastaClient, common::http, error::Error};

impl ShastaClient {
  pub async fn hsm_component_status_get_raw(
    &self,
    xname_vec: &[String],
  ) -> Result<Vec<Value>, Error> {
    let url_params: Vec<_> =
      xname_vec.iter().map(|xname| ("id", xname)).collect();

    let api_url = Url::parse_with_params(
      &format!("{}/smd/hsm/v2/State/Components", self.base_url()),
      &url_params,
    )
    .map_err(|e| {
      Error::Message(format!(
        "Could not build HSM components URL from base '{}': {}",
        self.base_url(),
        e
      ))
    })?;

    let response: Value =
      http::get_json(self.http(), api_url.as_str(), self.token()).await?;

    Ok(
      response
        .get("Components")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default(),
    )
  }

  /// Fetches nodes/components details using HSM v2.
  /// Splits xname_vec into chunks of 30 (CSM rejects larger requests) and
  /// parallelises across `tokio::spawn`.
  pub async fn hsm_component_status_get(
    &self,
    xname_vec: &[String],
  ) -> Result<Vec<Value>, Error> {
    let chunk_size = 30;
    let mut hsm_component_status_vec: Vec<Value> = Vec::new();
    let mut tasks = tokio::task::JoinSet::new();

    for sub_node_list in xname_vec.chunks(chunk_size) {
      let client = self.clone();
      let node_vec = sub_node_list.to_vec();
      tasks
        .spawn(async move { client.hsm_component_status_get_raw(&node_vec).await });
    }

    while let Some(message) = tasks.join_next().await {
      hsm_component_status_vec.append(&mut message??);
    }

    Ok(hsm_component_status_vec)
  }
}
