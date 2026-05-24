use std::collections::{BTreeMap, HashMap};

use serde_yaml::Value;
use uuid::Uuid;

use crate::{
  bos::{
    session::http_client::v2::types::{BosSession, Operation},
    template::http_client::v2::types::{BootSet, BosSessionTemplate, Cfs},
  },
  cfs,
  common::{self, yaml::yaml_str},
  error::Error,
  hsm,
  ims::{self, image::http_client::types::Link},
  node::utils::validate_target_hsm_members,
};

use super::{
  configuration, image, sessiontemplate,
  images::{
    filter_product_catalog_images, process_sat_file_image_ims_type_recipe,
    process_sat_file_image_old_version_struct,
    process_sat_file_image_product_type_ims_recipe,
  },
};


pub async fn validate_sat_file_session_template_section(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  image_yaml_vec: &[image::Image],
  configuration_yaml_vec: &[configuration::Configuration],
  session_template_yaml_vec: &[sessiontemplate::SessionTemplate],
  hsm_group_available_vec: &[String],
) -> Result<(), Error> {
  // Validate 'session_template' section in SAT file
  log::info!("Validate 'session_template' section in SAT file");
  for session_template_yaml in session_template_yaml_vec {
    // Validate session_template
    log::info!(
      "Validate 'session_template' '{}'",
      session_template_yaml.name
    );

    // Validate user has access to HSM groups in 'session_template' section
    log::info!(
      "Validate 'session_template' '{}' HSM groups",
      session_template_yaml.name
    );

    let bos_session_template_hsm_groups: Vec<String> =
      if let Some(boot_sets_compute) = session_template_yaml
        .bos_parameters
        .boot_sets
        .get("compute")
      {
        boot_sets_compute.node_groups.clone().unwrap_or_default()
      } else if let Some(boot_sets_uan) =
        session_template_yaml.bos_parameters.boot_sets.get("uan")
      {
        boot_sets_uan.node_groups.clone().unwrap_or_default()
      } else {
        return Err(Error::Message("No HSM group found in session_templates section in SAT file".to_string()));
      };

    for hsm_group in bos_session_template_hsm_groups {
      if !hsm_group_available_vec.contains(&hsm_group) {
        return Err(Error::Message(format!(
          "HSM group '{}' in session_templates {} not allowed, List of HSM groups available {:?}. Exit",
          hsm_group, session_template_yaml.name, hsm_group_available_vec
        )));
      }
    }

    // Validate boot image (session_template.image)
    log::info!(
      "Validate 'session_template' '{}' boot image",
      session_template_yaml.name
    );

    if let sessiontemplate::Image::ImageRef(ref_name_to_find) =
      &session_template_yaml.image
    {
      // Validate image_ref (session_template.image.image_ref). Search in SAT file for any
      // image with images[].ref_name
      log::info!("Searching ref_name '{}' in SAT file", ref_name_to_find,);

      let image_ref_name_found = image_yaml_vec
        .iter()
        .any(|image| image.ref_name.eq(&Some(ref_name_to_find).cloned()));

      if !image_ref_name_found {
        return Err(Error::Message(format!(
          "Could not find image ref '{}' in SAT file. Exit",
          ref_name_to_find
        )));
      }
    } else if let sessiontemplate::Image::Ims { ims } =
      &session_template_yaml.image
    {
      match ims {
        sessiontemplate::ImsDetails::Name {
          name: image_name_substr_to_find,
        } => {
          // Validate image name (session_template.image.ims.name). Search in SAT file and CSM
          log::info!(
            "Searching image name '{}' related to session template '{}' in SAT file",
            image_name_substr_to_find,
            session_template_yaml.name
          );

          let mut image_found = image_yaml_vec
            .iter()
            .any(|image| image.name.eq(image_name_substr_to_find));

          if !image_found {
            log::warn!(
              "Image name '{}' not found in SAT file, looking in CSM",
              image_name_substr_to_find
            );
            log::info!(
              "Searching image name '{}' related to session template '{}' in CSM",
              image_name_substr_to_find,
              session_template_yaml.name
            );

            image_found = ims::image::utils::try_get_by_name(
              shasta_token,
              shasta_base_url,
              shasta_root_cert,
              socks5_proxy,
              hsm_group_available_vec,
              image_name_substr_to_find,
              Some(&1),
            )
            .await
            .is_ok();
          }

          if !image_found {
            return Err(Error::Message(format!(
              "Could not find image name '{}' in session_template '{}'. Exit",
              image_name_substr_to_find, session_template_yaml.name
            )));
          }
        }
        sessiontemplate::ImsDetails::Id { id: image_id } => {
          // Validate image id (session_template.image.ims.id) in CSM
          log::info!(
            "Searching image id '{}' related to session template '{}' in CSM",
            image_id,
            session_template_yaml.name
          );

          let image_found = crate::ShastaClient::new(
            shasta_base_url,
            shasta_token,
            shasta_root_cert.to_vec(),
            socks5_proxy.map(str::to_owned),
          )?
          .ims_image_get(Some(image_id.as_str()))
          .await
          .is_ok();

          if !image_found {
            return Err(Error::Message(format!(
              "Could not find image id '{}' in session_template '{}'. Exit",
              image_id, session_template_yaml.name
            )));
          }
        }
      }
    }

    // Validate configuration
    log::info!(
      "Validate 'session_template' '{}' configuration",
      session_template_yaml.name
    );

    log::info!(
      "Searching configuration name '{}' related to session template '{}' in CSM in SAT file",
      session_template_yaml.configuration,
      session_template_yaml.name
    );

    let mut configuration_found =
      configuration_yaml_vec.iter().any(|configuration_yaml| {
        configuration_yaml
          .name
          .eq(&session_template_yaml.configuration)
      });

    if !configuration_found {
      // CFS configuration in session_template not found in SAT file, searching in CSM
      log::warn!("Configuration not found in SAT file, looking in CSM");
      log::info!(
        "Searching configuration name '{}' related to session_template '{}' in CSM",
        session_template_yaml.configuration,
        session_template_yaml.name
      );

      configuration_found = cfs::configuration::http_client::v3::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        socks5_proxy,
        Some(&session_template_yaml.configuration),
      )
      .await
      .is_ok();

      if !configuration_found {
        return Err(Error::Message(format!(
          "Could not find configuration '{}' in session_template '{}'. Exit",
          session_template_yaml.configuration, session_template_yaml.name,
        )));
      }
    }
  }

  Ok(())
}

