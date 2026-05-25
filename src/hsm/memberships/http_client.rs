//! `ShastaClient` methods for `/smd/hsm/v2/memberships`.

use crate::{ShastaClient, common::http, error::Error};

use super::types::Membership;

impl ShastaClient {
  pub async fn hsm_memberships_get_all(
    &self,
  ) -> Result<Vec<Membership>, Error> {
    let url = format!("{}/smd/hsm/v2/memberships", self.base_url());
    http::get_json(self.http(), &url, self.token()).await
  }

  pub async fn hsm_memberships_get_xname(
    &self,
    xname: &str,
  ) -> Result<Membership, Error> {
    log::debug!("Get membership of node '{}'", xname);
    let url = format!("{}/smd/hsm/v2/memberships/{}", self.base_url(), xname);
    http::get_json(self.http(), &url, self.token()).await
  }
}
