use manta_backend_dispatcher::{
  error::Error,
  interfaces::{
    apply_session::ApplySessionTrait,
    bos::{ClusterSessionTrait, ClusterTemplateTrait},
  },
  types::bos::{session::BosSession, session_template::BosSessionTemplate},
};

use super::Csm;

impl ApplySessionTrait for Csm {
  async fn apply_session(
    &self,
    gitea_token: &str,
    gitea_base_url: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    // k8s_api_url: &str,
    cfs_conf_sess_name: Option<&str>,
    playbook_yaml_file_name_opt: Option<&str>,
    hsm_group: Option<&str>,
    repos_name_vec: &[&str],
    repos_last_commit_id_vec: &[&str],
    ansible_limit: Option<&str>,
    ansible_verbosity: Option<&str>,
    ansible_passthrough: Option<&str>,
    // watch_logs: bool,
    /* kafka_audit: &Kafka,
    k8s: &K8sDetails, */
  ) -> Result<(String, String), Error> {
    crate::commands::apply_session::exec(
      gitea_token,
      gitea_base_url,
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      // k8s_api_url,
      cfs_conf_sess_name,
      playbook_yaml_file_name_opt,
      hsm_group,
      repos_name_vec,
      repos_last_commit_id_vec,
      ansible_limit,
      ansible_verbosity,
      ansible_passthrough,
      // watch_logs,
      /* kafka_audit,
      k8s, */
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }
}

impl ClusterSessionTrait for Csm {
  async fn post_template_session(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_session: manta_backend_dispatcher::types::bos::session::BosSession,
  ) -> Result<BosSession, Error> {
    crate::bos::session::http_client::v2::post(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      bos_session.into(),
    )
    .await
    .map(|bos_session| bos_session.into())
    .map_err(|e| Error::Message(e.to_string()))
  }
}

impl ClusterTemplateTrait for Csm {
  async fn get_template(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_session_template_id_opt: Option<&str>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    crate::bos::template::http_client::v2::get(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      bos_session_template_id_opt,
    )
    .await
    .map(|bos_session_template_vec| {
      bos_session_template_vec
        .into_iter()
        .map(|template| template.into())
        .collect::<Vec<BosSessionTemplate>>()
    })
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn get_and_filter_templates(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec: &[&str],
    hsm_member_vec: &[&str],
    bos_sessiontemplate_name_opt: Option<&str>,
    limit_number_opt: Option<&u8>,
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    let mut bos_sessiontemplate_vec =
      crate::bos::template::http_client::v2::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        bos_sessiontemplate_name_opt.map(|value| value),
      )
      .await
      .map_err(|e| Error::Message(e.to_string()))?;

    crate::bos::template::utils::filter(
      &mut bos_sessiontemplate_vec,
      None,
      hsm_group_name_vec,
      hsm_member_vec,
      limit_number_opt,
    )
    .map_err(|e| Error::Message(e.to_string()))?;

    Ok(
      bos_sessiontemplate_vec
        .into_iter()
        .map(|template| template.into())
        .collect::<Vec<BosSessionTemplate>>(),
    )
  }

  async fn get_all_templates(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
  ) -> Result<Vec<BosSessionTemplate>, Error> {
    crate::bos::template::http_client::v2::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
    )
    .await
    .map(|bos_session_template_vec| {
      bos_session_template_vec
        .into_iter()
        .map(|template| template.into())
        .collect::<Vec<BosSessionTemplate>>()
    })
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn put_template(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_template: &BosSessionTemplate,
    bos_template_name: &str,
  ) -> Result<BosSessionTemplate, Error> {
    crate::bos::template::http_client::v2::put(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &bos_template.clone().into(),
      bos_template_name,
    )
    .await
    .map(|bos_session_template| bos_session_template.into())
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn delete_template(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_template_id: &str,
  ) -> Result<(), Error> {
    crate::bos::template::http_client::v2::delete(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      bos_template_id,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }
}
