use std::{collections::HashMap, time::Instant};

use serde_yaml::Value;

use crate::{
  cfs::{
    self,
    configuration::http_client::v2::types::cfs_configuration_response::CfsConfigurationResponse,
  },
  commands::{
    apply_hw_cluster_pin,
    i_apply_sat_file::utils::{self, configuration, image, sessiontemplate},
  },
  common::kubernetes::{self},
  error::Error,
  hsm::group::utils::update_hsm_group_members,
  ims,
};

#[deprecated(
  since = "v0.86.2",
  note = "this function prints cfs session logs to stdout"
)]
pub async fn exec(
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
  // GET DATA
  //
  // Get data from SAT YAML file
  //
  // Get hardware pattern from SAT YAML file
  let hardware_yaml_value_vec_opt = sat_template_file_yaml
    .get("hardware")
    .and_then(Value::as_sequence);

  // Get CFS configurations from SAT YAML file
  let configuration_yaml_vec_opt = sat_template_file_yaml
    .get("configurations")
    .and_then(Value::as_sequence);

  // Get images from SAT YAML file
  let image_yaml_vec_opt = sat_template_file_yaml
    .get("images")
    .and_then(Value::as_sequence);

  // Get images from SAT YAML file
  let bos_session_template_yaml_vec_opt = sat_template_file_yaml
    .get("session_templates")
    .and_then(Value::as_sequence);

  // Get k8s credentials needed to check HPE/Cray product catalog in k8s
  let kube_client =
    kubernetes::get_client(k8s_api_url, shasta_k8s_secrets).await?;

  // Get HPE product catalog from k8s
  let cray_product_catalog =
    kubernetes::try_get_configmap(kube_client, "cray-product-catalog").await?;

  // Get data from CSM
  //
  let start = Instant::now();
  log::info!("Fetching data from the backend...");
  let (configuration_vec, image_vec, ims_recipe_vec) = tokio::try_join!(
    cfs::configuration::http_client::v2::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert
    ),
    ims::image::http_client::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert
    ),
    ims::recipe::http_client::get(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      None
    )
  )?;
  let duration = start.elapsed();
  log::info!(
    "Time elapsed to fetch information from backend: {:?}",
    duration
  );

  let configuration_struct_vec: Vec<configuration::Configuration> =
    serde_yaml::from_value(serde_yaml::Value::Sequence(
      configuration_vec
        .iter()
        .map(|c| serde_yaml::to_value(c).unwrap())
        .collect(),
    ))?;

  let image_struct_vec: Vec<image::Image> =
    serde_yaml::from_value(serde_yaml::Value::Sequence(
      image_vec
        .iter()
        .map(|i| serde_yaml::to_value(i).unwrap())
        .collect(),
    ))?;

  let bos_session_template_struct_vec: Vec<sessiontemplate::SessionTemplate> =
    if let Some(bos_session_template_yaml_vec) =
      bos_session_template_yaml_vec_opt
    {
      serde_yaml::from_value(serde_yaml::Value::Sequence(
        bos_session_template_yaml_vec
          .iter()
          .map(|s| serde_yaml::to_value(s).unwrap())
          .collect(),
      ))?
    } else {
      Vec::new()
    };

  // VALIDATION
  //
  // Validate 'configurations' section
  utils::validate_sat_file_configurations_section(
    &configuration_struct_vec,
    &image_struct_vec,
    &bos_session_template_struct_vec,
  )?;
  /* utils::validate_sat_file_configurations_section(
    configuration_yaml_vec_opt,
    image_yaml_vec_opt,
    bos_session_template_yaml_vec_opt,
  )?; */

  // Validate 'images' section
  /* utils::validate_sat_file_images_section(
    image_yaml_vec_opt.unwrap_or(&Vec::new()),
    configuration_yaml_vec_opt.unwrap_or(&Vec::new()),
    hsm_group_available_vec,
    &cray_product_catalog,
    image_vec,
    configuration_vec,
    ims_recipe_vec,
  )?; */
  utils::validate_sat_file_images_section(
    &image_struct_vec,
    &configuration_struct_vec,
    hsm_group_available_vec,
    &cray_product_catalog,
    image_vec,
    configuration_vec,
    ims_recipe_vec,
  )?;

  // Validate 'session_template' section
  utils::validate_sat_file_session_template_section(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    &image_struct_vec,
    &configuration_struct_vec,
    &bos_session_template_struct_vec,
    hsm_group_available_vec,
  )
  .await?;

  // PROCESS SAT FILE
  //
  // Process "hardware" section in SAT file
  log::info!("hardware pattern: {:?}", hardware_yaml_value_vec_opt);

  // Process "clusters" section
  //
  if let Some(hw_component_pattern_vec) = hardware_yaml_value_vec_opt {
    for hw_component_pattern in hw_component_pattern_vec {
      let target_hsm_group_name = hw_component_pattern
        .get("target")
        .and_then(|v| v.as_str())
        .unwrap();
      let parent_hsm_group_name = hw_component_pattern
        .get("parent")
        .and_then(Value::as_str)
        .unwrap();

      if let Some(pattern) =
        hw_component_pattern.get("pattern").and_then(Value::as_str)
      {
        log::info!(
          "Processing hw component pattern for '{}' for target HSM group '{}' and parent HSM group '{}'",
          pattern,
          target_hsm_group_name,
          parent_hsm_group_name
        );
        // When applying a SAT file, I'm assuming the user doesn't want to create new HSM groups or delete empty parent hsm groups
        // But this could be changed.
        if dry_run {
          println!("Dry run: Create HSM groups based on hardware pattern");
        } else {
          apply_hw_cluster_pin::command::exec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &target_hsm_group_name,
            parent_hsm_group_name,
            pattern,
            true,
            false,
            false,
          )
          .await?;
        }
      } else if let Some(nodes) = hw_component_pattern
        .get("nodespattern")
        .and_then(Value::as_str)
      {
        let hsm_group_members_vec: Vec<String> =
          crate::hsm::group::utils::get_member_vec_from_hsm_name_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &[target_hsm_group_name.to_string()],
          )
          .await?;
        let new_target_hsm_group_members_vec: Vec<String> = nodes
          .split(',')
          .filter(|node| !hsm_group_members_vec.contains(&node.to_string()))
          .map(str::to_string)
          .collect();

        log::info!(
          "Processing new nodes '{}' for target HSM group '{}'",
          nodes,
          target_hsm_group_name,
        );

        if dry_run {
          println!(
            "Dry Run mode: Update HSM group '{}' members to:\n{:?}",
            target_hsm_group_name, new_target_hsm_group_members_vec
          );
        } else {
          update_hsm_group_members(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &target_hsm_group_name,
            &hsm_group_members_vec
              .iter()
              .map(String::as_str)
              .collect::<Vec<&str>>(),
            &new_target_hsm_group_members_vec
              .iter()
              .map(String::as_str)
              .collect::<Vec<&str>>(),
          )
          .await?;
        }
      }
    }
  }

  // Process "configurations" section in SAT file
  //
  log::info!("Process configurations section in SAT file");
  let mut cfs_configuration_value_vec = Vec::new();

  let mut cfs_configuration_name_vec = Vec::new();

  for configuration_yaml in configuration_yaml_vec_opt.unwrap_or(&vec![]).iter()
  {
    let cfs_configuration: CfsConfigurationResponse =
      utils::create_cfs_configuration_from_sat_file(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        gitea_base_url,
        gitea_token,
        &cray_product_catalog,
        configuration_yaml,
        // tag,
        dry_run,
        site_name,
        overwrite,
      )
      .await?;

    let cfs_configuration_name = cfs_configuration.name.to_string();

    println!("CFS configuration '{}' created", cfs_configuration_name);

    cfs_configuration_name_vec.push(cfs_configuration_name.clone());

    cfs_configuration_value_vec.push(cfs_configuration.clone());
  }

  // Process "images" section in SAT file
  //
  log::info!("Process images section in SAT file");
  // List of image.ref_name already processed
  let mut ref_name_processed_hashmap: HashMap<String, String> = HashMap::new();

  let cfs_session_created_hashmap: HashMap<String, image::Image> =
    utils::i_import_images_section_in_sat_file(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      vault_base_url,
      site_name,
      k8s_api_url,
      &mut ref_name_processed_hashmap,
      &image_struct_vec,
      &cray_product_catalog,
      ansible_verbosity_opt,
      ansible_passthrough_opt,
      debug_on_failure,
      dry_run,
      watch_logs,
      timestamps,
    )
    .await?;

  log::info!(
    "Images created: {:?}",
    cfs_session_created_hashmap.keys().collect::<Vec<&String>>()
  );

  // Process "session_templates" section in SAT file
  //
  log::info!("Process session_template section in SAT file");
  utils::process_session_template_section_in_sat_file(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    ref_name_processed_hashmap,
    hsm_group_available_vec,
    sat_template_file_yaml,
    reboot,
    dry_run,
  )
  .await?;

  Ok(())
}
