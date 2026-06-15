//! Wrapper for `/cfs/v3/configurations`. Replaces
//! `src/cfs/configuration/http_client/v3/mod.rs`.
//!
//! Routed through the progenitor-generated client:
//! - *(none)* — every generated v3 configuration method exchanges the
//!   strict `types::V3ConfigurationData{,Collection}` shape:
//!     * `name: Option<String>` (csm-rs's hand-written
//!       `CfsConfigurationResponse.name` is `String`),
//!     * `last_updated: Option<chrono::DateTime<chrono::offset::Utc>>`
//!       (hand-written is `String` so we never have to drag `chrono`
//!       into the public surface),
//!     * `layers: Vec<V3ConfigurationLayer>` where each layer wraps
//!       `clone_url`, `branch`, `commit`, `playbook` as **regex-validated
//!       newtypes** (`V3ConfigurationLayerCloneUrl(String)`, etc.) with
//!       `#[serde(deny_unknown_fields)]` on the struct,
//!     * `additional_inventory: Option<V3AdditionalInventoryLayer>` (a
//!       single layer carrying its own newtype-validated fields;
//!       hand-written `CfsConfigurationResponse` keeps the older
//!       `additional_inventory: Option<AdditionalInventory>` shape with
//!       `cloneUrl` / `name` / `commit` / `branch` plain strings),
//!     * an extra `tenant_name: Option<String>` field the hand-written
//!       response doesn't carry.
//!   csm-rs's public `CfsConfigurationRequest` and
//!   `CfsConfigurationResponse` are re-exported from `cfs::v3`,
//!   consumed by `configuration/utils.rs`, `dispatcher_conv.rs`
//!   (`From` impls between hand-written types and the dispatcher
//!   mirrors), the SAT-file parser
//!   (`CfsConfigurationRequest::from_sat_file_serde_yaml` /
//!   `create_from_repos` — the latter is called directly from
//!   `backend_connector/cfs.rs`), and the `manta-backend-dispatcher`
//!   trait impls. Adopting the generated types would force a
//!   structural change across all those consumers (and the public
//!   `cfs::v3::CfsConfiguration{Request,Response}` API) so this wave
//!   keeps everything on raw `reqwest`. A follow-up commit can migrate
//!   individual methods once a generated->hand-written conversion layer
//!   (or a swap of the public types to the generated ones) lands.
//!
//! Stays on raw `reqwest` because the generated surface doesn't
//! cover what the existing public API needs:
//!
//! - `cfs_configuration_v3_get` always sends `?limit=100000` and
//!   returns the hand-written `Vec<CfsConfigurationResponse>` shape
//!   (single-name lookups are wrapped in a one-element `Vec` for
//!   uniform caller handling). The generated `get_configurations_v3`
//!   takes `limit: Option<std::num::NonZeroU64>` plus `after_id` /
//!   `in_use` / `cray_tenant_name` and returns the strict
//!   `V3ConfigurationDataCollection` — see the "routed via progenitor"
//!   section above for the shape mismatch. The single-name code path
//!   uses `get_configuration_v3` which returns one `V3ConfigurationData`,
//!   not modelled by the public `Vec<_>` signature.
//! - `cfs_configuration_v3_put` first calls
//!   `cfs_configuration_v3_get(Some(name))` to refuse overwriting an
//!   existing configuration (returning `Error::ConfigurationAlreadyExists`)
//!   and then sends a hand-rolled `{ "layers": configuration.layers }`
//!   JSON object — deliberately omitting `name` / `last_updated` /
//!   `additional_inventory` / `tenant_name`. The generated
//!   `put_configuration_v3` takes a full `&V3ConfigurationData` body
//!   (with the newtype-validated layers described above) and returns
//!   `V3ConfigurationData`. Adopting it would change both the request
//!   shape (`CfsConfigurationRequest` would have to become
//!   `V3ConfigurationData`) and the response shape (callers consume
//!   `CfsConfigurationResponse`), and would also lose the
//!   refuse-overwrite check that's part of the public contract.
//! - `cfs_configuration_v3_delete` returns `()`, which lines up with
//!   the generated `delete_configuration_v3` on its own (both 204-only).
//!   We still keep it on raw `reqwest` for now to avoid leaving a
//!   single progenitor-routed method dangling in a module whose other
//!   two methods are blocked on the contract mismatches above; routing
//!   just `delete` would force the wrapper to mix two error/transport
//!   paths for no behavioural gain. Migrating delete on its own is
//!   safe to revisit alongside the future swap of the public response
//!   type.

