use core::time;
use std::collections::HashMap;
use std::time::Instant;

use chrono::NaiveDateTime;

use crate::{
  bos::{self},
  bss::{self, types::BootParameters},
  cfs::{
    self,
    configuration::http_client::v2::types::cfs_configuration_response::CfsConfigurationResponse,
    session::http_client::v2::types::CfsSessionGetResponse,
  },
  common,
  error::Error,
  ims,
};

pub async fn get_data_to_delete(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hsm_name_available_vec: &[&str],
  configuration_name_pattern_opt: Option<&str>,
  since_opt: Option<NaiveDateTime>,
  until_opt: Option<NaiveDateTime>,
) -> Result<
  (
    Vec<CfsSessionGetResponse>,
    Vec<(String, String, String)>,
    Vec<String>,
    Vec<String>,
    Vec<(String, String, String)>,
    Vec<CfsConfigurationResponse>,
  ),
  Error,
> {
  // COLLECT SITE WIDE DATA FOR VALIDATION
  //
  let xname_from_groups_vec =
    crate::hsm::group::utils::get_member_vec_from_hsm_name_vec(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_name_available_vec,
    )
    .await?;

  let start = Instant::now();
  log::info!("Fetching data from the backend...");
  let (
    cfs_component_vec,
    mut cfs_configuration_vec,
    cfs_session_vec,
    bos_sessiontemplate_vec,
    bss_bootparameters_vec,
  ) = tokio::try_join!(
    cfs::component::http_client::v2::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert
    ),
    cfs::configuration::http_client::v2::get_all(
      shasta_token,
      shasta_base_url,
      shasta_root_cert
    ),
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
    bss::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
  )?;
  let duration = start.elapsed();
  log::info!(
    "Time elapsed to fetch information from backend: {:?}",
    duration
  );

  let mut cfs_session_to_delete_vec = cfs_session_vec.clone();
  let mut bos_sessiontemplate_to_delete_vec = bos_sessiontemplate_vec.clone();

  let keep_generic_sessions = common::jwt_ops::is_user_admin(shasta_token);

  // Filter CFS configurations related to HSM group, configuration name or configuration name
  // pattern
  cfs::configuration::utils::filter(
    &mut cfs_configuration_vec,
    &xname_from_groups_vec
      .iter()
      .map(|s| s.as_str())
      .collect::<Vec<&str>>(),
    &mut cfs_session_to_delete_vec,
    &mut bos_sessiontemplate_to_delete_vec,
    &cfs_component_vec,
    configuration_name_pattern_opt,
    hsm_name_available_vec,
    since_opt,
    until_opt,
    None,
    keep_generic_sessions,
  )?;

  // Get CFS configurations related with BOS sessiontemplate
  let cfs_configuration_name_from_bos_sessiontemplate_value_iter =
    bos_sessiontemplate_to_delete_vec
      .iter()
      .map(|bos_sessiontemplate| {
        bos_sessiontemplate.get_configuration().unwrap_or_default()
      });

  // Get CFS configurations related with CFS sessions
  let cfs_configuration_name_from_cfs_sessions = cfs_session_to_delete_vec
    .iter()
    .map(|cfs_session| cfs_session.configuration_name().unwrap_or_default());

  // Get list of CFS configuration names related to CFS sessions and BOS sessiontemplates
  let mut cfs_configuration_name_vec: Vec<String> =
    cfs_configuration_name_from_bos_sessiontemplate_value_iter
      .chain(cfs_configuration_name_from_cfs_sessions)
      .map(str::to_string)
      .collect();

  cfs_configuration_name_vec.sort();
  cfs_configuration_name_vec.dedup();

  // Get list of CFS configuration serde values related to CFS sessions and BOS
  // sessiontemplates
  cfs_configuration_vec.retain(|cfs_configuration| {
    cfs_configuration_name_vec.contains(&cfs_configuration.name)
  });

  // Get image ids from CFS sessions related to CFS configuration to delete
  let mut image_id_vec: Vec<String> =
    cfs::session::utils::images_id_from_cfs_session(&cfs_session_to_delete_vec)
      .map(str::to_string)
      .collect();

  log::info!("Image ids to delete: {:?}", image_id_vec);

  // Get list of CFS session name, CFS configuration name and image id for CFS sessions which
  // created an image
  let cfs_session_cfs_configuration_image_id_tuple_vec: Vec<(
    String,
    String,
    String,
  )> = cfs_session_to_delete_vec
    .iter()
    .filter(|cfs_session| cfs_session.first_result_id().is_some())
    .map(|cfs_session| {
      (
        cfs_session.name.clone(),
        cfs_session
          .configuration_name()
          .unwrap_or_default()
          .to_string(),
        cfs_session
          .first_result_id()
          .unwrap_or_default()
          .to_string(),
      )
    })
    .collect();

  // Get list of BOS sessiontemplate name, CFS configuration name and image ids for compute nodes
  let mut bos_sessiontemplate_cfs_configuration_image_id_tuple_vec: Vec<(
    String,
    String,
    String,
  )> = Vec::new();

  for bos_sessiontemplate in &bos_sessiontemplate_to_delete_vec {
    let bos_sessiontemplate_name: &str =
      bos_sessiontemplate.name.as_deref().unwrap_or_default();

    let cfs_configuration_name: &str =
      bos_sessiontemplate.get_configuration().unwrap_or_default();

    for image_id in bos_sessiontemplate.images_id() {
      bos_sessiontemplate_cfs_configuration_image_id_tuple_vec.push((
        bos_sessiontemplate_name.to_string(),
        cfs_configuration_name.to_string(),
        image_id.to_string(),
      ));
    }
  }

  // Group image ids by CFS configuration names
  let mut cfs_configuration_image_id: HashMap<&str, Vec<&str>> = HashMap::new();

  for (_, cfs_configuration, image_id) in
    &bos_sessiontemplate_cfs_configuration_image_id_tuple_vec
  {
    cfs_configuration_image_id
      .entry(cfs_configuration)
      .and_modify(|image_vec| image_vec.push(image_id))
      .or_insert(vec![image_id]);
  }

  for (_, cfs_configuration, image_id) in
    &cfs_session_cfs_configuration_image_id_tuple_vec
  {
    cfs_configuration_image_id
      .entry(cfs_configuration)
      .and_modify(|image_vec| image_vec.push(image_id))
      .or_insert(vec![image_id]);
  }

  // VALIDATION
  //
  let mut cfs_configuration_name_used_to_configure_nodes_vec: Vec<String> =
    Vec::new();
  let mut image_id_used_to_boot_nodes_vec: Vec<String> = Vec::new();

  // We can't allow any data deletion operation which can jeopardize the system stability,
  // therefore we will filter the list of the CFS configurations and Images used to configure or boot nodes
  for (cfs_configuration_name, mut image_id_vec) in cfs_configuration_image_id {
    let mut nodes_using_cfs_configuration_as_dessired_configuration_vec =
      cfs_component_vec
        .iter()
        .filter(|cfs_component| {
          cfs_component
            .desired_config
            .as_ref()
            .unwrap()
            .eq(cfs_configuration_name)
        })
        .map(|cfs_component| cfs_component.id.as_ref().unwrap().as_str())
        .collect::<Vec<&str>>();

    if !nodes_using_cfs_configuration_as_dessired_configuration_vec.is_empty() {
      cfs_configuration_name_used_to_configure_nodes_vec
        .push(cfs_configuration_name.to_string());

      nodes_using_cfs_configuration_as_dessired_configuration_vec.sort();

      eprintln!(
        "CFS configuration '{}' can't be deleted. Reason:\nCFS configuration '{}' used as desired configuration for nodes: {}",
        cfs_configuration_name, cfs_configuration_name, nodes_using_cfs_configuration_as_dessired_configuration_vec.join(", "));
    }

    image_id_vec.dedup();

    for image_id in &image_id_vec {
      let node_vec =
        get_node_vec_booting_image(image_id, &bss_bootparameters_vec);

      if !node_vec.is_empty() {
        image_id_used_to_boot_nodes_vec.push(image_id.to_string());
        eprintln!(
          "Image '{}' used to boot nodes: {}",
          image_id,
          node_vec.join(", ")
        );
      }
    }
  }

  // Get final list of CFS configuration serde values related to CFS sessions and BOS
  // sessiontemplates and excluding the CFS sessions to keep (in case user decides to
  // force the deletion operation)
  cfs_configuration_vec.retain(|cfs_configuration_value| {
    !cfs_configuration_name_used_to_configure_nodes_vec
      .contains(&cfs_configuration_value.name)
  });

  let cfs_session_cfs_configuration_image_id_tuple_filtered_vec: Vec<(
    String,
    String,
    String,
  )>;
  let bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec: Vec<
    (String, String, String),
  >;

  // EVALUATE IF NEED TO CONTINUE.
  // CHECK IF ANY CFS CONFIGURAION OR IMAGE IS CURRENTLY USED TO CONFIGURE OR BOOT NODES
  if !cfs_configuration_name_used_to_configure_nodes_vec.is_empty()
    || !image_id_used_to_boot_nodes_vec.is_empty()
  {
    // There are CFS configuraions or Images currently used by nodes. Better to be safe and
    // stop the process
    log::error!(
      "User trying to delete configurations or images used by other clusters/nodes"
    );
    return Err(
      Error::ConfigurationUsedAsRuntimeConfigurationOrUsedToBuildBootImageUsed,
    );
  } else {
    // We are safe to delete, none of the data selected for deletion is currently used as
    // neither configure nor boot the nodes
    cfs_configuration_name_vec.retain(|cfs_configuration_name| {
      !cfs_configuration_name_used_to_configure_nodes_vec
        .contains(&cfs_configuration_name.to_string())
    });

    image_id_vec.retain(|image_id| {
      !image_id_used_to_boot_nodes_vec.contains(&image_id.to_string())
    });

    cfs_session_cfs_configuration_image_id_tuple_filtered_vec =
      cfs_session_cfs_configuration_image_id_tuple_vec
        .into_iter()
        .filter(|(_, cfs_configuration_name, image_id)| {
          !cfs_configuration_name_used_to_configure_nodes_vec
            .contains(&cfs_configuration_name.to_string())
            && !image_id_used_to_boot_nodes_vec.contains(&image_id.to_string())
        })
        .collect();

    bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec =
      bos_sessiontemplate_cfs_configuration_image_id_tuple_vec
        .into_iter()
        .filter(|(_, cfs_configuration_name, image_id)| {
          !cfs_configuration_name_used_to_configure_nodes_vec
            .contains(cfs_configuration_name)
            && !image_id_used_to_boot_nodes_vec.contains(image_id)
        })
        .collect();
  }

  // Return ERROR IF THERE IS NO DATA TO DELETE
  if image_id_vec.is_empty()
    && cfs_session_cfs_configuration_image_id_tuple_filtered_vec.is_empty()
    && bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec
      .is_empty()
  {
    // We can't decide if CFS configuration and derivatives can be deleted.
    log::error!(
        "Delete configuration - Not enough information to proceed. Could not find information related to CFS configurations '{}'",
        cfs_configuration_name_vec.join(", ")
      );
    return Err(Error::ConfigurationDerivativesNotFound(
      cfs_configuration_name_vec.join(", "),
    ));
  }

  Ok((
    cfs_session_to_delete_vec,
    bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec,
    image_id_vec,
    cfs_configuration_name_vec,
    cfs_session_cfs_configuration_image_id_tuple_filtered_vec,
    cfs_configuration_vec,
  ))
}

