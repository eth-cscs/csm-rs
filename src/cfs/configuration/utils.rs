use crate::{
  bos::{self, template::http_client::v2::types::BosSessionTemplate},
  cfs::{
    self, component::http_client::v2::types::Component,
    configuration::http_client::v2::types::cfs_configuration_response::CfsConfigurationResponse,
    session::http_client::v2::types::CfsSessionGetResponse,
  },
  common::{self, gitea},
  error::Error,
  hsm,
  ims::{self, image::http_client::types::Image},
};

use chrono::NaiveDateTime;
use globset::Glob;
use serde_json::Value;

use super::http_client::{
  v2::types::cfs_configuration_request::CfsConfigurationRequest,
  v3::types::{
    cfs_configuration::LayerDetails, cfs_configuration_response::Layer,
  },
};

pub async fn create_new_configuration(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  configuration: &CfsConfigurationRequest,
  configuration_name: &str,
  overwrite: bool,
) -> Result<CfsConfigurationResponse, Error> {
  // Check if CFS configuration already exists
  log::info!("Check CFS configuration '{}' exists", configuration_name);

  let cfs_configuration_vec = crate::cfs::configuration::http_client::v2::get(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    Some(configuration_name),
  )
  .await
  .map_err(|e| Error::Message(e.to_string()))
  .unwrap_or_default();

  // Check if CFS configuration already exists and throw an error is that is the case
  if !cfs_configuration_vec.is_empty() {
    if overwrite {
      log::info!(
          "CFS configuration '{}' already exists but 'overwrite' has been enabled",
          configuration_name
        );
    } else {
      log::warn!(
        "CFS configuration '{}' already exists, cancel the process",
        configuration_name
      );
      return Err(Error::ConfigurationAlreadyExists(
        configuration_name.to_string(),
      ));
    }
  }

  log::info!(
    "CFS configuration '{}' does not exists, creating new CFS configuration",
    configuration_name
  );

  crate::cfs::configuration::http_client::v2::put(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    &configuration.clone().into(),
    configuration_name,
  )
  .await
  .map(|config| config.into())
  .map_err(|e| Error::Message(e.to_string()))
}

pub fn filter_3(
  cfs_configuration_vec: &mut Vec<CfsConfigurationResponse>,
  configuration_name_pattern_opt: Option<&str>,
  limit_number_opt: Option<&u8>,
  since_opt: Option<NaiveDateTime>,
  until_opt: Option<NaiveDateTime>,
) -> Result<Vec<CfsConfigurationResponse>, Error> {
  log::info!("Filter CFS configurations");

  // Filter CFS configurations based on user input (date range or configuration name)
  if let (Some(since), Some(until)) = (since_opt, until_opt) {
    cfs_configuration_vec.retain(|cfs_configuration| {
      let date =
        chrono::DateTime::parse_from_rfc3339(&cfs_configuration.last_updated)
          .unwrap()
          .naive_utc();

      since <= date && date < until
    });
  }

  // Filter CFS configurations based on pattern matching
  if let Some(configuration_name_pattern) = configuration_name_pattern_opt {
    let glob = Glob::new(configuration_name_pattern)
      .unwrap()
      .compile_matcher();
    cfs_configuration_vec.retain(|cfs_configuration| {
      glob.is_match(cfs_configuration.name.clone())
    });
  }

  // Sort by last updated date in ASC order
  cfs_configuration_vec.sort_by(|cfs_configuration_1, cfs_configuration_2| {
    cfs_configuration_1
      .last_updated
      .cmp(&cfs_configuration_2.last_updated)
  });

  // Limiting the number of results to return to client
  if let Some(limit_number) = limit_number_opt {
    *cfs_configuration_vec = cfs_configuration_vec[cfs_configuration_vec
      .len()
      .saturating_sub(*limit_number as usize)..]
      .to_vec();
  }

  Ok(cfs_configuration_vec.to_vec())
}

