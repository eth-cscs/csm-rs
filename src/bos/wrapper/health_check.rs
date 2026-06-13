//! Wrapper for `GET /v2/healthz`. Replaces `src/bos/health_check.rs`.
//!
//! Routing: progenitor `get_v2_healthz` + boundary conversion. The
//! generated method returns the typed spec shape
//! [`crate::bos::generated::types::Healthz`] (`apiStatus`, `dbStatus` —
//! both `Option<String>`). The public csm-rs API has always returned
//! `serde_json::Value` for this probe, so we convert at the wrapper
//! boundary to preserve backward compatibility for the few callers
//! that index into the result by string key.
//!
//! This is the first BOS method routed through the generated client;
//! it brings `gen_client` / `map_err` / `run` out of the dead-code
//! state for the BOS namespace.

use serde_json::Value;

use crate::{ShastaClient, error::Error};

use super::run;

impl ShastaClient {
  /// `GET /apis/bos/v2/healthz` — BOS liveness/readiness probe.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn bos_health_check(
    &self,
    token: &str,
  ) -> Result<Value, Error> {
    let typed =
      run(self, token, |c| async move { c.get_v2_healthz().await }).await?;
    serde_json::to_value(typed).map_err(Error::SerdeJsonError)
  }
}
