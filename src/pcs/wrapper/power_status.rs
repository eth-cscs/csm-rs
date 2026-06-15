//! Wrapper for PCS `power-status` endpoint. Replaces
//! `src/pcs/power_status/http_client.rs`.
//!
//! Routing: **stays on raw `reqwest`**. The generated
//! `Client::post_power_status` takes
//! [`crate::pcs::generated::types::PowerStatusGet`] and returns
//! [`crate::pcs::generated::types::PowerStatusAll`] / `PowerStatus`,
//! whose shapes differ from the hand-written types
//! re-exported at `crate::pcs::power_status::{PowerStatus,
//! PowerStatusAll, PowerState, ManagementState}`:
//!
//! - Generated `PowerStatus.xname: Option<types::Xname>` (regex-validated
//!   newtype) vs hand-written `PowerStatus.xname: String` (plain, always
//!   present).
//! - Generated `PowerStatus.last_updated:
//!   Option<chrono::DateTime<chrono::offset::Utc>>` vs hand-written
//!   `PowerStatus.last_updated: String` (raw RFC 3339 from the wire).
//! - Generated `PowerStatus.supported_power_transitions: Vec<PowerOperation>`
//!   (PCS-local enum) vs hand-written
//!   `supported_power_transitions: Vec<crate::pcs::transitions::types::Operation>`
//!   (csm-rs shares the transitions `Operation` enum across PCS).
//! - The `POST` body differs too: hand-written builds an inline
//!   `serde_json::json!` payload with default-empty-string sentinels for
//!   missing filters, while `PowerStatusGet` uses `Option<ManagementState>`
//!   / `Option<PowerState>` and an `xname` field typed as the orphan
//!   `NonEmptyStringList` newtype (see F.3 in the PCS output reference
//!   doc) — adopting it would force the public method to either accept
//!   that newtype or carry a fresh validation error path.
//!
//! `PowerStatus`, `PowerStatusAll`, `PowerState`, and `ManagementState`
//! are also re-exported as the canonical names at
//! `crate::pcs::power_status::*` and the 96-line
//! `pcs::power_status::dispatcher_conv` module mirrors all four into the
//! `manta_backend_dispatcher::types::pcs::power_status::types::*` peers
//! field-for-field. Routing through progenitor would either change the
//! public type (rippling through dispatcher_conv and the backend
//! dispatcher trait impls) or require an extra conversion layer at the
//! wrapper boundary just to unmake what progenitor did. Neither carries
//! its weight for a single endpoint.
//!
//! The `gen_client` / `map_err` / `run` helpers in
//! `crate::pcs::wrapper` are retained so a future spec revision (or a
//! decision to swap the public types for the generated newtype-bearing
//! shapes) can migrate this method incrementally without a second
//! scaffolding pass.

use serde_json::json;

use crate::{ShastaClient, common::http, error::Error};

use crate::pcs::power_status::types::PowerStatusAll;

impl ShastaClient {
  /// Query power status for a set of components, optionally filtering
  /// by power state and management state.
  ///
  /// `POST /power-control/v1/power-status`. When `xname_vec_opt` is
  /// `None`, every known component is queried; the two filter arguments
  /// default to the empty string (no filter) when `None`.
  ///
  /// # Arguments
  ///
  /// - `xname_vec_opt` — restrict the query to these component xnames.
  /// - `power_state_filter_opt` — e.g. `"on"`, `"off"`, `"undefined"`.
  /// - `management_state_filter_opt` — e.g. `"available"`,
  ///   `"unavailable"`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn pcs_power_status_post(
    &self,
    token: &str,
    xname_vec_opt: Option<&[&str]>,
    power_state_filter_opt: Option<&str>,
    management_state_filter_opt: Option<&str>,
  ) -> Result<PowerStatusAll, Error> {
    let url = format!("{}/power-control/v1/power-status", self.base_url());

    let body = json!({
      "xname": xname_vec_opt
        .map(|v| v.iter().map(|&x| x.to_string()).collect::<Vec<String>>())
        .unwrap_or_default(),
      "powerStateFilter": power_state_filter_opt.unwrap_or(""),
      "managementStateFilter": management_state_filter_opt.unwrap_or(""),
    });

    http::post_json(self.http(), &url, token, &body).await
  }
}
