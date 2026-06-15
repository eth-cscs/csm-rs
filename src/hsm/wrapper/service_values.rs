//! Wrapper for `/service/values/role`. Replaces
//! `src/hsm/service/values/role/http_client.rs`.
//!
//! Maintains the historical public method name (`hsm_roles_get`) and
//! return type (`Vec<String>`) — callers and the integration test in
//! `tests/shasta_client_hsm.rs` rely on both. The generated method
//! returns `Values100RoleArray { role: Vec<HmsRole100> }`, which we
//! flatten to the simpler `Vec<String>` here.

use crate::{ShastaClient, error::Error};

use super::run;

impl ShastaClient {
  /// `GET /smd/hsm/v2/service/values/role` — list known component roles.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_roles_get(
    &self,
    token: &str,
  ) -> Result<Vec<String>, Error> {
    let arr = run(self, token, |c| async move {
      c.do_role_values_get().await
    })
    .await?;
    Ok(arr.role.into_iter().map(|r| r.0).collect())
  }
}
