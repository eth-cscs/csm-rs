//! `ShastaClient` method for `/smd/hsm/v2/service/values/role`.

use crate::{ShastaClient, error::Error};

use super::types::Role;

impl ShastaClient {
  /// Get list of HSM Roles.
  pub async fn hsm_roles_get(&self) -> Result<Vec<String>, Error> {
    let api_url = format!("{}/smd/hsm/v2/service/values/role", self.base_url());

    self
      .http()
      .get(api_url)
      .bearer_auth(self.token())
      .send()
      .await
      .map_err(Error::NetError)?
      .json::<Role>()
      .await
      .map(|role| role.role)
      .map_err(Error::NetError)
  }
}
