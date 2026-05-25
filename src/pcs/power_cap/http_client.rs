use crate::{ShastaClient, common::http, error::Error};

use super::types::{PowerCapComponent, PowerCapTaskInfo};

impl ShastaClient {
  /// List all power-cap tasks known to PCS.
  ///
  /// `GET /power-control/v1/power-cap`.
  pub async fn pcs_power_cap_get(&self) -> Result<PowerCapTaskInfo, Error> {
    let url = format!("{}/power-control/v1/power-cap", self.base_url());
    http::get_json(self.http(), &url, self.token()).await
  }

  /// Fetch a single power-cap task by its `task_id`.
  ///
  /// `GET /power-control/v1/power-cap/{task_id}`.
  pub async fn pcs_power_cap_get_task_id(
    &self,
    task_id: &str,
  ) -> Result<PowerCapTaskInfo, Error> {
    let url =
      format!("{}/power-control/v1/power-cap/{}", self.base_url(), task_id);
    http::get_json(self.http(), &url, self.token()).await
  }

  /// Capture a power-cap snapshot for the given component xnames.
  ///
  /// `PUT /power-control/v1/power-cap/snapshot` with `{"xnames": [...]}`.
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

  /// Apply a set of power-cap values to the given components.
  ///
  /// `PUT /power-control/v1/power-cap/snapshot` with the supplied
  /// per-component cap definitions.
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