/// Filter the list of CFS configurations provided. This operation is very expensive since it is
/// filtering by HSM group which means it needs to link CFS configurations with CFS sessions and
/// BOS sessiontemplate. Aditionally, it will also fetch CFS components to find CFS sessions and
/// BOS sessiontemplates linked to specific xnames that also belongs to the HSM group the user is
/// filtering from.
pub fn filter(
  cfs_configuration_vec: &mut Vec<CfsConfigurationResponse>,
  xname_from_groups_vec: &Vec<String>,
  cfs_session_vec: &mut Vec<CfsSessionGetResponse>,
  bos_sessiontemplate_vec: &mut Vec<BosSessionTemplate>,
  cfs_component_vec: &Vec<Component>,
  configuration_name_pattern_opt: Option<&str>,
  hsm_group_name_vec: &[String],
  since_opt: Option<NaiveDateTime>,
  until_opt: Option<NaiveDateTime>,
  limit_number_opt: Option<&u8>,
  keep_generic_sessions: bool,
) -> Result<Vec<CfsConfigurationResponse>, Error> {
  log::info!("Filter CFS configurations");

  // Filter BOS sessiontemplates based on HSM groups
  bos::template::utils::filter(
    bos_sessiontemplate_vec,
    hsm_group_name_vec,
    &xname_from_groups_vec,
    None,
  );

  // Filter CFS sessions based on HSM groups
  cfs::session::utils::filter(
    cfs_session_vec,
    hsm_group_name_vec,
    xname_from_groups_vec,
    None,
    keep_generic_sessions,
  )?;

  // Get boot image id and desired configuration from BOS sessiontemplates
  let image_id_cfs_configuration_target_from_bos_sessiontemplate: Vec<(
    String,
    String,
    Vec<String>,
  )> = bos::template::utils::get_image_id_cfs_configuration_target_tuple_vec(
    bos_sessiontemplate_vec,
  );

  // Get image id, configuration and targets from CFS sessions
  let image_id_cfs_configuration_target_from_cfs_session: Vec<(
    String,
    String,
    Vec<String>,
  )> = cfs::session::utils::get_image_id_cfs_configuration_target_tuple_vec(
    cfs_session_vec,
  );

  // Get desired configuration from CFS components
  let desired_config_vec: Vec<String> = cfs_component_vec
    .into_iter()
    .map(|cfs_component| cfs_component.desired_config.clone().unwrap())
    .collect();

  // Merge CFS configurations in list of filtered CFS sessions and BOS sessiontemplates and
  // desired configurations in CFS components
  let cfs_configuration_in_cfs_session_and_bos_sessiontemplate: Vec<String> = [
    image_id_cfs_configuration_target_from_bos_sessiontemplate
      .into_iter()
      .map(|(_, config, _)| config)
      .collect(),
    image_id_cfs_configuration_target_from_cfs_session
      .into_iter()
      .map(|(_, config, _)| config)
      .collect(),
    desired_config_vec,
  ]
  .concat();

  // Filter CFS configurations
  //
  // Filter CFS configurations based on HSM group names
  cfs_configuration_vec.retain(|cfs_configuration| {
    hsm_group_name_vec
      .iter()
      .any(|hsm_group| cfs_configuration.name.contains(hsm_group))
      || cfs_configuration_in_cfs_session_and_bos_sessiontemplate
        .contains(&cfs_configuration.name)
  });

  // Filter CFS configurations based on user input (date range or configuration name)
  if let (Some(since), Some(until)) = (since_opt, until_opt) {
    cfs_configuration_vec.retain(|cfs_configuration| {
      let date =
        chrono::DateTime::parse_from_rfc3339(&cfs_configuration.last_updated)
          .unwrap()
          .naive_utc();

      since <= date && date < until
    });
  }

  // Sort by last updated date in ASC order
  cfs_configuration_vec.sort_by(|cfs_configuration_1, cfs_configuration_2| {
    cfs_configuration_1
      .last_updated
      .cmp(&cfs_configuration_2.last_updated)
  });

  // Filter CFS configurations based on mattern matching
  if let Some(configuration_name_pattern) = configuration_name_pattern_opt {
    let glob = Glob::new(configuration_name_pattern)
      .unwrap()
      .compile_matcher();
    cfs_configuration_vec.retain(|cfs_configuration| {
      glob.is_match(cfs_configuration.name.clone())
    });
  }

  // Limiting the number of results to return to client
  if let Some(limit_number) = limit_number_opt {
    *cfs_configuration_vec = cfs_configuration_vec[cfs_configuration_vec
      .len()
      .saturating_sub(*limit_number as usize)..]
      .to_vec();
  }

  Ok(cfs_configuration_vec.to_vec())
}

