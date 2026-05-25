//! Entry-point function for the apply-SAT-file workflow.

use std::{collections::HashMap, time::Instant};

use serde_yaml::Value;

use crate::{
  cfs::configuration::http_client::v2::types::cfs_configuration_response::CfsConfigurationResponse,
  commands::{
    apply_hw_cluster_pin,
    i_apply_sat_file::utils::{
      self, SatFile, configuration, image, sessiontemplate,
    },
  },
  common::kubernetes::{self},
  error::Error,
  hsm::group::utils::update_hsm_group_members,
};

/// Apply a SAT (System Admin Toolkit) template file against a Shasta system.
///
/// Parses `sat_template_file_yaml`, validates each section against the
/// current state of CSM and the HPE Cray product catalog (read from
/// Kubernetes), and then realises the file by:
///
/// 1. Applying any `hardware` patterns to HSM groups (either component
///    patterns via [`apply_hw_cluster_pin`] or explicit `nodespattern`
///    membership updates).
/// 2. Creating CFS configurations for each entry in `configurations`.
/// 3. Importing every image in `images` (building it through IMS/CFS).
/// 4. Creating BOS session templates from `session_templates`, optionally
///    rebooting the targeted nodes.
///
/// # Arguments
///
/// - `sat_template_file_yaml` — the parsed SAT file as YAML.
/// - `hsm_group_available_vec` — HSM groups the caller is allowed to
///   target; used to reject SAT files that reference out-of-scope groups.
/// - `shasta_k8s_secrets` / `k8s_api_url` — credentials for the in-cluster
///   `cray-product-catalog` ConfigMap lookup.
/// - `dry_run` — when `true`, validates and logs the intended actions
///   without mutating CSM.
/// - `overwrite` — replace existing CFS configurations / images with the
///   same name instead of failing.
/// - `reboot` — after creating BOS session templates, also reboot the
///   target nodes through them.
///
/// # Errors
///
/// Returns [`Error`] if the SAT file is malformed, validation against the
/// live CSM state fails, or any underlying API call (CFS, IMS, BOS, HSM,
/// Kubernetes) fails.
///
/// # Deprecated
///
/// Marked deprecated since 0.86.2 because it streams CFS session logs
/// directly to stdout, which is unsuitable for library consumers; prefer
/// composing the lower-level `cfs`/`bos`/`ims` APIs in new code.
#[deprecated(
  since = "0.86.2",
  note = "this function prints cfs session logs to stdout"
)]
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
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
  let _image_yaml_vec_opt = sat_template_file_yaml
    .get("images")
    .and_then(Value::as_sequence);

  // Get images from SAT YAML file
  let _bos_session_template_yaml_vec_opt = sat_template_file_yaml
    .get("session_templates")
    .and_then(Value::as_sequence);

  // Get k8s credentials needed to check HPE/Cray product catalog in k8s
  let kube_client =
    kubernetes::get_client(k8s_api_url, shasta_k8s_secrets, socks5_proxy)
      .await?;

  // Get HPE product catalog from k8s
  let cray_product_catalog =
    kubernetes::try_get_configmap(kube_client, "cray-product-catalog").await?;

  // Get data from CSM
  //
  let start = Instant::now();
  log::info!("Fetching data from the backend...");
  let shasta_client = crate::ShastaClient::new(
    shasta_base_url,
    shasta_root_cert.to_vec(),
    socks5_proxy.map(str::to_owned),
  )?;
  let (configuration_vec, image_vec, ims_recipe_vec) = tokio::try_join!(
    shasta_client.cfs_configuration_v2_get_all(shasta_token),
    shasta_client.ims_image_get_all(shasta_token),
    shasta_client.ims_recipe_get(shasta_token, None),
  )?;

  let duration = start.elapsed();
  log::info!(
    "Time elapsed to fetch information from backend: {:?}",
    duration
  );

  let sat_file_struct: SatFile =
    serde_yaml::from_str(&serde_yaml::to_string(&sat_template_file_yaml)?)?;

  let configuration_struct_vec: Vec<configuration::Configuration> =
    sat_file_struct.configurations.unwrap_or_default();

  let image_struct_vec: Vec<image::Image> =
    sat_file_struct.images.unwrap_or_default();

  let bos_session_template_struct_vec: Vec<sessiontemplate::SessionTemplate> =
    sat_file_struct.session_templates.unwrap_or_default();

  // VALIDATION
  //
  // Validate 'configurations' section
  utils::validate_sat_file_configurations_section(
    &configuration_struct_vec,
    &image_struct_vec,
    &bos_session_template_struct_vec,
  )?;

  // Validate 'images' section
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
    socks5_proxy,
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
        .ok_or_else(|| {
          Error::Message(
            "SAT file: hardware pattern missing 'target'".to_string(),
          )
        })?;
      let parent_hsm_group_name = hw_component_pattern
        .get("parent")
        .and_then(Value::as_str)
        .ok_or_else(|| {
          Error::Message(
            "SAT file: hardware pattern missing 'parent'".to_string(),
          )
        })?;

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
          log::info!("Dry run: Create HSM groups based on hardware pattern");
        } else {
          apply_hw_cluster_pin::command::exec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            socks5_proxy,
            target_hsm_group_name,
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
            socks5_proxy,
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
          log::info!(
            "Dry Run mode: Update HSM group '{}' members to:\n{:?}",
            target_hsm_group_name,
            new_target_hsm_group_members_vec
          );
        } else {
          update_hsm_group_members(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            socks5_proxy,
            target_hsm_group_name,
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
        socks5_proxy,
        gitea_base_url,
        gitea_token,
        &cray_product_catalog,
        configuration_yaml,
        dry_run,
        site_name,
        overwrite,
      )
      .await?;

    let cfs_configuration_name = cfs_configuration.name.to_string();

    log::info!("CFS configuration '{}' created", cfs_configuration_name);

    cfs_configuration_name_vec.push(cfs_configuration_name.clone());

    cfs_configuration_value_vec.push(cfs_configuration.clone());
  }

  // Process "images" section in SAT file
  //
  log::info!("Process images section in SAT file");
  // List of image.ref_name already processed
  let mut ref_name_processed_hashmap: HashMap<String, String> = HashMap::new();

  #[allow(deprecated)]
  let cfs_session_created_hashmap: HashMap<String, image::Image> =
    utils::i_import_images_section_in_sat_file(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
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
    socks5_proxy,
    ref_name_processed_hashmap,
    hsm_group_available_vec,
    sat_template_file_yaml,
    reboot,
    dry_run,
  )
  .await?;

  Ok(())
}