pub async fn process_session_template_section_in_sat_file(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  ref_name_processed_hashmap: HashMap<String, String>,
  hsm_group_available_vec: &[String],
  sat_file_yaml: Value,
  reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let empty_vec = Vec::new();
  let bos_session_template_list_yaml = sat_file_yaml
    .get("session_templates")
    .and_then(Value::as_sequence)
    .unwrap_or(&empty_vec);

  if bos_session_template_list_yaml.is_empty() {
    log::warn!(
      "No 'session_templates' section found in SAT file. Skipping session template processing"
    );
    return Ok(());
  }

  let mut bos_st_created_vec: Vec<String> = Vec::new();

  for bos_sessiontemplate_yaml in bos_session_template_list_yaml {
    // Get boot image details in BOS sessiontemplate. This is needed to create the BOS
    // sessiontemplate BootSets
    let image_details: ims::image::http_client::types::Image =
      if let Some(bos_sessiontemplate_image) =
        bos_sessiontemplate_yaml.get("image")
      {
        let (image_reference, is_image_id) =
          get_image_reference_from_bos_sessiontemplate_yaml(
            bos_sessiontemplate_image,
            &ref_name_processed_hashmap,
          )?;
        if dry_run {
          let dry_run_mock_image =
            get_image_details_from_bos_sessiontemplate_yaml(
              shasta_token,
              shasta_base_url,
              shasta_root_cert,
              socks5_proxy,
              hsm_group_available_vec,
              &image_reference,
              is_image_id,
            )
            .await
            .unwrap_or_else(|_| {
              // In dry run mode, generate a mock image
              

              if is_image_id {
                // Image reference is an image ID
                ims::image::http_client::types::Image {
                  id: Some(image_reference.to_string()),
                  created: None,
                  name: "dryrun_image".to_string(),
                  link: Some(Link {
                    path: "dryrun_path".to_string(),
                    etag: Some("dryrun_etag".to_string()),
                    r#type: "dryrun_type".to_string(),
                  }),
                  arch: None,
                  metadata: None,
                }
              } else {
                // Image reference is an image name
                ims::image::http_client::types::Image {
                  id: None,
                  created: None,
                  name: image_reference.to_string(),
                  link: Some(Link {
                    path: "dryrun_path".to_string(),
                    etag: Some("dryrun_etag".to_string()),
                    r#type: "dryrun_type".to_string(),
                  }),
                  arch: None,
                  metadata: None,
                }
              }
            });

          log::info!(
            "Dry run mode: Generate mock Image\n{}",
            serde_json::to_string_pretty(&dry_run_mock_image)?
          );

          dry_run_mock_image
        } else {
          get_image_details_from_bos_sessiontemplate_yaml(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            socks5_proxy,
            hsm_group_available_vec,
            &image_reference,
            is_image_id,
          )
          .await?
        }
      } else {
        return Err(Error::Message(
          "ERROR: no 'image' section in session_template.\nExit".to_string(),
        ));
      };

    log::info!("Image with name '{}' found", image_details.name);

    // Get CFS configuration to configure the nodes
    let bos_session_template_configuration_name =
      yaml_str(bos_sessiontemplate_yaml, "configuration")?.to_string();

    // Check CFS configuration exists in CSM
    log::info!(
      "Looking for CFS configuration with name: {}",
      bos_session_template_configuration_name
    );

    if dry_run {
      log::info!(
        "Dry run mode: CFS configuration '{}' found in CSM.",
        bos_session_template_configuration_name
      );
    } else {
      cfs::configuration::http_client::v3::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        socks5_proxy,
        Some(&bos_session_template_configuration_name),
      )
      .await?;
    };

    // let ims_image_name = image_details.name.to_string();
    let image_link = image_details.link.as_ref().ok_or_else(|| {
      Error::Message(format!(
        "IMS image '{}' has no 'link' (no S3 manifest)",
        image_details.name
      ))
    })?;
    let ims_image_etag: &str = image_link.etag.as_deref().ok_or_else(|| {
      Error::Message(format!(
        "IMS image '{}' link has no 'etag'",
        image_details.name
      ))
    })?;
    let ims_image_path: &str = image_link.path.as_ref();
    let ims_image_type: &str = image_link.r#type.as_ref();

    let bos_sessiontemplate_name = bos_sessiontemplate_yaml
      .get("name")
      .and_then(Value::as_str)
      .map(str::to_string)
      .unwrap_or_default();

    let mut boot_set_vec: HashMap<String, BootSet> = HashMap::new();

    let boot_sets_mapping = bos_sessiontemplate_yaml
      .get("bos_parameters")
      .and_then(|bos_parameters| bos_parameters.get("boot_sets"))
      .and_then(Value::as_mapping)
      .ok_or_else(|| {
        Error::Message(
          "SAT file: session_template is missing 'bos_parameters.boot_sets'"
            .to_string(),
        )
      })?;
    for (parameter, boot_set) in boot_sets_mapping {
      let kernel_parameters = boot_set
        .get("kernel_parameters")
        .and_then(Value::as_str)
        .ok_or_else(|| {
          Error::Message(
            "SAT file: boot_set is missing 'kernel_parameters'".to_string(),
          )
        })?;
      let arch_opt = boot_set
        .get("arch")
        .and_then(Value::as_str)
        .map(str::to_string);

      let node_roles_groups_opt: Option<Vec<String>> = boot_set
        .get("node_roles_groups")
        .and_then(Value::as_sequence)
        .and_then(|node_role_groups| {
          node_role_groups
            .iter()
            .map(|hsm_group_value| hsm_group_value.as_str().map(str::to_string))
            .collect()
        });

      // Validate/check user can create BOS sessiontemplates based on node roles. Users
      // with tenant role are not allowed to create BOS sessiontemplates based on node roles
      // however admin tenants are allowed to create BOS sessiontemplates based on node roles
      if !hsm_group_available_vec.is_empty()
        && node_roles_groups_opt
          .clone()
          .is_some_and(|node_roles_groups| !node_roles_groups.is_empty())
      {
        return Err(Error::Message(
          "User type tenant can't user node roles in BOS sessiontemplate. Exit"
            .to_string(),
        ));
      }

      let node_groups_opt: Option<Vec<String>> = boot_set
        .get("node_groups")
        .and_then(Value::as_sequence)
        .and_then(|node_group| {
          node_group
            .iter()
            .map(|hsm_group_value| hsm_group_value.as_str().map(str::to_string))
            .collect()
        });

      //FIXME: Get rid of this by making sure CSM admins don't create HSM groups for system
      //wide operations instead of using roles
      let node_groups_opt = node_groups_opt.map(|node_groups| {
        hsm::group::hacks::filter_system_hsm_group_names(node_groups)
      });

      // Validate/check HSM groups in YAML file session_templates.bos_parameters.boot_sets.<parameter>.node_groups matches with
      // Check hsm groups in SAT file includes the hsm_group_param
      for node_group in node_groups_opt.clone().unwrap_or_default() {
        if !hsm_group_available_vec.contains(&node_group) {
          return Err(Error::Message(format!(
            "User does not have access to HSM group '{}' in SAT file under session_templates.bos_parameters.boot_sets.compute.node_groups section. Exit",
            node_group
          )));
        }
      }

      // Validate user has access to the xnames in the BOS sessiontemplate
      let node_list_opt: Option<Vec<String>> = boot_set
        .get("node_list")
        .and_then(Value::as_sequence)
        .and_then(|node_list| {
          node_list
            .iter()
            .map(|node_value_value| {
              node_value_value.as_str().map(str::to_string)
            })
            .collect()
        });

      // Validate user has access to the list of nodes in BOS sessiontemplate
      if let Some(node_list) = &node_list_opt {
        validate_target_hsm_members(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          socks5_proxy,
          &node_list.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
        )
        .await?;
      }

      let cfs = Cfs {
        configuration: Some(bos_session_template_configuration_name.clone()),
      };

      let rootfs_provider = boot_set
        .get("rootfs_provider")
        .and_then(Value::as_str)
        .map(str::to_string);
      let rootfs_provider_passthrough = boot_set
        .get("rootfs_provider_passthrough")
        .and_then(Value::as_str)
        .map(str::to_string);

      let boot_set = BootSet {
        name: None,
        path: Some(ims_image_path.to_string()),
        r#type: Some(ims_image_type.to_string()),
        etag: Some(ims_image_etag.to_string()),
        kernel_parameters: Some(kernel_parameters.to_string()),
        node_list: node_list_opt,
        node_roles_groups: node_roles_groups_opt,
        node_groups: node_groups_opt,
        rootfs_provider,
        rootfs_provider_passthrough,
        cfs: Some(cfs),
        arch: arch_opt,
      };

      let parameter_str = parameter.as_str().ok_or_else(|| {
        Error::Message(
          "SAT file: boot_set key is not a string".to_string(),
        )
      })?;
      boot_set_vec.insert(parameter_str.to_string(), boot_set);
    }

    let cfs = Cfs {
      configuration: Some(bos_session_template_configuration_name),
    };

    let create_bos_session_template_payload = BosSessionTemplate {
      name: None,
      description: None,
      enable_cfs: Some(true),
      cfs: Some(cfs),
      boot_sets: Some(boot_set_vec),
      links: None,
      tenant: None,
    };

    if dry_run {
      log::info!(
        "Dry run mode: Create BOS sessiontemplate:\n{}",
        serde_json::to_string_pretty(&create_bos_session_template_payload)?
      );

      // Generate a mock name for the BOS session template
      let dry_run_bos_sessiontemplate_name =
        format!("DRYRUN_{}", Uuid::new_v4());
      log::info!(
        "Dry Run Mode: BOS sessiontemplate name '{}' created",
        dry_run_bos_sessiontemplate_name
      );
      bos_st_created_vec.push(dry_run_bos_sessiontemplate_name);
    } else {
      let bos_sessiontemplate = crate::ShastaClient::new(
        shasta_base_url,
        shasta_token,
        shasta_root_cert.to_vec(),
        socks5_proxy.map(str::to_owned),
      )?
      .bos_template_v2_put(
        &create_bos_session_template_payload,
        &bos_sessiontemplate_name,
      )
      .await?;

      log::info!(
        "BOS sessiontemplate name '{}' created",
        bos_sessiontemplate_name
      );

      let created_name = bos_sessiontemplate.name.ok_or_else(|| {
        Error::Message(
          "BOS sessiontemplate API response is missing 'name'".to_string(),
        )
      })?;
      bos_st_created_vec.push(created_name);
    }
  }

  // Create BOS session. Note: reboot operation shuts down the nodes and they may not start
  // up... hence we will split the reboot into 2 operations shutdown and start

  if reboot {
    log::info!("Rebooting");

    for bos_st_name in bos_st_created_vec {
      log::info!(
        "Creating BOS session for BOS sessiontemplate '{}' with action 'reboot'",
        bos_st_name
      );

      // BOS session v2
      let bos_session = BosSession {
        name: None,
        tenant: None,
        operation: Some(Operation::Reboot),
        template_name: bos_st_name.clone(),
        limit: None,
        stage: None,
        include_disabled: None,
        status: None,
        components: None,
      };

      if dry_run {
        log::info!(
          "Dry run mode: Create BOS session:\n{}",
          serde_json::to_string_pretty(&bos_session)?
        );
      } else {
        crate::ShastaClient::new(
          shasta_base_url,
          shasta_token,
          shasta_root_cert.to_vec(),
          socks5_proxy.map(str::to_owned),
        )?
        .bos_session_v2_post(bos_session)
        .await?;
      }
    }
  }

  // Audit
  let user = common::jwt_ops::get_name(shasta_token)?;
  let username = common::jwt_ops::get_preferred_username(shasta_token)?;

  log::info!(target: "app::audit", "User: {} ({}) ; Operation: Apply cluster", user, username);

  Ok(())
}