/* /// Filter the list of CFS configurations provided. This operation is very expensive since it is
/// filtering by HSM group which means it needs to link CFS configurations with CFS sessions and
/// BOS sessiontemplate. Aditionally, it will also fetch CFS components to find CFS sessions and
/// BOS sessiontemplates linked to specific xnames that also belongs to the HSM group the user is
/// filtering from.
#[deprecated(note = "please use `filter_2` instead")]
pub async fn filter(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cfs_configuration_vec: &mut Vec<CfsConfigurationResponse>,
  configuration_name_pattern_opt: Option<&str>,
  hsm_group_name_vec: &[String],
  since_opt: Option<NaiveDateTime>,
  until_opt: Option<NaiveDateTime>,
  limit_number_opt: Option<&u8>,
) -> Result<Vec<CfsConfigurationResponse>, Error> {
  log::info!("Filter CFS configurations");

  // COLLECT SITE WIDE DATA FOR VALIDATION
  //
  let xname_from_groups_vec =
    hsm::group::utils::get_member_vec_from_hsm_name_vec(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_name_vec,
    )
    .await?;

  let (mut cfs_session_vec, mut bos_sessiontemplate_vec, cfs_component_vec) = tokio::try_join!(
    cfs::session::http_client::v2::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
    ),
    bos::template::http_client::v2::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
    ),
    cfs::component::http_client::v2::get_parallel(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &xname_from_groups_vec,
    ),
  )?;

  // Filter BOS sessiontemplates based on HSM groups
  bos::template::utils::filter(
    &mut bos_sessiontemplate_vec,
    hsm_group_name_vec,
    &xname_from_groups_vec,
    None,
  );

  // Filter CFS sessions based on HSM groups
  cfs::session::utils::filter_by_hsm(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    &mut cfs_session_vec,
    hsm_group_name_vec,
    None,
    true,
  )
  .await?;

  // Get boot image id and desired configuration from BOS sessiontemplates
  let image_id_cfs_configuration_target_from_bos_sessiontemplate: Vec<(
    String,
    String,
    Vec<String>,
  )> = bos::template::utils::get_image_id_cfs_configuration_target_tuple_vec(
    bos_sessiontemplate_vec,
  );

  // Get image id, configuration and targets from CFS sessions
  let image_id_cfs_configuration_target_from_cfs_session: Vec<(
    String,
    String,
    Vec<String>,
  )> = cfs::session::utils::get_image_id_cfs_configuration_target_tuple_vec(
    cfs_session_vec,
  );

  // Get desired configuration from CFS components
  let desired_config_vec: Vec<String> = cfs_component_vec
    .into_iter()
    .map(|cfs_component| cfs_component.desired_config.unwrap())
    .collect();

  // Merge CFS configurations in list of filtered CFS sessions and BOS sessiontemplates and
  // desired configurations in CFS components
  let cfs_configuration_in_cfs_session_and_bos_sessiontemplate: Vec<String> = [
    image_id_cfs_configuration_target_from_bos_sessiontemplate
      .into_iter()
      .map(|(_, config, _)| config)
      .collect(),
    image_id_cfs_configuration_target_from_cfs_session
      .into_iter()
      .map(|(_, config, _)| config)
      .collect(),
    desired_config_vec,
  ]
  .concat();

  // Filter CFS configurations
  //
  // Filter CFS configurations based on HSM group names
  cfs_configuration_vec.retain(|cfs_configuration| {
    hsm_group_name_vec
      .iter()
      .any(|hsm_group| cfs_configuration.name.contains(hsm_group))
      || cfs_configuration_in_cfs_session_and_bos_sessiontemplate
        .contains(&cfs_configuration.name)
  });

  // Filter CFS configurations based on user input (date range or configuration name)
  if let (Some(since), Some(until)) = (since_opt, until_opt) {
    cfs_configuration_vec.retain(|cfs_configuration| {
      let date =
        chrono::DateTime::parse_from_rfc3339(&cfs_configuration.last_updated)
          .unwrap()
          .naive_utc();

      since <= date && date < until
    });
  }

  // Sort by last updated date in ASC order
  cfs_configuration_vec.sort_by(|cfs_configuration_1, cfs_configuration_2| {
    cfs_configuration_1
      .last_updated
      .cmp(&cfs_configuration_2.last_updated)
  });

  // Filter CFS configurations based on mattern matching
  if let Some(configuration_name_pattern) = configuration_name_pattern_opt {
    let glob = Glob::new(configuration_name_pattern)
      .unwrap()
      .compile_matcher();
    cfs_configuration_vec.retain(|cfs_configuration| {
      glob.is_match(cfs_configuration.name.clone())
    });
  }

  // Limiting the number of results to return to client
  if let Some(limit_number) = limit_number_opt {
    *cfs_configuration_vec = cfs_configuration_vec[cfs_configuration_vec
      .len()
      .saturating_sub(*limit_number as usize)..]
      .to_vec();
  }

  Ok(cfs_configuration_vec.to_vec())
} */

