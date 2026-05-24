use serde_json::json;

use crate::{ShastaClient, common::http, error::Error};

use super::types::PowerStatusAll;

impl ShastaClient {
  pub async fn pcs_power_status_post(
    &self,
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

    http::post_json(self.http(), &url, self.token(), &body).await
  }
}
