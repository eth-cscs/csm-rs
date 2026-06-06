//! Cancel an in-flight CFS session and clean up the resources derived
//! from it.
//!
//! Moved here from `commands::delete_and_cancel_session::command` in
//! the architecture re-audit so the dispatcher
//! ([`crate::backend_connector::cfs`]) can call the domain helper
//! directly instead of reaching across into the `commands` layer.

use serde_json::Value;

use crate::{
  bss::types::BootParameters,
  cfs::{
    session::utils::get_list_xnames_related_to_session,
    v2::{CfsSessionGetResponse, Component},
  },
  error::Error,
  hsm::group::types::Group,
};

/// Cancel an in-flight CFS session and clean up the resources derived
/// from it.
///
/// Resolves the xnames touched by `cfs_session` (via HSM-group
/// membership or explicit xname targets), then deletes the session
/// itself together with the CFS components, BSS boot parameters, and
/// any related BOS artefacts.
///
/// # Arguments
///
/// - `group_available_vec` — HSM groups the caller is allowed to
///   target; sessions that reach outside this set are refused.
/// - `cfs_component_vec` / `bos_bootparameters_vec` — current snapshots
///   used to decide what needs cleaning up.
/// - `dry_run` — when `true`, log the intended deletions without
///   mutating CSM.
///
/// # Errors
///
/// Returns an [`Error`] variant on CSM, transport, or
/// deserialization failure; see the crate-level `Error` enum
/// for the full set.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  client: &crate::ShastaClient,
  shasta_token: &str,
  group_available_vec: Vec<Group>,
  cfs_session: &CfsSessionGetResponse,
  cfs_component_vec: &[Component],
  bos_bootparameters_vec: &[BootParameters],
  dry_run: bool,
) -> Result<(), Error> {
  let cfs_session_name = &cfs_session.name;

  log::debug!("Deleting session '{}'", cfs_session_name);

  // Get xnames related to CFS session to delete:
  // - xnames belonging to HSM group related to CFS session
  // - xnames in CFS session
  let xname_vec = get_list_xnames_related_to_session(
    group_available_vec,
    cfs_session.clone(),
  )
  .await?;

  let cfs_session_target_definition =
    cfs_session.get_target_def().ok_or_else(|| {
      Error::Message(
        "CFS session has no target definition (image/dynamic)".to_string(),
      )
    })?;

  // DELETE DATA
  //
  // * if session is of type dynamic (runtime session) then:
  // Get retry_policy
  if cfs_session_target_definition == "dynamic" {
    // The CFS session is of type 'target dynamic' (runtime CFS batcher) - cancel session by
    // setting error_count to retry_policy value
    log::info!("CFS session target definition is 'dynamic'.");

    let cfs_global_options =
      client.cfs_component_v3_get_options(shasta_token).await?;

    let retry_policy = cfs_global_options
      .get("default_batcher_retry_policy")
      .and_then(Value::as_u64)
      .ok_or_else(|| {
        Error::Message(
          "CFS options response missing 'default_batcher_retry_policy'"
            .to_string(),
        )
      })?;

    cancel_session(
      client,
      shasta_token,
      xname_vec,
      Some(cfs_component_vec.to_vec()),
      retry_policy,
      dry_run,
    )
    .await?;
  } else if cfs_session_target_definition == "image" {
    // The CFS session is not of type 'target dynamic' (runtime CFS batcher)
    let image_created_by_cfs_session_vec: Vec<&str> =
      cfs_session.results_id().collect();
    if !image_created_by_cfs_session_vec.is_empty() {
      // Delete images
      delete_images(
        client,
        shasta_token,
        &image_created_by_cfs_session_vec,
        bos_bootparameters_vec,
        dry_run,
      )
      .await?;
    }
  } else {
    return Err(Error::Message(format!(
      "CFS session target definition is '{}'. Don't know how to continue. Exit",
      cfs_session_target_definition
    )));
  };

  // Delete CFS session
  if dry_run {
    log::info!("Dry Run Mode: Delete CFS session '{}'", cfs_session_name);
  } else {
    client
      .cfs_session_v3_delete(shasta_token, cfs_session_name)
      .await?;
  }

  Ok(())
}

async fn delete_images(
  client: &crate::ShastaClient,
  shasta_token: &str,
  image_created_by_cfs_session_vec: &[&str],
  bss_bootparameters_vec_opt: &[BootParameters],
  dry_run: bool,
) -> Result<(), Error> {
  // Delete images
  for image_id in image_created_by_cfs_session_vec {
    let is_image_boot_node = bss_bootparameters_vec_opt
      .iter()
      .any(|boot_parameters| boot_parameters.get_boot_image().eq(image_id));

    if !is_image_boot_node {
      if dry_run {
        log::info!(
          "Dry Run Mode: CFS session target definition is 'image'. Deleting image '{}'",
          image_id
        );
      } else {
        client.ims_image_delete(shasta_token, image_id).await?;
      }
    } else {
      log::info!(
        "Image '{}' is a boot node image. It will not be deleted.",
        image_id
      );
    }
  }

  Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn cancel_session(
  client: &crate::ShastaClient,
  shasta_token: &str,
  xname_vec: Vec<String>,
  cfs_component_vec_opt: Option<Vec<Component>>,
  retry_policy: u64,
  dry_run: bool,
) -> Result<(), Error> {
  // Set CFS components error_count == retry_policy so CFS batcher stops retrying running
  log::info!(
    "Set 'error_count' {} to xnames {:?}",
    retry_policy,
    xname_vec
  );

  // Update CFS component error_count
  let cfs_component_vec: Vec<Component> = cfs_component_vec_opt
    .map(|cfs_component_vec| {
      cfs_component_vec
        .iter()
        .filter(|cfs_component| {
          cfs_component
            .id
            .as_ref()
            .is_some_and(|id| xname_vec.contains(id))
        })
        .cloned()
        .collect()
    })
    .ok_or_else(|| Error::Message("No CFS components".to_string()))?;

  log::info!(
    "Update error count on nodes {:?} to {}",
    xname_vec,
    retry_policy
  );

  if dry_run {
    log::info!(
      "Dry Run Mode: Update error count on nodes {:?}",
      cfs_component_vec
    );
  } else {
    client
      .cfs_component_v2_put_component_list(shasta_token, cfs_component_vec)
      .await?;
  }

  Ok(())
}