/* /// Filter the list of CFS configurations provided. This operation is very expensive since it is
/// filtering by HSM group which means it needs to link CFS configurations with CFS sessions and
/// BOS sessiontemplate. Aditionally, it will also fetch CFS components to find CFS sessions and
/// BOS sessiontemplates linked to specific xnames that also belongs to the HSM group the user is
/// filtering from.
pub async fn filter(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cfs_configuration_vec: &mut Vec<CfsConfigurationResponse>,
  configuration_name_pattern_opt: Option<&str>,
  hsm_group_name_vec: &[String],
  limit_number_opt: Option<&u8>,
) -> Result<Vec<CfsConfigurationResponse>, Error> {
  log::info!("Filter CFS configurations");
  // Get HSM Group members the user has access to
  let hsm_group_members_vec =
    hsm::group::utils::get_member_vec_from_hsm_name_vec(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_name_vec,
    )
    .await?;

  // Get CFS components the user has access to, all CFS sessions, BOS session template
  // Fetch "CFS components" the user has access to (filter by HSM group members)
  // Note: nodes can be configured calling the "CFS Component API" directly (bypassing BOS
  // session API)
  let (cfs_component_vec, mut cfs_session_vec, mut bos_sessiontemplate_vec) = tokio::try_join!(
    cfs::component::http_client::v2::get_parallel(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &hsm_group_members_vec,
    ),
    crate::cfs::session::http_client::v2::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert
    ),
    crate::bos::template::http_client::v2::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert
    )
  )?;

  // Filter BOS sessiontemplates based on HSM groups
  bos::template::utils::filter(
    &mut bos_sessiontemplate_vec,
    hsm_group_name_vec,
    &hsm_group_members_vec,
    None,
  );

  // Filter CFS sessions based on HSM groups
  cfs::session::utils::filter_by_hsm(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    &mut cfs_session_vec,
    hsm_group_name_vec,
    None,
    true,
  )
  .await?;

  // Get boot image id and desired configuration from BOS sessiontemplates
  let image_id_cfs_configuration_target_from_bos_sessiontemplate: Vec<(
    String,
    String,
    Vec<String>,
  )> = bos::template::utils::get_image_id_cfs_configuration_target_tuple_vec(
    bos_sessiontemplate_vec,
  );

  // Get image id, configuration and targets from CFS sessions
  let image_id_cfs_configuration_target_from_cfs_session: Vec<(
    String,
    String,
    Vec<String>,
  )> = cfs::session::utils::get_image_id_cfs_configuration_target_tuple_vec(
    cfs_session_vec,
  );

  // Get desired configuration from CFS components
  let desired_config_vec: Vec<String> = cfs_component_vec
    .into_iter()
    .map(|cfs_component| cfs_component.desired_config.unwrap())
    .collect();

  // Merge CFS configurations in list of filtered CFS sessions and BOS sessiontemplates and
  // desired configurations in CFS components
  let cfs_configuration_in_cfs_session_and_bos_sessiontemplate: Vec<String> = [
    image_id_cfs_configuration_target_from_bos_sessiontemplate
      .into_iter()
      .map(|(_, config, _)| config)
      .collect(),
    image_id_cfs_configuration_target_from_cfs_session
      .into_iter()
      .map(|(_, config, _)| config)
      .collect(),
    desired_config_vec,
  ]
  .concat();

  // Filter CFS configurations
  //
  // Filter CFS configurations based on HSM group names
  cfs_configuration_vec.retain(|cfs_configuration| {
    hsm_group_name_vec
      .iter()
      .any(|hsm_group| cfs_configuration.name.contains(hsm_group))
      || cfs_configuration_in_cfs_session_and_bos_sessiontemplate
        .contains(&cfs_configuration.name)
  });

  // Sort by last updated date in ASC order
  cfs_configuration_vec.sort_by(|cfs_configuration_1, cfs_configuration_2| {
    cfs_configuration_1
      .last_updated
      .cmp(&cfs_configuration_2.last_updated)
  });

  // Limit the number of results to return to client
  if let Some(limit_number) = limit_number_opt {
    *cfs_configuration_vec = cfs_configuration_vec[cfs_configuration_vec
      .len()
      .saturating_sub(*limit_number as usize)..]
      .to_vec();
  }

  // Filter CFS configurations based on mattern matching
  if let Some(configuration_name_pattern) = configuration_name_pattern_opt {
    let glob = Glob::new(configuration_name_pattern)
      .unwrap()
      .compile_matcher();
    cfs_configuration_vec.retain(|cfs_configuration| {
      glob.is_match(cfs_configuration.name.clone())
    });
  }

  Ok(cfs_configuration_vec.to_vec())
} */

