use crate::{ShastaClient, common::http, error::Error};

use super::types::{PowerCapComponent, PowerCapTaskInfo};

impl ShastaClient {
  pub async fn pcs_power_cap_get(&self) -> Result<PowerCapTaskInfo, Error> {
    let url = format!("{}/power-control/v1/power-cap", self.base_url());
    http::get_json(self.http(), &url, self.token()).await
  }

  pub async fn pcs_power_cap_get_task_id(
    &self,
    task_id: &str,
  ) -> Result<PowerCapTaskInfo, Error> {
    let url = format!(
      "{}/power-control/v1/power-cap/{}",
      self.base_url(),
      task_id
    );
    http::get_json(self.http(), &url, self.token()).await
  }

  pub async fn pcs_power_cap_post_snapshot(
    &self,
    xname_vec: Vec<&str>,
  ) -> Result<PowerCapTaskInfo, Error> {
    log::info!("Create PCS power snapshot for nodes:\n{:?}", xname_vec);
    log::debug!("Create PCS power snapshot for nodes:\n{:?}", xname_vec);

    let url =
      format!("{}/power-control/v1/power-cap/snapshot", self.base_url());
    let body = serde_json::json!({ "xnames": xname_vec });
    http::put_json(self.http(), &url, self.token(), &body).await
  }

  pub async fn pcs_power_cap_patch(
    &self,
    power_cap: Vec<PowerCapComponent>,
  ) -> Result<PowerCapTaskInfo, Error> {
    log::info!("Create PCS power cap:\n{:#?}", power_cap);
    log::debug!("Create PCS power cap:\n{:#?}", power_cap);

    let url =
      format!("{}/power-control/v1/power-cap/snapshot", self.base_url());
    http::put_json(self.http(), &url, self.token(), &power_cap).await
  }
}
