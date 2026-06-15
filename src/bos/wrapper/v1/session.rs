//! Wrapper for `/bos/v1/session`. Replaces
//! `src/bos/session/http_client/v1/mod.rs`.
//!
//! The upstream BOS spec is v2-only — there is no progenitor coverage
//! for v1, so this is a pure file relocation. All methods stay on raw
//! `reqwest`.
//!
//! Methods present:
//! - `bos_session_v1_post`

use serde_json::{Value, json};

use crate::{ShastaClient, common::http, error::Error};

impl ShastaClient {
  /// `POST /bos/v1/session` — create a v1 BOS session for the given
  /// template name and operation (e.g. `boot`, `reboot`, `shutdown`).
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bos_session_v1_post(
    &self,
    token: &str,
    bos_template_name: &str,
    operation: &str,
  ) -> Result<Value, Error> {
    let payload = json!({
      "operation": operation,
      "templateName": bos_template_name,
    });

    log::debug!("Create BOS session v1");
    log::debug!("Create BOS session v1 payload:\n{payload:#?}");

    let url = format!("{}/bos/v1/session", self.base_url());
    http::post_json(self.http(), &url, token, &payload).await
  }
}
