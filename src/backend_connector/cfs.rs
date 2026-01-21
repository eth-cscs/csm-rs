use std::pin::Pin;

use chrono::NaiveDateTime;
use futures::{AsyncBufRead, AsyncReadExt};
use manta_backend_dispatcher::{
  error::Error,
  interfaces::cfs::CfsTrait,
  types::{
    bos::session_template::BosSessionTemplate,
    bss::BootParameters,
    cfs::{
      cfs_configuration_details::LayerDetails,
      cfs_configuration_request::CfsConfigurationRequest,
      cfs_configuration_response::{CfsConfigurationResponse, Layer},
      session::{CfsSessionGetResponse, CfsSessionPostRequest},
    },
    ims::Image as FrontEndImage,
    K8sAuth, K8sDetails,
  },
};

use super::Csm;
use crate::common::{
  jwt_ops, kubernetes, vault::http_client::fetch_shasta_k8s_secrets_from_vault,
};

impl CfsTrait for Csm {
  type T = Pin<Box<dyn AsyncBufRead + Send>>;

  async fn get_cfs_health(&self) -> Result<(), Error> {
    crate::cfs::health::test_connectivity_to_backend(self.base_url.as_str())
      .await
      .map_err(|e| Error::Message(e.to_string()))
  }

  async fn post_session(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    session: &CfsSessionPostRequest,
  ) -> Result<CfsSessionGetResponse, Error> {
    crate::cfs::session::http_client::v3::post(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &session.clone().into(),
    )
    .await
    .map(|cfs_session| cfs_session.into())
    .map_err(|e| Error::Message(e.to_string()))
  }

  /// Fetch CFS sessions ref --> https://apidocs.svc.cscs.ch/paas/cfs/operation/get_sessions/
  async fn get_sessions(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    session_name_opt: Option<&String>,
    limit_opt: Option<u8>,
    after_id_opt: Option<String>,
    min_age_opt: Option<String>,
    max_age_opt: Option<String>,
    status_opt: Option<String>,
    name_contains_opt: Option<String>,
    is_succeded_opt: Option<bool>,
    tags_opt: Option<String>,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    // Get local/backend CFS sessions
    let local_cfs_session_vec = crate::cfs::session::http_client::v3::get(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      session_name_opt,
      limit_opt,
      after_id_opt,
      min_age_opt,
      max_age_opt,
      status_opt,
      name_contains_opt,
      is_succeded_opt,
      tags_opt,
    )
    .await;

    // Convert to manta session
    let border_session_vec = local_cfs_session_vec
      .map(|cfs_session_vec| {
        cfs_session_vec
          .into_iter()
          .map(|cfs_session| cfs_session.into())
          .collect::<Vec<CfsSessionGetResponse>>()
      })
      .map_err(|e| Error::Message(e.to_string()));

    border_session_vec
  }

