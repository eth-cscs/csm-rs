use serde_json::Value;

use crate::error::Error;

use super::{
  types::{Job, SshContainer},
  utils::wait_ims_job_to_finish,
};

/// Get IMS job ref --> https://csm12-apidocs.svc.cscs.ch/paas/ims/operation/post_v3_job/
/// Creates an IMS job of type 'customize'. Used to create 'ephemeral-environments'
pub async fn post_customize(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  image_root_archive_name: &str,
  artifact_id: &str,
  public_key_id: &str,
) -> Result<Value, Error> {
  let ssh_container = SshContainer {
    name: "jail".to_string(),
    jail: true,
  };

  let ssh_container_list = vec![ssh_container];

  let ims_job = Job {
    job_type: "customize".to_string(),
    image_root_archive_name: image_root_archive_name.to_string(),
    kernel_file_name: Some("kernel".to_string()),
    initrd_file_name: Some("initrd".to_string()),
    kernel_parameters_file_name: None,
    artifact_id: artifact_id.to_string(),
    public_key_id: public_key_id.to_string(),
    ssh_containers: Some(ssh_container_list),
    enable_debug: Some(false),
    build_env_size: None,
    require_dkms: None, // FIXME: check if SAT file uses this value
    id: None,
    created: None,
    status: None,
    kubernetes_job: None,
    kubernetes_service: None,
    kubernetes_configmap: None,
    resultant_image_id: None,
    kubernetes_namespace: None,
    arch: None,
  };

  let client;

  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  // Build client
  if std::env::var("SOCKS5").is_ok() {
    // socks5 proxy
    log::debug!("SOCKS5 enabled");
    let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5")?)?;
    client = client_builder.proxy(socks5proxy).build()?;
  } else {
    client = client_builder.build()?;
  }

  let api_url = shasta_base_url.to_owned() + "/ims/v3/jobs";

  let response = client
    .post(api_url)
    .bearer_auth(shasta_token)
    .json(&ims_job)
    .send()
    .await
    .map_err(|error| Error::NetError(error))?;

  if response.status().is_success() {
    // Make sure we return a vec if user requesting a single value
    response
      .json()
      .await
      .map_err(|error| Error::NetError(error))
  } else {
    let payload = response
      .json::<Value>()
      .await
      .map_err(|error| Error::NetError(error))?;

    Err(Error::CsmError(payload))
  }
}

/// Creates an IMS job, this method is asynchronous, meaning, it will returns when the server
/// returns the job creation call
pub async fn post(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  ims_job: &Job,
) -> Result<Job, Error> {
  let client;

  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  // Build client
  if std::env::var("SOCKS5").is_ok() {
    // socks5 proxy
    log::debug!("SOCKS5 enabled");
    let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5")?)?;
    client = client_builder.proxy(socks5proxy).build()?;
  } else {
    client = client_builder.build()?;
  }

  let api_url = shasta_base_url.to_owned() + "/ims/v3/jobs";

  client
    .post(api_url)
    .bearer_auth(shasta_token)
    .json(&ims_job)
    .send()
    .await
    .map_err(Error::NetError)?
    .error_for_status()
    .map_err(Error::NetError)?
    .json()
    .await
    .map_err(Error::NetError)
}

/// Synchronous version of the post method, used if want to wait till the IMS job is finished
pub async fn post_sync(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  ims_job: &Job,
) -> Result<Job, Error> {
  log::info!("Create IMS job");
  log::debug!(
    "Create IMS job request payload:\n{}",
    serde_json::to_string_pretty(&ims_job)?
  );

  let ims_job: Job =
    post(shasta_token, shasta_base_url, shasta_root_cert, ims_job).await?;

  let ims_job_id = ims_job.id.unwrap();

  // Wait till the IMS job finishes
  wait_ims_job_to_finish(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    &ims_job_id,
  )
  .await?;

  // Get IMS job status
  get(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    Some(&ims_job_id),
  )
  .await?
  .first()
  .cloned()
  .ok_or_else(|| {
    Error::Message(format!("ERROR - IMS job '{}' not found", ims_job_id))
  })
}

/// Create IMS job ref --> https://csm12-apidocs.svc.cscs.ch/paas/ims/operation/post_v3_job/
pub async fn get(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  job_id_opt: Option<&str>,
) -> Result<Vec<Job>, Error> {
  let client;

  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  // Build client
  if std::env::var("SOCKS5").is_ok() {
    // socks5 proxy
    log::debug!("SOCKS5 enabled");
    let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5")?)?;
    client = client_builder.proxy(socks5proxy).build()?;
  } else {
    client = client_builder.build()?;
  }

  let api_url = if let Some(job_id) = job_id_opt {
    shasta_base_url.to_owned() + "/ims/v3/jobs/" + job_id
  } else {
    shasta_base_url.to_owned() + "/ims/v3/jobs"
  };

  let response = client
    .get(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?
    .error_for_status()
    .map_err(Error::NetError)?;

  if job_id_opt.is_some() {
    Ok(vec![response
      .json::<Job>()
      .await
      .map_err(Error::NetError)?])
  } else {
    response.json().await.map_err(Error::NetError)
  }
}
