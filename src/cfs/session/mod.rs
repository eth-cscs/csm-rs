pub mod http_client;
pub mod utils;

use crate::cfs;
use http_client::v2::types::{CfsSessionGetResponse, CfsSessionPostRequest};

use crate::{
  common::{
    kubernetes::{self, i_print_cfs_session_logs},
    vault::http_client::fetch_shasta_k8s_secrets_from_vault,
  },
  error::Error,
};

/// Fetch CFS sessions ref --> https://apidocs.svc.cscs.ch/paas/cfs/operation/get_sessions/
/// Returns list of CFS sessions ordered by start time.
/// This methods filter by either HSM group name or HSM group members or both
pub async fn get_and_sort(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  min_age_opt: Option<&String>,
  max_age_opt: Option<&String>,
  status_opt: Option<&String>,
  session_name_opt: Option<&String>,
  is_succeded_opt: Option<bool>,
) -> Result<Vec<CfsSessionGetResponse>, Error> {
  let mut cfs_session_vec = cfs::session::http_client::v2::get(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    min_age_opt,
    max_age_opt,
    status_opt,
    session_name_opt,
    is_succeded_opt,
  )
  .await?;

  // Sort CFS sessions by start time order ASC
  cfs_session_vec.sort_by(|a, b| {
    a.status
      .as_ref()
      .unwrap()
      .session
      .as_ref()
      .unwrap()
      .start_time
      .as_ref()
      .unwrap()
      .cmp(
        b.status
          .as_ref()
          .unwrap()
          .session
          .as_ref()
          .unwrap()
          .start_time
          .as_ref()
          .unwrap(),
      )
  });

  Ok(cfs_session_vec)
}

pub async fn post(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  session: &CfsSessionPostRequest,
) -> Result<CfsSessionGetResponse, Error> {
  log::info!("Create CFS session '{}'", session.name);
  log::debug!("Create CFS session request payload:\n{:#?}", session);

  cfs::session::http_client::v2::post(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    session,
  )
  .await
}

/// Creates a CFS session and waits for it to finish.
/// Optionally, it can also print the CFS session logs if `watch_logs` is set to true.
#[deprecated(
  since = "v0.42.3-beta.71",
  note = "this function prints CFS logs to stdout"
)]
pub async fn i_post_sync(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  vault_base_url: &str,
  site_name: &str,
  k8s_api_url: &str,
  session: &CfsSessionPostRequest,
  watch_logs: bool,
  timestamps: bool,
) -> Result<CfsSessionGetResponse, Error> {
  // Create CFS session
  log::info!("Create CFS session '{}'", session.name);
  let cfs_session: CfsSessionGetResponse = cfs::session::post(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    session,
  )
  .await?;

  let cfs_session_name: String = cfs_session.name.unwrap();

  // FIXME: refactor becase this code is duplicated in command `manta apply sat-file` and also in
  // `manta logs`
  if watch_logs {
    log::info!("Fetching logs form CFS session {} ...", session.name);
    let shasta_k8s_secrets = fetch_shasta_k8s_secrets_from_vault(
      vault_base_url,
      shasta_token,
      site_name,
    )
    .await?;

    let client = kubernetes::get_client(k8s_api_url, shasta_k8s_secrets)
      .await
      .unwrap();

    let _ =
      i_print_cfs_session_logs(client, &cfs_session_name, timestamps).await;
  }

  // User does not want the CFS logs but we still need to wayt the CFS session to
  // finis. Wait till the CFS session finishes
  utils::wait_cfs_session_to_finish(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    &cfs_session_name,
  )
  .await?;

  // Get most recent CFS session status
  let cfs_session: CfsSessionGetResponse = get_and_sort(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    None,
    None,
    None,
    Some(&cfs_session_name),
    None,
  )
  .await?
  .first()
  .unwrap()
  .clone();

  Ok(cfs_session)
}
