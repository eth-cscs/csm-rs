//! Wrapper for `/cfs/v2/configurations`. Replaces
//! `src/cfs/configuration/http_client/v2/mod.rs`.
//!
//! Routed through the progenitor-generated client:
//! - *(none)* — every generated v2 configuration method exchanges the
//!   strict `types::V2Configuration{,Array}` shape:
//!     * `name: Option<String>` (csm-rs's hand-written
//!       `CfsConfigurationResponse.name` is `String`),
//!     * `last_updated: Option<chrono::DateTime<chrono::offset::Utc>>`
//!       (hand-written is `String` so we never have to drag `chrono`
//!       into the public surface),
//!     * `layers: Vec<V2ConfigurationLayer>` where each layer wraps
//!       `clone_url`, `branch`, `commit`, `playbook` as **regex-validated
//!       newtypes** (`V2ConfigurationLayerCloneUrl(String)`, etc.) with
//!       `#[serde(deny_unknown_fields)]` on the struct,
//!     * `additional_inventory: Option<V2AdditionalInventoryLayer>` (a
//!       single layer; hand-written `CfsConfigurationResponse` keeps
//!       the older `additional_inventory: Option<AdditionalInventory>`
//!       shape with `cloneUrl`/`name`/`commit`/`branch` plain strings).
//!   csm-rs's public `CfsConfigurationRequest` and
//!   `CfsConfigurationResponse` are re-exported from `cfs::v2`,
//!   consumed by `configuration/utils.rs`, `dispatcher_conv.rs`
//!   (`From` impls between hand-written types and the dispatcher
//!   mirrors), the SAT-file parser
//!   (`CfsConfigurationRequest::from_sat_file_serde_yaml`), and the
//!   `manta-backend-dispatcher` trait impls. Adopting the generated
//!   types would force a structural change across all those consumers
//!   (and the public `cfs::v2::CfsConfiguration{Request,Response}`
//!   API) so this wave keeps everything on raw `reqwest`. A follow-up
//!   commit can migrate individual methods once a
//!   generated->hand-written conversion layer (or a swap of the public
//!   types to the generated ones) lands.
//!
//! Stays on raw `reqwest` because the generated surface doesn't
//! cover what the existing public API needs:
//!
//! - `cfs_configuration_v2_get` always sends `?limit=100000` and
//!   returns the hand-written `Vec<CfsConfigurationResponse>` shape.
//!   The generated `get_configurations_v2` takes only
//!   `in_use: Option<bool>` (no `limit`) and returns the strict
//!   `V2ConfigurationArray` — see the "routed via progenitor" section
//!   above for the shape mismatch. The single-name code path also
//!   wraps the response in a `Vec` of length 1 for uniform caller
//!   handling, which the generated `get_configuration_v2` (returns one
//!   `V2Configuration`) doesn't model.
//! - `cfs_configuration_v2_get_all` is a convenience wrapper over
//!   `cfs_configuration_v2_get(None)`, not an endpoint binding of its
//!   own.
//! - `cfs_configuration_v2_put` sends a hand-rolled
//!   `{ "layers": configuration.layers }` JSON object — it deliberately
//!   omits the `name` / `lastUpdated` / `additional_inventory` fields
//!   the spec would otherwise require. The generated `put_configuration_v2`
//!   takes a full `&V2Configuration` body (with the newtype-validated
//!   layers described above) and returns `V2Configuration`. Adopting
//!   it would change both the request shape (`CfsConfigurationRequest`
//!   would have to become `V2Configuration`) and the response shape
//!   (callers consume `CfsConfigurationResponse`).
//! - `cfs_configuration_v2_delete` returns `()`, which lines up with
//!   the generated `delete_configuration_v2` on its own. We still keep
//!   it on raw `reqwest` for now to avoid leaving a single
//!   progenitor-routed method dangling in a module whose other three
//!   methods are blocked on the contract mismatches above; routing
//!   just `delete` would force the wrapper to mix two error/transport
//!   paths for no behavioural gain. Migrating delete on its own is
//!   safe to revisit alongside the future swap of the public response
//!   type.

use crate::{
  ShastaClient,
  cfs::configuration::http_client::v2::types::{
    cfs_configuration_request::CfsConfigurationRequest,
    cfs_configuration_response::CfsConfigurationResponse,
  },
  common::http,
  error::Error,
};

const STUPID_LIMIT: i64 = 100000;

impl ShastaClient {
  /// Fetch one CFS configuration by name, or every configuration when
  /// `configuration_name_opt` is `None`.
  ///
  /// `GET /cfs/v2/configurations[/{name}]`. Always returns a `Vec` for
  /// uniform handling at the call site — single-name lookups produce a
  /// one-element vector.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_configuration_v2_get(
    &self,
    token: &str,
    configuration_name_opt: Option<&str>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    log::debug!(
      "Get CFS configuration '{}'",
      configuration_name_opt.unwrap_or("all available")
    );

    let api_url = if let Some(name) = configuration_name_opt {
      format!("{}/cfs/v2/configurations/{}", self.base_url(), name)
    } else {
      format!("{}/cfs/v2/configurations", self.base_url())
    };

    if configuration_name_opt.is_some() {
      let payload: CfsConfigurationResponse = http::get_json_with_query(
        self.http(),
        &api_url,
        token,
        &[("limit", STUPID_LIMIT)],
      )
      .await?;
      Ok(vec![payload])
    } else {
      http::get_json_with_query(
        self.http(),
        &api_url,
        token,
        &[("limit", STUPID_LIMIT)],
      )
      .await
    }
  }

  /// List every CFS configuration on the system.
  ///
  /// Convenience wrapper for `cfs_configuration_v2_get(None)`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_configuration_v2_get_all(
    &self,
    token: &str,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    self.cfs_configuration_v2_get(token, None).await
  }

  /// Create or replace a CFS configuration by name with the supplied
  /// layer list.
  ///
  /// `PUT /cfs/v2/configurations/{configuration_name}`. The request body
  /// is `{ "layers": configuration.layers }`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_configuration_v2_put(
    &self,
    token: &str,
    configuration: &CfsConfigurationRequest,
    configuration_name: &str,
  ) -> Result<CfsConfigurationResponse, Error> {
    log::debug!("Create CFS configuration '{configuration_name}'");
    log::debug!("Create CFS configuration request:\n{configuration:#?}");

    let api_url = format!(
      "{}/cfs/v2/configurations/{}",
      self.base_url(),
      configuration_name
    );

    let request_payload = serde_json::json!({ "layers": configuration.layers });

    log::debug!(
      "CFS configuration request payload:\n{}",
      serde_json::to_string_pretty(&request_payload)
        .unwrap_or_else(|e| format!("<serialize error: {e}>"))
    );

    http::put_json(self.http(), &api_url, token, &request_payload).await
  }

  /// Delete a CFS configuration by id.
  ///
  /// `DELETE /cfs/v2/configurations/{configuration_id}`. CFS rejects
  /// the delete if the configuration is still referenced by an image
  /// or runtime binding; that surfaces as an HTTP error.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_configuration_v2_delete(
    &self,
    token: &str,
    configuration_id: &str,
  ) -> Result<(), Error> {
    log::debug!("Delete CFS configuration {configuration_id:?}");

    let api_url = format!(
      "{}/cfs/v2/configurations/{}",
      self.base_url(),
      configuration_id
    );
    http::delete(self.http(), &api_url, token).await
  }
}
