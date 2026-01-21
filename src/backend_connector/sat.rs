use manta_backend_dispatcher::{
  error::Error,
  interfaces::{
    apply_hw_cluster_pin::ApplyHwClusterPin, apply_sat_file::SatTrait,
  },
};

use super::Csm;

impl SatTrait for Csm {
  async fn apply_sat_file(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    site_name: &str,
    k8s_api_url: &str,
    shasta_k8s_secrets: serde_json::Value,
    sat_template_file_yaml: serde_yaml::Value,
    hsm_group_available_vec: &[String],
    ansible_verbosity_opt: Option<u8>,
    ansible_passthrough_opt: Option<&str>,
    gitea_base_url: &str,
    gitea_token: &str,
    reboot: bool,
    watch_logs: bool,
    timestamps: bool,
    debug_on_failure: bool,
    overwrite: bool,
    dry_run: bool,
  ) -> Result<(), Error> {
    crate::commands::i_apply_sat_file::command::exec(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      vault_base_url,
      site_name,
      k8s_api_url,
      shasta_k8s_secrets,
      sat_template_file_yaml,
      hsm_group_available_vec,
      ansible_verbosity_opt,
      ansible_passthrough_opt,
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
    .map_err(|e| Error::Message(e.to_string()))
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