  async fn get_and_filter_sessions(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_vec: Vec<String>,
    xname_vec: Vec<&str>,
    min_age_opt: Option<&String>,
    max_age_opt: Option<&String>,
    type_opt: Option<&String>,
    status_opt: Option<&String>,
    cfs_session_name_opt: Option<&String>,
    limit_number_opt: Option<&u8>,
    is_succeded_opt: Option<bool>,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    if !hsm_group_name_vec.is_empty() && !xname_vec.is_empty() {
      eprintln!(
        "ERROR - Cannot filter by both HSM group names and xnames simultaneously"
      );
      std::process::exit(1);
    }

    let mut hsm_group_available_vec =
      crate::hsm::group::utils::get_group_available(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
      )
      .await
      // .map_err(|e| Error::Message(e.to_string()))?;
      .map_err(|e: crate::error::Error| {
        let manta_error: manta_backend_dispatcher::error::Error = e.into();
        manta_error
      })?;

    let (hsm_group_name_vec, xname_vec) = if !hsm_group_name_vec.is_empty() {
      // Filter HSM groups based on argument
      hsm_group_available_vec
        .retain(|group| hsm_group_name_vec.contains(&group.label));

      if hsm_group_available_vec.is_empty() {
        eprintln!("ERROR - None of the requested HSM groups are available");
        std::process::exit(1);
      };

      let mut member_available_vec = hsm_group_available_vec
        .iter()
        .flat_map(|g| g.get_members())
        .collect::<Vec<String>>();

      member_available_vec.sort();
      member_available_vec.dedup();

      (
        hsm_group_available_vec
          .into_iter()
          .map(|group| group.label)
          .collect::<Vec<String>>(),
        member_available_vec,
      )
    } else if !xname_vec.is_empty() {
      // Filter members available in the target HSM groups
      hsm_group_available_vec.retain(|group| {
        group
          .get_members()
          .iter()
          .any(|member| xname_vec.contains(&member.as_str()))
      });

      if hsm_group_available_vec.is_empty() {
        eprintln!(
              "ERROR - None of the requested xnames are available in the target HSM groups"
            );
        std::process::exit(1);
      }

      (
        hsm_group_available_vec
          .into_iter()
          .map(|group| group.label)
          .collect(),
        xname_vec.into_iter().map(|s| s.to_string()).collect(),
      )
    } else {
      // all HSM groups available
      // all members available
      let member_available_vec = hsm_group_available_vec
        .iter()
        .flat_map(|g| g.get_members())
        .collect::<Vec<String>>();

      (
        hsm_group_available_vec
          .into_iter()
          .map(|group| group.label)
          .collect(),
        member_available_vec,
      )
    };

    let mut cfs_session_vec = crate::cfs::session::get_and_sort(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      min_age_opt,
      max_age_opt,
      status_opt,
      cfs_session_name_opt,
      is_succeded_opt,
    )
    .await
    // .map_err(|e| Error::Message(e.to_string()))?;
    .map_err(|e: crate::error::Error| {
      let manta_error: manta_backend_dispatcher::error::Error = e.into();
      manta_error
    })?;

    crate::cfs::session::utils::filter(
      &mut cfs_session_vec,
      None,
      &hsm_group_name_vec,
      &xname_vec,
      type_opt,
      limit_number_opt,
      jwt_ops::is_user_admin(shasta_token),
    )
    // .map_err(|e| Error::Message(e.to_string()))?;
    .map_err(|e: crate::error::Error| {
      let manta_error: manta_backend_dispatcher::error::Error = e.into();
      manta_error
    })?;

    if cfs_session_vec.is_empty() {
      return Err(Error::Message("No CFS session found".to_string()));
    }

    for cfs_session in cfs_session_vec.iter_mut() {
      log::debug!("CFS session:\n{:#?}", cfs_session);

      if cfs_session.is_target_def_image() && cfs_session.is_success() {
        log::info!(
          "Find image ID related to CFS configuration {} in CFS session {}",
          cfs_session.configuration_name().unwrap(),
          cfs_session.name
        );

        let new_image_id_opt = if cfs_session
          .status
          .as_ref()
          .and_then(|status| {
            status.artifacts.as_ref().and_then(|artifacts| {
              artifacts
                .first()
                .and_then(|artifact| artifact.result_id.clone())
            })
          })
          .is_some()
        {
          let image_id = cfs_session.first_result_id();

          let new_image_vec_rslt: Result<
            Vec<crate::ims::image::http_client::types::Image>,
            _,
          > = crate::ims::image::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            // hsm_group_name_vec,
            image_id,
          )
          .await;

          // if new_image_id_vec_rslt.is_ok() && new_image_id_vec_rslt.as_ref().unwrap().first().is_some()
          if let Ok(Some(new_image)) = new_image_vec_rslt
            .as_ref()
            .map(|new_image_vec| new_image_vec.first())
          {
            Some(new_image.clone().id.unwrap_or("".to_string()))
          } else {
            None
          }
        } else {
          None
        };

        if new_image_id_opt.is_some() {
          cfs_session
            .status
            .clone()
            .unwrap()
            .artifacts
            .unwrap()
            .first()
            .unwrap()
            .clone()
            .result_id = new_image_id_opt;
        }
      }
    }

