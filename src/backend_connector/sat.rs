//! `SatTrait`, `ApplyHwClusterPin` impls for [`Csm`](super::Csm).
//!
//! `apply_sat_file` is a thin shim over
//! [`crate::commands::i_apply_sat_file::command::exec`]: it destructures
//! the [`ApplySatFileParams`] bag, transcodes the structured SAT value
//! from `serde_json::Value` to `serde_yaml::Value` (lossless for any
//! valid SAT file), fetches the Kubernetes secrets from Vault
//! (consistent with the `console`, `cfs`, and `cfs::session` backend
//! paths), forwards every other argument unchanged, and then maps the
//! per-crate `CfsConfigurationResponse` / `Image` /
//! `BosSessionTemplate` / `BosSession` types into the corresponding
//! `manta_backend_dispatcher::types::*` types via the existing `From`
//! impls so the returned tuple satisfies the trait.

use manta_backend_dispatcher::{
  error::Error,
  interfaces::{
    apply_hw_cluster_pin::ApplyHwClusterPin,
    apply_sat_file::{ApplySatFileParams, SatTrait},
  },
  types::{
    bos::{session::BosSession, session_template::BosSessionTemplate},
    cfs::cfs_configuration_response::CfsConfigurationResponse,
    ims::Image,
  },
};

use super::Csm;
use crate::common::vault::http_client::fetch_shasta_k8s_secrets_from_vault;

impl SatTrait for Csm {
  async fn apply_sat_file(
    &self,
    params: ApplySatFileParams<'_>,
  ) -> Result<
    (
      Vec<CfsConfigurationResponse>,
      Vec<Image>,
      Vec<BosSessionTemplate>,
      Vec<BosSession>,
    ),
    Error,
  > {
    let ApplySatFileParams {
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      vault_base_url,
      site_name,
      k8s_api_url,
      sat_file,
      hsm_group_available_vec,
      ansible_verbosity,
      ansible_passthrough,
      gitea_base_url,
      gitea_token,
      reboot,
      watch_logs,
      timestamps,
      debug_on_failure,
      overwrite,
      dry_run,
    } = params;

    // The trait carries the SAT file as a structured `serde_json::Value`
    // (parsed once by the CLI). Transcode into `serde_yaml::Value` for
    // the existing `exec` signature — lossless for any valid SAT file
    // since the SAT spec is JSON-compatible.
    let sat_template_file_yaml: serde_yaml::Value =
      serde_json::from_value(sat_file).map_err(|e| {
        Error::Message(format!(
          "SAT file value is not a valid YAML mapping: {e}"
        ))
      })?;

    let shasta_k8s_secrets = fetch_shasta_k8s_secrets_from_vault(
      vault_base_url,
      shasta_token,
      site_name,
      socks5_proxy,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))?;

    #[allow(deprecated)]
    let (configurations, images, session_templates, sessions) =
      crate::commands::i_apply_sat_file::command::exec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        socks5_proxy,
        vault_base_url,
        site_name,
        k8s_api_url,
        shasta_k8s_secrets,
        sat_template_file_yaml,
        hsm_group_available_vec,
        ansible_verbosity,
        ansible_passthrough,
        gitea_base_url,
        gitea_token,
        reboot,
        watch_logs,
        timestamps,
        debug_on_failure,
        overwrite,
        dry_run,
      )
      .await
      .map_err(|e| Error::Message(e.to_string()))?;

    Ok((
      configurations.into_iter().map(Into::into).collect(),
      images.into_iter().map(Into::into).collect(),
      session_templates.into_iter().map(Into::into).collect(),
      sessions.into_iter().map(Into::into).collect(),
    ))
  }
}

impl ApplyHwClusterPin for Csm {
  async fn apply_hw_cluster_pin(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    pattern: &str,
    nodryrun: bool,
    create_target_hsm_group: bool,
    delete_empty_parent_hsm_group: bool,
  ) -> Result<(), Error> {
    crate::commands::apply_hw_cluster_pin::command::exec(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      self.socks5_proxy.as_deref(),
      target_hsm_group_name,
      parent_hsm_group_name,
      pattern,
      nodryrun,
      create_target_hsm_group,
      delete_empty_parent_hsm_group,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }
}
