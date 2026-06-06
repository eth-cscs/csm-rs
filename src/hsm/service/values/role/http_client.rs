//! `ShastaClient` method for `/smd/hsm/v2/service/values/role`.

use crate::{ShastaClient, error::Error};

use super::types::Role;

impl ShastaClient {
  /// Get list of HSM Roles.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_roles_get(&self, token: &str) -> Result<Vec<String>, Error> {
    let api_url = format!("{}/smd/hsm/v2/service/values/role", self.base_url());

    self
      .http()
      .get(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?
      .json::<Role>()
      .await
      .map(|role| role.role)
      .map_err(Error::NetError)
  }
}