    Ok(
      cfs_session_vec
        .into_iter()
        .map(|cfs_session| cfs_session.into())
        .collect(),
    )
  }

  async fn delete_and_cancel_session(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    group_available_vec: &[manta_backend_dispatcher::types::Group],
    cfs_session: &manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse,
    cfs_component_vec: &[manta_backend_dispatcher::types::cfs::component::Component],
    bss_bootparameters_vec: &[BootParameters],
    dry_run: bool,
  ) -> Result<(), Error> {
    let group_available_vec: Vec<crate::hsm::group::types::Group> =
      group_available_vec
        .iter()
        .map(|group| group.clone().into())
        .collect();

    let cfs_session: crate::cfs::session::http_client::v2::types::CfsSessionGetResponse =
        cfs_session.clone().into();

    let cfs_component_vec: Vec<
      crate::cfs::component::http_client::v2::types::Component,
    > = cfs_component_vec
      .iter()
      .map(|component| component.clone().into())
      .collect();

    let bss_bootparameters_vec: Vec<crate::bss::types::BootParameters> =
      bss_bootparameters_vec
        .iter()
        .map(|bp| bp.clone().into())
        .collect();

    crate::commands::delete_and_cancel_session::command::exec(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      group_available_vec,
      &cfs_session,
      &cfs_component_vec,
      &bss_bootparameters_vec,
      dry_run,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn create_configuration_from_repos(
    &self,
    gitea_token: &str,
    gitea_base_url: &str,
    shasta_root_cert: &[u8],
    repo_name_vec: &[&str],
    local_git_commit_vec: &[&str],
    playbook_file_name_opt: Option<&str>,
  ) -> Result<CfsConfigurationRequest, Error> {
    Ok(crate::cfs::configuration::http_client::v3::types::cfs_configuration_request::CfsConfigurationRequest::create_from_repos(
            gitea_token,
            gitea_base_url,
            shasta_root_cert,
            repo_name_vec,
            local_git_commit_vec,
            playbook_file_name_opt,
        ).await.map_err(|e| Error::Message(e.to_string()))?.into())
  }

  async fn get_configuration(
    &self,
    auth_token: &str,
    base_url: &str,
    root_cert: &[u8],
    configuration_name_opt: Option<&String>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    let cfs_configuration_vec =
      crate::cfs::configuration::http_client::v3::get(
        auth_token,
        base_url,
        root_cert,
        configuration_name_opt.map(|elem| elem.as_str()),
      )
      .await
      .map_err(|e| Error::Message(e.to_string()));

    cfs_configuration_vec
      .map(|config_vec| config_vec.into_iter().map(|c| c.into()).collect())
  }

  async fn get_and_filter_configuration(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration_name: Option<&str>,
    configuration_name_pattern: Option<&str>,
    hsm_group_name_vec: &[String],
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
    limit_number_opt: Option<&u8>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    //TODO: Get rid of this by making sure CSM admins don't create HSM groups for system
    //wide operations instead of using roles
    let hsm_group_name_vec =
      crate::hsm::group::hacks::filter_system_hsm_group_names(
        hsm_group_name_vec.to_vec(),
      );

    dbg!(&hsm_group_name_vec);

    crate::cfs::configuration::utils::get_and_filter(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      configuration_name,
      configuration_name_pattern,
      &hsm_group_name_vec,
      since_opt,
      until_opt,
      limit_number_opt,
    )
    .await
    .map(|config_vec| config_vec.into_iter().map(|c| c.into()).collect())
    // .map_err(|e| Error::Message(e.to_string()))
    .map_err(|e: crate::error::Error| {
      let manta_error: manta_backend_dispatcher::error::Error = e.into();
      manta_error
    })
  }

  async fn get_configuration_layer_details(
    &self,
    shasta_root_cert: &[u8],
    gitea_base_url: &str,
    gitea_token: &str,
    layer: Layer,
    site_name: &str,
  ) -> Result<LayerDetails, Error> {
    crate::cfs::configuration::utils::get_configuration_layer_details(
      shasta_root_cert,
      gitea_base_url,
      gitea_token,
      layer.into(),
      site_name,
    )
    .await
    .map(|layer_details| layer_details.into())
    .map_err(|e| e.into())
  }

  /// Create a new CFS configuration
  async fn put_configuration(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration: &CfsConfigurationRequest,
    configuration_name: &str,
    overwrite: bool,
  ) -> Result<CfsConfigurationResponse, Error> {
    crate::cfs::configuration::utils::create_new_configuration(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &configuration.clone().into(),
      configuration_name,
      overwrite,
    )
    .await
    .map(|cfs_configuration| cfs_configuration.into())
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn get_session_logs_stream(
    &self,
    shasta_token: &str,
    site_name: &str,
    cfs_session_name: &str,
    timestamps: bool,
    k8s: &K8sDetails,
  ) -> Result<Pin<Box<dyn AsyncBufRead + Send>>, Error> {
    let shasta_k8s_secrets = match &k8s.authentication {
      K8sAuth::Native {
        certificate_authority_data,
        client_certificate_data,
        client_key_data,
      } => {
        serde_json::json!({ "certificate-authority-data": certificate_authority_data, "client-certificate-data": client_certificate_data, "client-key-data": client_key_data })
      }
      K8sAuth::Vault { base_url } => {
        fetch_shasta_k8s_secrets_from_vault(&base_url, shasta_token, &site_name)
          .await
          .map_err(|e| Error::Message(format!("{e}")))?
      }
    };

    let client = kubernetes::get_client(&k8s.api_url, shasta_k8s_secrets)
      .await
      .map_err(|e| Error::Message(format!("{e}")))?;

    let (log_stream_git_clone, exit_code) =
      kubernetes::get_cfs_session_init_container_git_clone_logs_stream(
        client.clone(),
        cfs_session_name,
        timestamps,
      )
      .await
      .map_err(|e| Error::Message(format!("{e}")))?;

    if exit_code != 0 {
      log::error!(
        "CFS session '{}' git-clone init container failed with exit code {}",
        cfs_session_name,
        exit_code
      );
      return Ok(Box::pin(log_stream_git_clone));
    }

    let log_stream_inventory =
      kubernetes::get_cfs_session_container_inventory_logs_stream(
        client.clone(),
        cfs_session_name,
        timestamps,
      )
      .await
      .map_err(|e| Error::Message(format!("{e}")))?;

    let log_stream_ansible =
      kubernetes::get_cfs_session_container_ansible_logs_stream(
        client,
        cfs_session_name,
        timestamps,
      )
      .await
      .map_err(|e| Error::Message(format!("{e}")))?;

    // NOTE: here is where we convert from impl AsyncBufRead to Pin<Box<dyn AsyncBufRead>>
    // through dynamic dispatch
    Ok(Box::pin(
      log_stream_git_clone
        .chain(log_stream_inventory)
        .chain(log_stream_ansible),
    ))
  }

  async fn update_runtime_configuration(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xnames: &[String],
    desired_configuration: &str,
    enabled: bool,
  ) -> Result<(), Error> {
    crate::cfs::component::utils::update_component_list_desired_configuration(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      xnames,
      desired_configuration,
      enabled,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }

  // Get all CFS sessions, IMS images and BOS sessiontemplates related to a CFS configuration
  async fn get_derivatives(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration_name: &str,
  ) -> Result<
    (
      Option<Vec<CfsSessionGetResponse>>,
      Option<Vec<BosSessionTemplate>>,
      Option<Vec<FrontEndImage>>,
    ),
    Error,
  > {
    crate::cfs::configuration::utils::get_derivatives(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      configuration_name,
    )
    .await
    .map(|(cfs_session_vec, bos_session_template_vec, image_vec)| {
      (
        cfs_session_vec.map(|cfs_session_vec| {
          cfs_session_vec
            .into_iter()
            .map(|cfs_session| cfs_session.into())
            .collect()
        }),
        bos_session_template_vec.map(|bos_session_template_vec| {
          bos_session_template_vec
            .into_iter()
            .map(|bos_session_template| bos_session_template.into())
            .collect()
        }),
        image_vec.map(|image_vec| {
          image_vec.into_iter().map(|image| image.into()).collect()
        }),
      )
    })
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn get_cfs_components(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration_name: Option<&str>,
    components_ids: Option<&str>,
    status: Option<&str>,
  ) -> Result<
    Vec<manta_backend_dispatcher::types::cfs::component::Component>,
    Error,
  > {
    crate::cfs::component::http_client::v3::get_query(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      configuration_name,
      components_ids,
      status,
    )
    .await
    .map(|component_vec| {
      component_vec
        .into_iter()
        .map(|component| component.into())
        .collect()
    })
    .map_err(|e| Error::Message(e.to_string()))
  }
}