/// Deletes CFS configuration, CFS session, BOS sessiontemplate, BOS session and images related to
/// a CFS configuration. This method is safe. It checks if CFS configuration to delete is assigned
/// to a CFS component as a 'desired configuration' and also checks if image related to CFS
/// configuration is used as a boot image of any node in the system.
pub async fn delete(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cfs_configuration_name_vec: &[String],
  image_id_vec: &[String],
  cfs_session_name_vec: &[String],
  bos_sessiontemplate_name_vec: &[String],
) -> Result<(), Error> {
  // DELETE DATA
  //
  // DELETE IMAGES
  for image_id in image_id_vec {
    log::info!("Deleting IMS image '{}'", image_id);
    let image_deleted_value_rslt = ims::image::http_client::delete(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &image_id,
    )
    .await;

    // process api response
    match image_deleted_value_rslt {
      Ok(_) => println!("IMS image deleted: {}", image_id),
      Err(e) => {
        eprintln!("{e}. Continue");
      }
    }
  }

  // DELETE BOS SESSIONS
  let bos_session_vec = bos::session::http_client::v2::get(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    None,
  )
  .await?;

  // Match BOS SESSIONS with the BOS SESSIONTEMPLATE RELATED
  for bos_session in bos_session_vec {
    let bos_session_id = &bos_session.name.unwrap();
    log::info!("Deleting BOS sesion '{}'", bos_session_id);

    if bos_sessiontemplate_name_vec.contains(&bos_session.template_name) {
      bos::session::http_client::v2::delete(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &bos_session_id,
      )
      .await?;

      println!(
        "BOS session deleted: {}",
        bos_session_id // For some reason CSM API to delete a BOS
                       // session does not returns the BOS session
                       // ID in the payload...
      );
    } else {
      log::debug!("Ignoring BOS session template {}", bos_session_id);
    }
  }

  // DELETE CFS SESSIONS
  let max_attempts = 5;
  for cfs_session_name in cfs_session_name_vec {
    log::info!("Deleting IMS image '{}'", cfs_session_name);
    let mut counter = 0;
    loop {
      let deletion_rslt = cfs::session::http_client::v3::delete(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &cfs_session_name,
      )
      .await;

      if deletion_rslt.is_err() && counter <= max_attempts {
        log::warn!("Could not delete CFS session {} attempt {} of {}, trying again in 2 seconds...", cfs_session_name, counter, max_attempts);
        tokio::time::sleep(time::Duration::from_secs(2)).await;
        counter += 1;
      } else if deletion_rslt.is_err() && counter > max_attempts {
        eprintln!(
          "ERROR deleting CFS session {}, please delete it manually.",
          cfs_session_name,
        );
        log::debug!("ERROR:\n{:#?}", deletion_rslt.unwrap_err());
        break;
      } else {
        println!("CfS session deleted: {}", cfs_session_name);
        break;
      }
    }
  }

  // DELETE BOS SESSIONTEMPLATES
  let max_attempts = 5;
  for bos_sessiontemplate_name in bos_sessiontemplate_name_vec {
    log::info!(
      "Deleting BOS sessiontemplate '{}'",
      bos_sessiontemplate_name
    );
    let mut counter = 0;
    loop {
      let deletion_rslt = bos::template::http_client::v2::delete(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &bos_sessiontemplate_name,
      )
      .await;

      if deletion_rslt.is_err() && counter <= max_attempts {
        log::warn!("Could not delete BOS sessiontemplate {} attempt {} of {}, trying again in 2 seconds...", bos_sessiontemplate_name, counter, max_attempts);
        tokio::time::sleep(time::Duration::from_secs(2)).await;
        counter += 1;
      } else if deletion_rslt.is_err() && counter > max_attempts {
        eprintln!(
          "ERROR deleting BOS sessiontemplate {}, please delete it manually.",
          bos_sessiontemplate_name,
        );
        log::debug!("ERROR:\n{:#?}", deletion_rslt.unwrap_err());
        break;
      } else {
        println!("BOS sessiontemplate deleted: {}", bos_sessiontemplate_name);
        break;
      }
    }
  }

  // DELETE CFS CONFIGURATIONS
  let max_attempts = 5;
  for cfs_configuration in cfs_configuration_name_vec {
    log::info!("Deleting CFS configuration '{}'", cfs_configuration);
    let mut counter = 0;
    loop {
      let deletion_rslt = cfs::configuration::http_client::v3::delete(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        cfs_configuration,
      )
      .await;

      if deletion_rslt.is_err() && counter <= max_attempts {
        log::warn!("Could not delete CFS configuration {} attempt {} of {}, trying again in 2 seconds...", cfs_configuration, counter, max_attempts);
        tokio::time::sleep(time::Duration::from_secs(2)).await;
        counter += 1;
      } else if deletion_rslt.is_err() && counter > max_attempts {
        eprintln!(
          "ERROR deleting CFS configuration {}, please delete it manually.",
          cfs_configuration,
        );
        log::debug!("ERROR:\n{:#?}", deletion_rslt.unwrap_err());
        break;
      } else {
        println!("CFS configuration deleted: {}", cfs_configuration);
        break;
      }
    }
  }

  Ok(())
}

/// Given a list of boot params, this function returns the list of hosts booting an image_id
pub fn get_node_vec_booting_image(
  image_id: &str,
  boot_param_vec: &[BootParameters],
) -> Vec<String> {
  let mut node_booting_image_vec = boot_param_vec
    .iter()
    .cloned()
    .filter(|boot_param| boot_param.get_boot_image().eq(image_id))
    .flat_map(|boot_param| boot_param.hosts)
    .collect::<Vec<_>>();

  node_booting_image_vec.sort();

  node_booting_image_vec
}
