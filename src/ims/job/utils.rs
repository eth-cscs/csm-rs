use std::io::{self, Write};

use crate::{ShastaClient, error::Error, ims::job::types::Job};

/// Wait for an IMS job to finish (polls every 2s, max 1800 attempts ~ 1h).
pub async fn wait_ims_job_to_finish(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  ims_job_id: &str,
) -> Result<(), Error> {
  let client = ShastaClient::new(
    shasta_base_url,
    shasta_token,
    shasta_root_cert.to_vec(),
    socks5_proxy.map(str::to_owned),
  )?;
  let mut i = 0;
  let max = 1800;
  loop {
    let ims_job: Job = client
      .ims_job_get(Some(ims_job_id))
      .await?
      .first()
      .cloned()
      .ok_or_else(|| {
        Error::Message(format!("ERROR - IMS job '{}' not found", ims_job_id))
      })?;

    log::debug!(
      "IMS job details:\n{}",
      serde_json::to_string_pretty(&ims_job).unwrap_or_default()
    );

    let ims_job_status = ims_job.status.unwrap_or_default();

    if (ims_job_status != "error" && ims_job_status != "success") && i < max {
      log::info!("\x1B[2K"); // Clear current line
      io::stdout().flush().unwrap_or(());
      log::info!(
        "\rWaiting IMS job '{}' with job status '{}'. Checking again in 2 secs. Attempt {} of {}.",
        ims_job_id, ims_job_status, i, max
      );
      io::stdout().flush().unwrap_or(());

      tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

      i += 1;
    } else {
      log::info!(
        "\nIMS job '{}' finished with job status '{}'",
        ims_job_id, ims_job_status
      );
      break;
    }
  }

  Ok(())
}