use crate::{
  ShastaClient,
  cfs::configuration::http_client::v3::types::{
    cfs_configuration_request::CfsConfigurationRequest,
    cfs_configuration_response::{
      CfsConfigurationResponse, CfsConfigurationVecResponse,
    },
  },
  common::http,
  error::Error,
};

const STUPID_LIMIT: i64 = 100000;

impl ShastaClient {
  /// Fetch one CFS configuration by name, or every configuration when
  /// `configuration_name_opt` is `None`, using the v3 API.
  ///
  /// `GET /cfs/v3/configurations[/{name}]`. CFS v3 returns plain-text
  /// error bodies and a different success shape for single vs. list
  /// lookups; both are normalised here.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_configuration_v3_get(
    &self,
    token: &str,
    configuration_name_opt: Option<&str>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    log::debug!("Get CFS configuration {configuration_name_opt:?}");

    let api_url = if let Some(name) = configuration_name_opt {
      format!("{}/cfs/v3/configurations/{}", self.base_url(), name)
    } else {
      format!("{}/cfs/v3/configurations", self.base_url())
    };

    let response = self
      .http()
      .get(api_url)
      .query(&[("limit", STUPID_LIMIT)])
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    // CFS v3 returns plain-text errors on failure (not JSON), and a different
    // success shape depending on whether a single config was requested.
    if configuration_name_opt.is_some() {
      let payload: CfsConfigurationResponse =
        http::handle_json_or_text_response(response).await?;
      Ok(vec![payload])
    } else {
      let payload: CfsConfigurationVecResponse =
        http::handle_json_or_text_response(response).await?;
      Ok(payload.configurations)
    }
  }

  /// Create a CFS configuration by name, refusing to overwrite if one
  /// already exists.
  ///
  /// `PUT /cfs/v3/configurations/{configuration_name}`. Unlike a bare
  /// `PUT`, this checks first via [`Self::cfs_configuration_v3_get`]
  /// and returns [`Error::Message`] if a configuration with the same
  /// name is already present.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_configuration_v3_put(
    &self,
    token: &str,
    configuration: &CfsConfigurationRequest,
    configuration_name: &str,
  ) -> Result<CfsConfigurationResponse, Error> {
    // Check if CFS configuration already exists
    log::debug!("Check CFS configuration '{configuration_name}' exists");

    let cfs_configuration_rslt = self
      .cfs_configuration_v3_get(token, Some(configuration_name))
      .await;

    if cfs_configuration_rslt
      .is_ok_and(|cfs_configuration_vec| !cfs_configuration_vec.is_empty())
    {
      return Err(Error::ConfigurationAlreadyExists(
        configuration_name.to_string(),
      ));
    }

    log::debug!(
      "CFS configuration '{configuration_name}' does not exists, creating new CFS configuration"
    );

    log::debug!("Create CFS configuration '{configuration_name}'");
    log::debug!("Create CFS configuration request:\n{configuration:#?}");

    let api_url = format!(
      "{}/cfs/v3/configurations/{}",
      self.base_url(),
      configuration_name
    );

    let request_payload = serde_json::json!({ "layers": configuration.layers });
    log::debug!(
      "CFS configuration request payload:\n{}",
      serde_json::to_string_pretty(&request_payload)
        .unwrap_or_else(|e| format!("<serialize error: {e}>"))
    );

    let response = self
      .http()
      .put(api_url)
      .json(&request_payload)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  /// Delete a CFS configuration by id via the v3 API.
  ///
  /// `DELETE /cfs/v3/configurations/{configuration_id}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn cfs_configuration_v3_delete(
    &self,
    token: &str,
    configuration_id: &str,
  ) -> Result<(), Error> {
    log::debug!("Delete CFS configuration '{configuration_id}'");

    let api_url = format!(
      "{}/cfs/v3/configurations/{}",
      self.base_url(),
      configuration_id
    );
    http::delete(self.http(), &api_url, token).await
  }
}
