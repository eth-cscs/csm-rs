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
    apply_sat_file::{
      ApplyConfigurationParams, ApplyImageParams, ApplySatFileParams,
      ApplySessionTemplateParams, SatTrait,
    },
  },
  types::{
    bos::{session::BosSession, session_template::BosSessionTemplate},
    cfs::cfs_configuration_response::CfsConfigurationResponse,
    ims::Image,
  },
};

use super::Csm;
use crate::{
  commands::i_apply_sat_file::utils,
  common::{
    kubernetes, vault::http_client::fetch_shasta_k8s_secrets_from_vault,
  },
};

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

  async fn apply_configuration(
    &self,
    params: ApplyConfigurationParams<'_>,
  ) -> Result<CfsConfigurationResponse, Error> {
    let ApplyConfigurationParams {
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      vault_base_url,
      site_name,
      k8s_api_url,
      gitea_base_url,
      gitea_token,
      configuration,
      dry_run,
      overwrite,
    } = params;

    // Transcode the structured Value (carried as JSON end-to-end) into
    // the serde_yaml::Value the per-entry creator expects. Lossless for
    // any valid SAT entry since the SAT spec is JSON-compatible.
    let configuration_yaml: serde_yaml::Value =
      serde_json::from_value(configuration).map_err(|e| {
        Error::Message(format!(
          "SAT configuration value is not a valid YAML mapping: {e}"
        ))
      })?;

    // Fetch the cray-product-catalog ConfigMap. The configurations
    // section uses it to resolve `product:` layers; gitea is used to
    // resolve branches to commits.
    let shasta_k8s_secrets = fetch_shasta_k8s_secrets_from_vault(
      vault_base_url,
      shasta_token,
      site_name,
      socks5_proxy,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))?;
    let kube_client =
      kubernetes::get_client(k8s_api_url, shasta_k8s_secrets, socks5_proxy)
        .await
        .map_err(|e| Error::Message(e.to_string()))?;
    let cray_product_catalog =
      kubernetes::try_get_configmap(kube_client, "cray-product-catalog")
        .await
        .map_err(|e| Error::Message(e.to_string()))?;

    let cfs_configuration = utils::create_cfs_configuration_from_sat_file(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      gitea_base_url,
      gitea_token,
      &cray_product_catalog,
      &configuration_yaml,
      dry_run,
      site_name,
      overwrite,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))?;

    Ok(cfs_configuration.into())
  }

  async fn apply_image(
    &self,
    params: ApplyImageParams<'_>,
  ) -> Result<Image, Error> {
    let ApplyImageParams {
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      vault_base_url,
      site_name,
      k8s_api_url,
      image,
      ref_lookup,
      hsm_group_available_vec: _,
      ansible_verbosity,
      ansible_passthrough,
      debug_on_failure,
      watch_logs,
      timestamps,
      dry_run,
    } = params;

    // Transcode JSON -> YAML -> typed SAT image shape.
    let image_yaml: serde_yaml::Value =
      serde_json::from_value(image).map_err(|e| {
        Error::Message(format!(
          "SAT image value is not a valid YAML mapping: {e}"
        ))
      })?;
    let image_struct: utils::image::Image = serde_yaml::from_value(image_yaml)
      .map_err(|e| {
        Error::Message(format!(
          "SAT image does not match the expected shape: {e}"
        ))
      })?;

    // Live state the per-image creator depends on.
    let shasta_k8s_secrets = fetch_shasta_k8s_secrets_from_vault(
      vault_base_url,
      shasta_token,
      site_name,
      socks5_proxy,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))?;
    let kube_client =
      kubernetes::get_client(k8s_api_url, shasta_k8s_secrets, socks5_proxy)
        .await
        .map_err(|e| Error::Message(e.to_string()))?;
    let cray_product_catalog =
      kubernetes::try_get_configmap(kube_client, "cray-product-catalog")
        .await
        .map_err(|e| Error::Message(e.to_string()))?;

    #[allow(deprecated)]
    let image = utils::images::i_create_image_from_sat_file_serde_yaml(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      vault_base_url,
      site_name,
      k8s_api_url,
      &image_struct,
      &cray_product_catalog,
      ansible_verbosity,
      ansible_passthrough,
      &ref_lookup,
      debug_on_failure,
      dry_run,
      watch_logs,
      timestamps,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))?;

    Ok(image.into())
  }

  async fn apply_session_template(
    &self,
    params: ApplySessionTemplateParams<'_>,
  ) -> Result<(BosSessionTemplate, Option<BosSession>), Error> {
    let ApplySessionTemplateParams {
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      session_template,
      ref_lookup,
      hsm_group_available_vec,
      reboot,
      dry_run,
    } = params;

    // The existing per-section function reads the entry out of
    // `sat_file_yaml["session_templates"][...]`. Wrap our single entry
    // in that shape rather than extracting the (intricate, 300-line)
    // loop body — same code path, much smaller diff. The trade-off is
    // that the audit log fires per-element instead of per-apply.
    let session_template_yaml: serde_yaml::Value =
      serde_json::from_value(session_template).map_err(|e| {
        Error::Message(format!(
          "SAT session_template value is not a valid YAML mapping: {e}"
        ))
      })?;
    let mut wrapper = serde_yaml::Mapping::new();
    wrapper.insert(
      serde_yaml::Value::String("session_templates".to_string()),
      serde_yaml::Value::Sequence(vec![session_template_yaml]),
    );
    let synthetic = serde_yaml::Value::Mapping(wrapper);

    let (mut templates, mut sessions) =
      utils::process_session_template_section_in_sat_file(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        socks5_proxy,
        ref_lookup,
        hsm_group_available_vec,
        synthetic,
        reboot,
        dry_run,
      )
      .await
      .map_err(|e| Error::Message(e.to_string()))?;

    let template = templates.pop().ok_or_else(|| {
      Error::Message("session_template apply returned no template".to_string())
    })?;
    // If reboot was requested, a BosSession was created (and added to
    // the returned vec only in non-dry-run mode — that's the existing
    // behaviour). pop() returns None when no reboot or dry-run.
    let session = sessions.pop();

    Ok((template.into(), session.map(Into::into)))
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
