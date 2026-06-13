//! Wrapper for `/memberships`. Replaces `src/hsm/memberships/http_client.rs`.
//!
//! Maintains the historical public method names (`hsm_memberships_get_all`,
//! `hsm_memberships_get_xname`) that callers (e.g. `src/node/utils.rs`) and
//! the integration tests in `tests/shasta_client_hsm.rs` rely on. The
//! generated `do_memberships_get` returns `Vec<Membership100>` straight
//! (no wrapper struct), so the `run` adapter unwraps directly to the
//! public return type `Vec<Membership>` via the
//! `pub use Membership100 as Membership` alias in `types.rs`.
//!
//! `do_memberships_get` takes 16 optional query-parameter filters; the
//! historical API took none, so all 16 are passed as `None` to preserve
//! the "get every membership record" semantics.

use crate::{ShastaClient, error::Error, hsm::memberships::types::Membership};

use super::run;

impl ShastaClient {
  /// `GET /smd/hsm/v2/memberships` — every membership record HSM knows.
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
    run(self, token, |c| async move {
      c.do_memberships_get(
        None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None,
      )
      .await
    })
    .await
  }

  /// `GET /smd/hsm/v2/memberships/{xname}` — membership record for a
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
    run(self, token, |c| async move {
      c.do_membership_get(xname).await
    })
    .await
  }
}