/// Returns image reference related to a session template in SAT file.
/// An image refenrece can be:
///     - image_name
///     - image_id
/// Image names are supposed to be fetched using 'get_fuzzy' function (so we increase the probablity of finding the image in CSM if it was created using 'sat bootprep --overwrite-images') while image ids can be fetched
/// by just 'get' function
/// This function returns a tuple with the image reference and a boolean indicating whether the image is
/// an image id or not
fn get_image_reference_from_bos_sessiontemplate_yaml(
  bos_sessiontemplate_image: &Value,
  ref_name_processed_hashmap: &HashMap<String, String>,
) -> Result<(String, bool), Error> {
  if let Some(bos_sessiontemplate_image_ims) =
    bos_sessiontemplate_image.get("ims")
  {
    // Get boot image to configure the nodes
    if let Some(bos_session_template_image_ims_name) =
      bos_sessiontemplate_image_ims.get("name")
    {
      // BOS sessiontemplate boot image defined by name
      let image_name = bos_session_template_image_ims_name
        .as_str()
        .ok_or_else(|| {
          Error::Message(
            "SAT file: session_template image.ims.name is not a string"
              .to_string(),
          )
        })?
        .to_string();

      Ok((image_name, false))
    } else if let Some(bos_session_template_image_ims_id) =
      bos_sessiontemplate_image_ims.get("id")
    {
      // BOS sessiontemplate boot image defined by id
      let image_id = bos_session_template_image_ims_id
        .as_str()
        .ok_or_else(|| {
          Error::Message(
            "SAT file: session_template image.ims.id is not a string"
              .to_string(),
          )
        })?
        .to_string();

      Ok((image_id, true))
    } else {
      Err(Error::Message("ERROR: neither 'image.ims.name' nor 'image.ims.id' fields defined in session_template.".to_string()))
    }
  } else if let Some(bos_session_template_image_image_ref) =
    bos_sessiontemplate_image.get("image_ref")
  {
    // BOS sessiontemplate boot image defined by image_ref
    let image_ref = bos_session_template_image_image_ref
      .as_str()
      .ok_or_else(|| {
        Error::Message(
          "SAT file: session_template image.image_ref is not a string"
            .to_string(),
        )
      })?
      .to_string();

    let image_id = ref_name_processed_hashmap
      .get(&image_ref)
      .cloned()
      .ok_or_else(|| {
        Error::Message(format!(
          "SAT file: image_ref '{}' not found in processed image set",
          image_ref
        ))
      })?;

    Ok((image_id, true))
  } else if let Some(image_name_substring) = bos_sessiontemplate_image.as_str()
  {
    let image_name = image_name_substring;
    // Backward compatibility
    // Get base image details

    Ok((image_name.to_string(), false))
  } else {
    Err(Error::Message("ERROR: neither 'image.ims' nor 'image.image_ref' nor 'image.<image id>' sections found in session_template.image.\nExit".to_string()))
  }
}

