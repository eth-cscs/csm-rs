use serde_json::json;

use crate::{ShastaClient, common::http, error::Error};

use super::types::PowerStatusAll;

impl ShastaClient {
  /// Query power status for a set of components, optionally filtering
  /// by power state and management state.
  ///
  /// `POST /power-control/v1/power-status`. When `xname_vec_opt` is
  /// `None`, every known component is queried; the two filter arguments
  /// default to the empty string (no filter) when `None`.
  ///
  /// # Arguments
  ///
  /// - `xname_vec_opt` — restrict the query to these component xnames.
  /// - `power_state_filter_opt` — e.g. `"on"`, `"off"`, `"undefined"`.
  /// - `management_state_filter_opt` — e.g. `"available"`,
  ///   `"unavailable"`.
  pub async fn pcs_power_status_post(
    &self,
    token: &str,
    xname_vec_opt: Option<&[&str]>,
    power_state_filter_opt: Option<&str>,
    management_state_filter_opt: Option<&str>,
  ) -> Result<PowerStatusAll, Error> {
    let url = format!("{}/power-control/v1/power-status", self.base_url());

    let body = json!({
      "xname": xname_vec_opt
        .map(|v| v.iter().map(|&x| x.to_string()).collect::<Vec<String>>())
        .unwrap_or_default(),
      "powerStateFilter": power_state_filter_opt.unwrap_or(""),
      "managementStateFilter": management_state_filter_opt.unwrap_or(""),
    });

    http::post_json(self.http(), &url, token, &body).await
  }
}