/// If filtering by HSM group, then configuration name must include HSM group name (It assumms each configuration
/// is built for a specific cluster based on ansible vars used by the CFS session). The reason
/// for this is because CSCS staff deletes all CFS sessions every now and then...
pub async fn get_and_filter(
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
  // Get list of configurations.
  // Returns list with only one element if "configuration name" provided

  // COLLECT SITE WIDE DATA FOR VALIDATION
  //
  let xname_from_groups_vec =
    hsm::group::utils::get_member_vec_from_hsm_name_vec(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_name_vec,
    )
    .await?;

  let (
    mut cfs_configuration_vec,
    mut cfs_session_vec,
    mut bos_sessiontemplate_vec,
    cfs_component_vec,
  ) = tokio::try_join!(
    cfs::configuration::http_client::v2::get(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      configuration_name,
    ),
    cfs::session::http_client::v2::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
    ),
    bos::template::http_client::v2::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
    ),
    cfs::component::http_client::v2::get_parallel(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &xname_from_groups_vec,
    ),
  )?;

  let keep_generic_sessions = common::jwt_ops::is_user_admin(shasta_token);

  // Filter CFS configurations if user is not admin
  cfs::configuration::utils::filter(
    &mut cfs_configuration_vec,
    &xname_from_groups_vec,
    &mut cfs_session_vec,
    &mut bos_sessiontemplate_vec,
    &cfs_component_vec,
    configuration_name_pattern,
    hsm_group_name_vec,
    since_opt,
    until_opt,
    limit_number_opt,
    keep_generic_sessions,
  )?;

  Ok(cfs_configuration_vec)
}