async fn get_image_details_from_bos_sessiontemplate_yaml(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  hsm_group_available_vec: &[String],
  image_reference: &str,
  is_image_id: bool,
) -> Result<ims::image::http_client::types::Image, Error> {
  

  if is_image_id {
    crate::ShastaClient::new(
      shasta_base_url,
      shasta_token,
      shasta_root_cert.to_vec(),
      socks5_proxy.map(str::to_owned),
    )?
    .ims_image_get(Some(image_reference))
    .await
    .and_then(|image_vec| {
      image_vec.first().cloned().ok_or_else(|| {
        Error::Message(format!(
          "Image '{}' not found in CSM",
          image_reference
        ))
      })
    })
  } else {

    ims::image::utils::try_get_by_name(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      hsm_group_available_vec,
      image_reference,
      Some(&1),
    )
    .await
    .and_then(|image_vec| {
      image_vec.first().cloned().ok_or_else(|| {
        Error::Message(format!(
          "Image '{}' not found in CSM",
          image_reference
        ))
      })
    })
  }
}


pub(super) async fn get_base_image_id_from_sat_file_image_yaml(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  // image_yaml: &Value,
  image_yaml: &image::Image,
  _ref_name_image_id_hashmap: &HashMap<String, String>,
  cray_product_catalog: &BTreeMap<String, String>,
  image_name: &String,
  dry_run: bool,
) -> Result<String, Error> {
  // Get/process base image
  // if let Some(sat_file_image_ims_value_yaml) = image_yaml.get("ims") {
  let base_image_id: String = if let image::BaseOrIms::Ims { ims } =
    &image_yaml.base_or_ims
  {
    // ----------- BASE IMAGE - BACKWARD COMPATIBILITY WITH PREVIOUS SAT FILE
    log::info!(
      "SAT file - 'image.ims' job ('images' section in SAT file is outdated - switching to backward compatibility)"
    );

    process_sat_file_image_old_version_struct(ims)?
  // } else if let Some(sat_file_image_base_value_yaml) = image_yaml.get("base") {
  } else if let image::BaseOrIms::Base { base } = &image_yaml.base_or_ims {
    /* if let Some(sat_file_image_base_image_ref_value_yaml) =
      sat_file_image_base_value_yaml.get("image_ref")
    { */
    if let image::Base::ImageRef { image_ref } = base {
      log::info!("SAT file - 'image.base.image_ref' job");

      image_ref.clone()
    /* } else if let Some(sat_file_image_base_ims_value_yaml) =
      sat_file_image_base_value_yaml.get("ims")
    { */
    } else if let image::Base::Ims { ims } = base {
      if let image::ImageBaseIms::NameType { name, r#type } = ims {
        log::info!("SAT file - 'image.base.ims' job");
        if r#type == "recipe" {
          log::info!("SAT file - 'image.base.ims' job of type 'recipe'");

          process_sat_file_image_ims_type_recipe(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            socks5_proxy,
            name,
            image_name,
            dry_run,
          )
          .await?
        } else {
          return Err(Error::Message(
            "Can't process SAT file 'images.base.ims' is missing. Exit"
              .to_string(),
          ));
        }
      } else if let image::Base::Ims { ims } = base {
        #[allow(clippy::collapsible_match)]
        if let image::ImageBaseIms::IdType { id, r#type } = ims {
          if r#type == "image" {
            log::info!("SAT file - 'image.base.ims' job of type 'image'");

            id.to_string()
          } else {
            return Err(Error::Message(
              "Can't process SAT file 'images.base.ims' is missing. Exit"
                .to_string(),
            ));
          }
        } else {
          return Err(Error::Message(
            "Can't process SAT file 'images.base.ims' is missing. Exit"
              .to_string(),
          ));
        }
      } else {
        return Err(Error::Message(
          "Can't process SAT file 'images.base.ims' is missing. Exit"
            .to_string(),
        ));
      }
    // ----------- BASE IMAGE - CRAY PRODUCT CATALOG
    /* } else if let Some(sat_file_image_base_product_value_yaml) =
      sat_file_image_base_value_yaml.get("product")
    { */
    } else if let image::Base::Product { product } = base {
      log::info!("SAT file - 'image.base.product' job");
      // Base image created from a cray product
      let product_name = &product.name;

      let product_version = product.version.as_ref().ok_or_else(|| {
        Error::Message(format!(
          "SAT file: image base.product '{}' is missing 'version'",
          product_name
        ))
      })?;

      let product_type = &product.r#type;

      let product_image_map = serde_yaml::from_str::<serde_json::Value>(
        &cray_product_catalog[product_name],
      )?[product_version][product_type]
        .as_object()
        .cloned()
        .ok_or_else(|| {
          Error::Message(format!(
            "Cray product catalog: '{}.{}.{}' is missing or not an object",
            product_name, product_version, product_type
          ))
        })?;

      let image_id = if let Some(filter) = product.filter.as_ref() {
        filter_product_catalog_images(
          filter,
          product_image_map.clone(),
          image_name,
        )?
      } else {
        // There is no 'image.product.filter' value defined in SAT file. Check Cray
        // product catalog only has 1 image. Othewise fail
        log::info!(
          "No 'image.product.filter' defined in SAT file. Checking Cray product catalog only/must have 1 image"
        );
        product_image_map
          .values()
          .next()
          .and_then(|value| value.get("id"))
          .and_then(serde_json::Value::as_str)
          .map(str::to_string)
          .ok_or_else(|| {
            Error::Message(format!(
              "Cray product catalog: '{}.{}.{}' has no entries with an 'id' field",
              product_name, product_version, product_type
            ))
          })?
      };

      // ----------- BASE IMAGE - CRAY PRODUCT CATALOG TYPE RECIPE
      if product_type == "recipes" {
        // Create base image from an IMS job (the 'id' field in
        // images[].base.product.id is the id of the IMS recipe used to
        // build the new base image)

        log::info!("SAT file - 'image.base.product' job based on IMS recipes");

        let product_recipe_id = image_id.clone();

        process_sat_file_image_product_type_ims_recipe(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          socks5_proxy,
          &product_recipe_id,
          image_name,
          dry_run,
        )
        .await?

        // ----------- BASE IMAGE - CRAY PRODUCT CATALOG TYPE IMAGE
      } else if product_type == "images" {
        // Base image already created and its id is available in the Cray
        // product catalog

        log::info!("SAT file - 'image.base.product' job based on IMS images");

        log::info!("Getting base image id from Cray product catalog");

        image_id
      } else {
        return Err(Error::Message(
          "Can't process SAT file, field 'images.base.product.type' must be either 'images' or 'recipes'. Exit".to_string(),
        ));
      }
    } else {
      return Err(Error::Message(
        "Can't process SAT file 'images.base.product' is missing. Exit"
          .to_string(),
      ));
    }
  } else {
    return Err(Error::Message(
      "Can't process SAT file 'images.base' is missing. Exit".to_string(),
    ));
  };

  Ok(base_image_id)
}
