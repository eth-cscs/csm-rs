//! `ShastaClient` methods for `/smd/hsm/v2/memberships`.

use crate::{ShastaClient, common::http, error::Error};

use super::types::Membership;

impl ShastaClient {
  /// `GET /hsm/v2/memberships` — every membership record HSM knows.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_memberships_get_all(
    &self,
    token: &str,
  ) -> Result<Vec<Membership>, Error> {
    let url = format!("{}/smd/hsm/v2/memberships", self.base_url());
    http::get_json(self.http(), &url, token).await
  }

  /// `GET /hsm/v2/memberships/{xname}` — membership record for a
  /// single component.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_memberships_get_xname(
    &self,
    token: &str,
    xname: &str,
  ) -> Result<Membership, Error> {
    log::debug!("Get membership of node '{xname}'");
    let url = format!("{}/smd/hsm/v2/memberships/{}", self.base_url(), xname);
    http::get_json(self.http(), &url, token).await
  }
}