// Get all CFS sessions, IMS images and BOS sessiontemplates related to a CFS configuration
pub async fn get_derivatives(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  configuration_name: &str,
) -> Result<
  (
    Option<Vec<CfsSessionGetResponse>>,
    Option<Vec<BosSessionTemplate>>,
    Option<Vec<Image>>,
  ),
  Error,
> {
  // List of image ids from CFS sessions and BOS sessiontemplates related to CFS configuration
  let mut image_id_vec: Vec<&str> = Vec::new();

  let (mut cfs_session_vec, mut bos_sessiontemplate_vec, mut ims_image_vec) = tokio::try_join!(
    cfs::session::http_client::v2::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert
    ),
    bos::template::http_client::v2::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert
    ),
    ims::image::http_client::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert
    )
  )?;

  // Filter CFS sessions
  cfs::session::utils::filter_by_cofiguration(
    &mut cfs_session_vec,
    configuration_name,
  );

  // Filter BOS sessiontemplate
  bos_sessiontemplate_vec.retain(|bos_sessiontemplate| {
    bos_sessiontemplate
      .images_id()
      .any(|image_id_aux| image_id_vec.contains(&image_id_aux))
      || bos_sessiontemplate.get_configuration().unwrap_or_default()
        == configuration_name
  });

  // Add all image ids in CFS sessions into image_id_vec
  image_id_vec.extend(
    cfs_session_vec
      .iter()
      .flat_map(|cfs_session| cfs_session.results_id()),
  );

  // Add boot images from BOS sessiontemplate to image_id_vec
  image_id_vec.extend(
    bos_sessiontemplate_vec
      .iter()
      .flat_map(|bos_sessiontemplate| bos_sessiontemplate.images_id()),
  );

  // Filter images
  ims_image_vec.retain(|image| {
    image_id_vec.contains(&image.id.as_deref().unwrap_or_default())
  });

  Ok((
    Some(cfs_session_vec),
    Some(bos_sessiontemplate_vec),
    Some(ims_image_vec),
  ))
}

pub async fn get_configuration_layer_details(
  shasta_root_cert: &[u8],
  gitea_base_url: &str,
  gitea_token: &str,
  layer: Layer,
  site_name: &str,
) -> Result<LayerDetails, Error> {
  let commit_id: String =
    layer.commit.clone().unwrap_or("Not defined".to_string());
  // let branch_name_opt: Option<&str> = layer.branch.as_deref();
  // let mut most_recent_commit: bool = false;
  let mut branch_name_vec: Vec<String> = Vec::new();
  let mut tag_name_vec: Vec<String> = Vec::new();
  let commit_sha;

  let repo_ref_vec_rslt = gitea::http_client::get_all_refs_from_repo_url(
    gitea_base_url,
    gitea_token,
    &layer.clone_url,
    shasta_root_cert,
  )
  .await;

  let repo_ref_vec = match repo_ref_vec_rslt {
    Ok(value) => value,
    Err(error) => {
      log::warn!(
        "Could not fetch repo '{}' refs. Reason:\n{:#?}",
        layer.clone_url,
        error
      );
      vec![]
    }
  };

  let mut ref_value_vec: Vec<&Value> = repo_ref_vec
    .iter()
    .filter(|repo_ref| {
      repo_ref
        .pointer("/object/sha")
        .unwrap()
        .as_str()
        .unwrap()
        .eq(&commit_id)
    })
    .collect();

  // Check if ref filtering returns an annotated tag, if so, then get the SHA of its
  // commit because it will be needed in case there are branches related to the
  // annotated tag
  if ref_value_vec.len() == 1 {
    // Potentially an annotated tag
    let ref_value = ref_value_vec.first().unwrap();
    log::debug!("Found ref in remote git repo:\n{:#?}", ref_value);

    let ref_type: &str =
      ref_value.pointer("/object/type").unwrap().as_str().unwrap();

    let mut r#ref = ref_value["ref"].as_str().unwrap().split("/").skip(1);

    let _ref_1 = r#ref.next();
    let ref_2 = r#ref.next();

    if ref_type == "tag" {
      // Yes, we are processing an annotated tag
      let tag_name = ref_2.unwrap();

      let commit_sha_value = gitea::http_client::get_commit_from_tag(
        ref_value["url"].as_str().unwrap(),
        &tag_name,
        gitea_token,
        shasta_root_cert,
        site_name,
      )
      .await?;

      commit_sha = commit_sha_value
        .pointer("/commit/sha")
        .unwrap()
        .as_str()
        .unwrap();

      let annotated_tag_commit_sha =
        [commit_id.clone(), commit_sha.to_string()];

      ref_value_vec = repo_ref_vec
        .iter()
        .filter(|repo_ref| {
          let ref_sha: String = repo_ref
            .pointer("/object/sha")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

          annotated_tag_commit_sha.contains(&ref_sha)
        })
        .collect();
    }
  }

  for ref_value in ref_value_vec {
    log::debug!("Found ref in remote git repo:\n{:#?}", ref_value);
    let ref_type: &str =
      ref_value.pointer("/object/type").unwrap().as_str().unwrap();
    let mut r#ref = ref_value["ref"].as_str().unwrap().split("/").skip(1);

    let ref_1 = r#ref.next();
    let ref_2 = r#ref.collect::<Vec<_>>().join("/");

    if ref_type == "commit" {
      // either branch or lightweight tag
      if let (Some("heads"), branch_name_aux) = (ref_1, ref_2.clone()) {
        // branch
        branch_name_vec.push(branch_name_aux);
      } else if let (Some("tags"), tag_name_aux) = (ref_1, ref_2) {
        // lightweight tag
        tag_name_vec.push(tag_name_aux);
      }
    } else {
      // annotated tag
      tag_name_vec.push(ref_2);
    }
  }

  if let Some(cfs_config_layer_branch) = &layer.branch {
    branch_name_vec.push(cfs_config_layer_branch.to_string());
  }

  let commit_id_opt = layer.commit.as_ref();

  let gitea_commit_details: serde_json::Value =
    if let Some(commit_id) = commit_id_opt {
      let repo_name = layer
        .clone_url
        .trim_start_matches("https://api-gw-service-nmn.local/vcs/")
        .trim_end_matches(".git");

      gitea::http_client::get_commit_details_from_external_url(
        repo_name,
        commit_id,
        gitea_token,
        shasta_root_cert,
        site_name,
      )
      .await?
    } else {
      serde_json::json!({})
    };

  Ok(LayerDetails::new(
    &layer.name,
    layer
      .clone_url
      .trim_start_matches(
        format!("https://api.cmn.{}.cscs.ch", site_name).as_str(),
      )
      .trim_end_matches(".git"),
    &commit_id,
    gitea_commit_details
      .pointer("/commit/committer/name")
      .unwrap_or(&serde_json::json!("Not defined"))
      .as_str()
      .unwrap(),
    gitea_commit_details
      .pointer("/commit/committer/date")
      .unwrap_or(&serde_json::json!("Not defined"))
      .as_str()
      .unwrap(),
    &branch_name_vec.join(","),
    &tag_name_vec.join(","),
    &layer.playbook,
  ))
}
