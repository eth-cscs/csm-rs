use core::time;
use std::collections::BTreeMap;

use futures::{AsyncBufRead, AsyncBufReadExt, StreamExt, TryStreamExt};

use k8s_openapi::api::core::v1::{ConfigMap, Container, Pod};
use kube::api::DeleteParams;
use kube::{
  api::{AttachParams, AttachedProcess},
  config::{
    AuthInfo, Cluster, Context, KubeConfigOptions, Kubeconfig, NamedAuthInfo,
    NamedCluster, NamedContext,
  },
  Api,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use termion::color;

use crate::common::vault::http_client::fetch_shasta_k8s_secrets_from_vault;
use crate::error::Error;
use http::Uri;
use kube::runtime::reflector;
use secrecy::SecretBox;

#[derive(Serialize, Deserialize, Debug)]
pub enum K8sAuth {
  Native {
    certificate_authority_data: String,
    client_certificate_data: String,
    client_key_data: String,
  },
  Vault {
    base_url: String,
    secret_path: String,
    role_id: String,
  },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct K8sDetails {
  pub api_url: String,
  pub authentication: K8sAuth,
}

pub async fn get_client(
  k8s_api_url: &str,
  shasta_k8s_secrets: Value,
) -> Result<kube::Client, Error> {
  let certificate_authority_data = shasta_k8s_secrets
    ["certificate-authority-data"]
    .as_str()
    .unwrap();
  let client_certificate_data = shasta_k8s_secrets["client-certificate-data"]
    .as_str()
    .unwrap();
  let client_key_data = shasta_k8s_secrets["client-key-data"]
    .as_str()
    .map(|s| s.to_string())
    .unwrap();

  let shasta_cluster = Cluster {
    server: Some(k8s_api_url.to_string()),
    tls_server_name: Some("kube-apiserver".to_string()), // The value "kube-apiserver" has been taken from the
    // Subject: CN value in the Shasta certificate running
    // this command echo | openssl s_client -showcerts -servername 10.252.1.12 -connect 10.252.1.12:6442 2>/dev/null | openssl x509 -inform pem -noout -text
    insecure_skip_tls_verify: Some(true),
    certificate_authority: None,
    certificate_authority_data: Some(String::from(certificate_authority_data)),
    proxy_url: None,
    extensions: None,
    disable_compression: None,
  };

  let shasta_named_cluster = NamedCluster {
    name: String::from("shasta"),
    cluster: Some(shasta_cluster),
  };

  /* let socks5_proxy_url = if let Ok(socks5_address) = std::env::var("SOCKS5") {
    log::debug!("SOCKS5 enabled");
    socks5_address
      .parse::<Uri>()
      .map_err(|_| Error::Message("Could not parse socks5_proxy".to_string()))?
  }

  shasta_named_cluster */

  let shasta_auth_info = AuthInfo {
    username: None,
    password: None,
    token: None,
    token_file: None,
    client_certificate: None,
    client_certificate_data: Some(String::from(client_certificate_data)),
    client_key: None,
    client_key_data: Some(SecretBox::try_from(client_key_data).unwrap()),
    impersonate: None,
    impersonate_groups: None,
    auth_provider: None,
    exec: None,
  };

  let shasta_named_auth_info = NamedAuthInfo {
    name: String::from("kubernetes-admin"),
    auth_info: Some(shasta_auth_info),
  };

  let shasta_context = Context {
    cluster: String::from("shasta"),
    user: Some(String::from("kubernetes-admin")),
    namespace: None,
    extensions: None,
  };

  let shasta_named_context = NamedContext {
    name: String::from("kubernetes-admin@kubernetes"),
    context: Some(shasta_context),
  };

  let kube_config = Kubeconfig {
    preferences: None,
    clusters: vec![shasta_named_cluster],
    auth_infos: vec![shasta_named_auth_info],
    contexts: vec![shasta_named_context],
    current_context: Some(String::from("kubernetes-admin@kubernetes")),
    extensions: None,
    kind: None,
    api_version: None,
  };

  let kube_config_options = KubeConfigOptions {
    context: Some(String::from("kubernetes-admin@kubernetes")),
    cluster: Some(String::from("shasta")),
    user: Some(String::from("kubernetes-admin")),
  };

  let mut config =
    kube::Config::from_custom_kubeconfig(kube_config, &kube_config_options)
      .await
      .map_err(|e| Error::K8sError(e.to_string()))?;

  let client = if let Ok(socks5_address) = std::env::var("SOCKS5") {
    log::info!("K8s SOCKS5 enabled");
    let socks5_proxy_uri = socks5_address.parse::<Uri>().map_err(|_| {
      Error::Message("Could not parse socks5_proxy".to_string())
    })?;

    config.proxy_url = Some(socks5_proxy_uri);

    kube::Client::try_from(config)
      .map_err(|e| Error::K8sError(e.to_string()))?
  } else {
    kube::Client::try_from(config)
      .map_err(|e| Error::K8sError(e.to_string()))?
  };

  Ok(client)
}

#[deprecated(
  since = "v0.42.3-beta.71",
  note = "this function prints CFS logs to stdout"
)]
pub async fn i_print_cfs_session_logs(
  client: kube::Client,
  cfs_session_name: &str,
  timestamps: bool,
) -> Result<(), Error> {
  let logs_stream = get_cfs_session_init_container_git_clone_logs_stream(
    client.clone(),
    cfs_session_name,
    timestamps,
  )
  .await?;

  let mut lines = logs_stream.lines();

  while let Some(line) = lines.try_next().await.unwrap() {
    println!("{}", line);
  }

  // let _ = print_cfs_session_container_ansible_logs_stream(client, cfs_session_name).await;

  let mut logs_stream = get_cfs_session_container_inventory_logs_stream(
    client.clone(),
    cfs_session_name,
    timestamps,
  )
  .await?
  .lines();

  while let Some(line) = logs_stream.try_next().await.unwrap() {
    println!("{}", line);
  }

  let mut logs_stream = get_cfs_session_container_ansible_logs_stream(
    client.clone(),
    cfs_session_name,
    timestamps,
  )
  .await?
  .lines();

  while let Some(line) = logs_stream.try_next().await.unwrap() {
    println!("{}", line);
  }

  Ok(())
}

pub async fn try_get_configmap(
  client: kube::Client,
  configmap_name: &str,
) -> Result<BTreeMap<String, String>, Error> {
  let configmap_api: kube::Api<ConfigMap> =
    kube::Api::namespaced(client, "services");

  let params = kube::api::ListParams::default()
    .fields(&("metadata.name=".to_owned() + configmap_name));

  let configmap = configmap_api
    .list(&params)
    .await
    .map_err(|e| Error::K8sError(e.to_string()))?;

  let configmap_data = configmap
    .items
    .first()
    .ok_or_else(|| {
      Error::K8sError("ERROR - There is no configmap".to_string())
    })?
    .clone();

  configmap_data.data.ok_or_else(|| {
    Error::K8sError("ERROR - There is no data in the configmap".to_string())
  })
}

pub async fn get_cfs_session_init_container_git_clone_logs_stream(
  client: kube::Client,
  cfs_session_name: &str,
  timestamps: bool,
) -> Result<impl AsyncBufRead, Error> {
  get_init_container_logs_stream(
    client,
    cfs_session_name,
    "git-clone",
    "services",
    format!("cfsession={}", cfs_session_name).as_str(),
    timestamps,
  )
  .await
}

pub async fn get_cfs_session_container_inventory_logs_stream(
  client: kube::Client,
  cfs_session_name: &str,
  timestamps: bool,
) -> Result<impl AsyncBufRead, Error> {
  get_container_logs_stream(
    client,
    cfs_session_name,
    "inventory",
    "services",
    format!("cfsession={}", cfs_session_name).as_str(),
    timestamps,
  )
  .await
}

pub async fn get_cfs_session_container_ansible_logs_stream(
  client: kube::Client,
  cfs_session_name: &str,
  timestamps: bool,
) -> Result<impl AsyncBufRead, Error> {
  get_container_logs_stream(
    client,
    cfs_session_name,
    "ansible",
    "services",
    format!("cfsession={}", cfs_session_name).as_str(),
    timestamps,
  )
  .await
}

pub async fn get_init_container_logs_stream(
  client: kube::Client,
  cfs_session_name: &str,
  init_container_name: &str,
  namespace: &str,
  label_selector: &str,
  timestamps: bool,
) -> Result<impl AsyncBufRead, Error> {
  // Get logs for 'git-clone' init container

  let pods_api: kube::Api<Pod> = kube::Api::namespaced(client, namespace);

  let params = kube::api::ListParams::default()
    .limit(1)
    .labels(label_selector);

  let mut pods = pods_api
    .list(&params)
    .await
    .map_err(|e| Error::K8sError(format!("{e}")))?;

  log::debug!(
    "Pods related to CFS session '{}' found are:\n'{:#?}'",
    cfs_session_name,
    pods,
  );

  let mut i = 0;
  let max = 4;
  let delay_secs = 2;

  // Waiting for pod to start
  while pods.items.is_empty() && i <= max {
    println!(
            "Pod for cfs session '{}' missing (probably being created). Trying again in {} secs. Attempt {} of {}",
            cfs_session_name,
            delay_secs,
            i + 1,
            max
        );
    i += 1;
    tokio::time::sleep(time::Duration::from_secs(delay_secs)).await;
    pods = pods_api
      .list(&params)
      .await
      .map_err(|e| Error::K8sError(format!("{e}")))?;
  }

  if pods.items.is_empty() {
    return Err(
      Error::K8sError(format!(
        "Pod for cfs session {} missing. Aborting operation",
        cfs_session_name
      ))
      .into(),
    );
  }

  let cfs_session_pod = &pods.items[0].clone();

  let cfs_session_pod_name = cfs_session_pod.metadata.name.clone().unwrap();
  log::info!("Pod name: {}", cfs_session_pod_name);

  let mut init_container_vec = cfs_session_pod
    .spec
    .as_ref()
    .unwrap()
    .init_containers
    .clone()
    .unwrap();

  log::debug!(
    "Init containers found in pod {}: {:#?}",
    cfs_session_pod_name,
    init_container_vec
  );

  let mut git_clone_container: &Container = init_container_vec
    .iter()
    .find(|container| container.name.eq(init_container_name))
    .unwrap();

  log::info!(
    "Fetching logs for init container {} in namespace/pod {}/{}",
    init_container_name,
    cfs_session_pod.clone().metadata.namespace.unwrap(),
    cfs_session_pod.clone().metadata.name.unwrap(),
  );

  let mut init_container_status = cfs_session_pod
    .status
    .clone()
    .unwrap()
    .init_container_statuses
    .unwrap()
    .into_iter()
    .find(|init_container| init_container.name.eq(&git_clone_container.name));

  let mut i = 0;
  let max = 60;

  // Waiting for init container to start
  while (init_container_status.is_none()
    || init_container_status
      .clone()
      .unwrap()
      .state
      .unwrap()
      .waiting
      .is_some())
    && i <= max
  {
    log::debug!(
      "Init container '{}' state:\n{:?}",
      git_clone_container.name,
      init_container_status
    );
    println!(
      "Waiting for container '{}' to be ready. Checking again in 2 secs. Attempt {} of {}",
      git_clone_container.name,
      i + 1,
      max
    );

    i += 1;
    tokio::time::sleep(time::Duration::from_secs(2)).await;

    let cfs_session_pod = pods_api
      .list(&params)
      .await
      .map_err(|e| Error::K8sError(format!("{e}")))?
      .items[0]
      .clone();

    init_container_vec = cfs_session_pod
      .spec
      .as_ref()
      .unwrap()
      .init_containers
      .clone()
      .unwrap();

    git_clone_container = init_container_vec
      .iter()
      .find(|container| container.name.eq("git-clone"))
      .unwrap();

    init_container_status = cfs_session_pod
      .status
      .clone()
      .unwrap()
      .init_container_statuses
      .unwrap()
      .into_iter()
      .find(|init_container| init_container.name.eq(&git_clone_container.name));
  }

  if init_container_status.is_none()
    || init_container_status
      .unwrap()
      .state
      .unwrap()
      .waiting
      .is_some()
  {
    return Err(
      Error::K8sError(format!(
        "Container '{}' not ready. Aborting operation",
        init_container_name
      ))
      .into(),
    );
  }

  // get_container_logs_stream(git_clone_container, cfs_session_pod, &pods_api).await
  log::info!("Looking for container '{}'", init_container_name);

  pods_api
    .log_stream(
      cfs_session_pod.metadata.name.as_ref().unwrap(),
      &kube::api::LogParams {
        follow: true,
        container: Some(init_container_name.to_string()),
        // ..kube::api::LogParams::default(),
        limit_bytes: None,
        pretty: true,
        previous: false,
        since_seconds: None,
        since_time: None,
        tail_lines: None,
        timestamps,
      },
    )
    .await
    .map_err(|e| Error::K8sError(format!("{e}")))
}

pub async fn get_container_logs_stream(
  client: kube::Client,
  cfs_session_name: &str,
  container_name: &str,
  namespace: &str,
  label_selector: &str,
  timestamps: bool,
) -> Result<impl AsyncBufRead, Error> {
  let pods_api: kube::Api<Pod> = kube::Api::namespaced(client, namespace);

  let params = kube::api::ListParams::default()
    .limit(1)
    // .labels(format!("cfsession={}", cfs_session_name).as_str());
    .labels(label_selector);

  let mut pods = pods_api
    .list(&params)
    .await
    .map_err(|e| Error::K8sError(format!("{e}")))?;

  let mut i = 0;
  let max = 30;
  let delay_secs = 2;

  // Waiting for pod to start
  while pods.items.is_empty() && i <= max {
    println!(
            "Waiting k8s to create pod/container for cfs session '{}'. Trying again in {} secs. Attempt {} of {}",
            cfs_session_name,
            delay_secs,
            i + 1,
            max
        );
    i += 1;
    tokio::time::sleep(time::Duration::from_secs(delay_secs)).await;
    pods = pods_api
      .list(&params)
      .await
      .map_err(|e| Error::K8sError(format!("{e}")))?;
  }

  if pods.items.is_empty() {
    return Err(
      Error::K8sError(format!(
        "Pod for cfs session '{}' not created. Aborting operation.",
        cfs_session_name
      ))
      .into(),
    );
  }

  let cfs_session_pod = &pods.items[0].clone();

  let cfs_session_pod_name = cfs_session_pod.metadata.name.clone().unwrap();
  log::info!("Pod name: {}", cfs_session_pod_name);

  let mut containers = cfs_session_pod.spec.as_ref().unwrap().containers.iter();

  log::debug!(
    "Containers found in pod {}: {:#?}",
    cfs_session_pod_name,
    containers
  );

  let ansible_container: &Container = containers
    .find(|container| container.name.eq(container_name))
    .unwrap();

  let mut container_status =
    get_container_status(cfs_session_pod, &ansible_container.name);

  let mut i = 0;
  let max = 300;

  // Waiting for container ansible-x to start
  while container_status.as_ref().is_none()
    || container_status.as_ref().unwrap().waiting.is_some() && i <= max
  {
    println!(
            "Container ({}) status missing or 'waiting'. Checking again in 2 secs. Attempt {} of {}",
            ansible_container.name,
            i + 1,
            max
        );
    i += 1;
    tokio::time::sleep(time::Duration::from_secs(2)).await;
    let pods = pods_api
      .list(&params)
      .await
      .map_err(|e| Error::K8sError(format!("{e}")))?;
    container_status =
      get_container_status(&pods.items[0], &ansible_container.name);
    log::debug!(
      "Container status:\n{:#?}",
      container_status.as_ref().unwrap()
    );
  }

  if container_status.as_ref().unwrap().waiting.is_some() {
    return Err(
      Error::K8sError(format!(
        "Container ({}) status is waiting. Aborting operation.",
        ansible_container.name
      ))
      .into(),
    );
  }

  if container_status.as_ref().unwrap().terminated.is_some()
    || container_status.as_ref().unwrap().running.is_some()
  {
    log::info!("Looking for container '{}'", container_name);

    pods_api
      .log_stream(
        cfs_session_pod.metadata.name.as_ref().unwrap(),
        &kube::api::LogParams {
          follow: true,
          container: Some(container_name.to_string()),
          limit_bytes: None,
          pretty: true,
          previous: false,
          since_seconds: None,
          since_time: None,
          tail_lines: None,
          timestamps,
          // ..kube::api::LogParams::default()
        },
      )
      .await
      .map_err(|e| Error::K8sError(format!("{e}")))
  } else {
    return Err(Error::Message(format!(
            "Container ({}) status is not running nor terminated. Aborting operation.\nContainer status:\n{:#?}",
            ansible_container.name, ansible_container
        )));
  }
}

pub async fn print_cfs_session_container_ansible_logs_stream(
  client: kube::Client,
  cfs_session_name: &str,
) -> Result<(), Error> {
  let container_name = "ansible";

  let pods_api: kube::Api<Pod> = kube::Api::namespaced(client, "services");

  let params = kube::api::ListParams::default()
    .limit(1)
    .labels(format!("cfsession={}", cfs_session_name).as_str());

  let mut pods = pods_api
    .list(&params)
    .await
    .map_err(|e| Error::K8sError(format!("{e}")))?;

  let mut i = 0;
  let max = 30;
  let delay_secs = 2;

  // Waiting for pod to start
  while pods.items.is_empty() && i <= max {
    println!(
            "Waiting k8s to create pod/container for cfs session '{}'. Trying again in {} secs. Attempt {} of {}",
            cfs_session_name,
            delay_secs,
            i + 1,
            max
        );
    i += 1;
    tokio::time::sleep(time::Duration::from_secs(delay_secs)).await;
    pods = pods_api
      .list(&params)
      .await
      .map_err(|e| Error::K8sError(format!("{e}")))?;
  }

  if pods.items.is_empty() {
    return Err(
      Error::K8sError(format!(
        "Pod for cfs session '{}' not created. Aborting operation.",
        cfs_session_name
      ))
      .into(),
    );
  }

  let cfs_session_pod = &pods.items[0].clone();

  let cfs_session_pod_name = cfs_session_pod.metadata.name.clone().unwrap();
  log::info!("Pod name: {}", cfs_session_pod_name);

  let mut containers = cfs_session_pod.spec.as_ref().unwrap().containers.iter();

  log::debug!(
    "Containers found in pod {}: {:#?}",
    cfs_session_pod_name,
    containers
  );

  let ansible_container: &Container = containers
    .find(|container| container.name.eq(container_name))
    .unwrap();

  let mut container_status =
    get_container_status(cfs_session_pod, &ansible_container.name);

  let mut i = 0;
  let max = 300;

  // Waiting for container ansible-x to start
  while container_status.as_ref().is_none()
    || container_status.as_ref().unwrap().waiting.is_some() && i <= max
  {
    println!(
            "Container ({}) status missing or 'waiting'. Checking again in 2 secs. Attempt {} of {}",
            ansible_container.name,
            i + 1,
            max
        );
    i += 1;
    tokio::time::sleep(time::Duration::from_secs(2)).await;
    let pods = pods_api
      .list(&params)
      .await
      .map_err(|e| Error::K8sError(format!("{e}")))?;
    container_status =
      get_container_status(&pods.items[0], &ansible_container.name);
    log::debug!(
      "Container status:\n{:#?}",
      container_status.as_ref().unwrap()
    );
  }

  if container_status.as_ref().unwrap().waiting.is_some() {
    return Err(
      Error::K8sError(format!(
        "Container ({}) status is waiting. Aborting operation.",
        ansible_container.name
      ))
      .into(),
    );
  }

  let mut attempt = 0;
  let max_attempts = 3;

  if container_status.as_ref().unwrap().terminated.is_some() {
    // Print CFS session logs already terminated on screen
    /* let logs_stream_rslt =
    get_container_logs_stream(ansible_container, cfs_session_pod, &pods_api).await; */
    log::info!("Looking for container '{}'", container_name);

    let logs_stream_rslt = pods_api
      .log_stream(
        cfs_session_pod.metadata.name.as_ref().unwrap(),
        &kube::api::LogParams {
          follow: true,
          container: Some(container_name.to_string()),
          ..kube::api::LogParams::default()
        },
      )
      .await
      .map_err(|e| Error::K8sError(format!("{e}")))
      .map(|stream| stream.lines());

    if let Ok(mut logs_stream) = logs_stream_rslt {
      while let Some(line) = logs_stream.try_next().await? {
        println!("{}", line);
      }
    }
  } else {
    // Print current CFS session logs on screen
    while container_status.as_ref().unwrap().running.is_some()
      && attempt < max_attempts
    {
      /* let logs_stream_rslt =
      get_container_logs_stream(ansible_container, cfs_session_pod, &pods_api).await; */
      log::info!("Looking for container '{}'", container_name);

      println!(
        "\n{}####{} Container {}'{}'{} logs\n",
        color::Fg(color::Green),
        color::Fg(color::Reset),
        color::Fg(color::Blue),
        container_name,
        color::Fg(color::Reset),
      );

      let logs_stream_rslt = pods_api
        .log_stream(
          cfs_session_pod.metadata.name.as_ref().unwrap(),
          &kube::api::LogParams {
            follow: true,
            container: Some(container_name.to_string()),
            ..kube::api::LogParams::default()
          },
        )
        .await
        .map_err(|e| Error::K8sError(format!("{e}")))
        .map(|stream| stream.lines());

      if let Ok(mut logs_stream) = logs_stream_rslt {
        while let Ok(line_opt) = logs_stream.try_next().await {
          if let Some(line) = line_opt {
            println!("{}", line);
          } else {
            attempt += 1;
          }
        }
      } else {
        attempt += 1;
      }

      container_status =
        get_container_status(cfs_session_pod, &ansible_container.name);
    }
  }

  Ok(())
}

pub async fn get_cfs_session_container_ansible_logs_details(
  client: kube::Client,
  cfs_session_name: &str,
) -> Result<(Container, Pod, Api<Pod>), Error> {
  let container_name = "ansible";

  let pods_api: kube::Api<Pod> = kube::Api::namespaced(client, "services");

  let params = kube::api::ListParams::default()
    .limit(1)
    .labels(format!("cfsession={}", cfs_session_name).as_str());

  let mut pods = pods_api
    .list(&params)
    .await
    .map_err(|e| Error::K8sError(format!("{e}")))?;

  let mut i = 0;
  let max = 30;
  let delay_secs = 2;

  // Waiting for pod to start
  while pods.items.is_empty() && i <= max {
    println!(
            "Waiting k8s to create pod/container for cfs session '{}'. Trying again in {} secs. Attempt {} of {}",
            cfs_session_name,
            delay_secs,
            i + 1,
            max
        );
    i += 1;
    tokio::time::sleep(time::Duration::from_secs(delay_secs)).await;
    pods = pods_api
      .list(&params)
      .await
      .map_err(|e| Error::K8sError(format!("{e}")))?;
  }

  if pods.items.is_empty() {
    return Err(
      Error::K8sError(format!(
        "Pod for cfs session '{}' not created. Aborting operation.",
        cfs_session_name
      ))
      .into(),
    );
  }

  let cfs_session_pod = pods.items[0].clone();

  let cfs_session_pod_name = cfs_session_pod.metadata.name.clone().unwrap();
  log::info!("Pod name: {}", cfs_session_pod_name);

  let mut containers = cfs_session_pod.spec.as_ref().unwrap().containers.iter();

  log::debug!(
    "Containers found in pod {}: {:#?}",
    cfs_session_pod_name,
    containers
  );

  let ansible_container: &Container = containers
    .find(|container| container.name.eq(container_name))
    .unwrap();

  let mut container_status =
    get_container_status(&cfs_session_pod, &ansible_container.name);

  let mut i = 0;
  let max = 300;

  // Waiting for container ansible-x to start
  while container_status.as_ref().is_none()
    || container_status.as_ref().unwrap().waiting.is_some() && i <= max
  {
    println!(
            "Container ({}) status missing or 'waiting'. Checking again in 2 secs. Attempt {} of {}",
            ansible_container.name,
            i + 1,
            max
        );
    i += 1;
    tokio::time::sleep(time::Duration::from_secs(2)).await;
    let pods = pods_api
      .list(&params)
      .await
      .map_err(|e| Error::K8sError(format!("{e}")))?;
    container_status =
      get_container_status(&pods.items[0], &ansible_container.name);
    log::debug!(
      "Container status:\n{:#?}",
      container_status.as_ref().unwrap()
    );
  }

  if container_status.as_ref().unwrap().waiting.is_some() {
    return Err(
      Error::K8sError(format!(
        "Container ({}) status is waiting. Aborting operation.",
        ansible_container.name
      ))
      .into(),
    );
  }

  Ok((ansible_container.clone(), cfs_session_pod.clone(), pods_api))
}

pub fn get_container_status(
  pod: &k8s_openapi::api::core::v1::Pod,
  container_name: &String,
) -> Option<k8s_openapi::api::core::v1::ContainerState> {
  let container_status = pod
    .status
    .as_ref()
    .unwrap()
    .container_statuses
    .as_ref()
    .and_then(|status_vec| {
      status_vec
        .iter()
        .find(|container_status| container_status.name.eq(container_name))
    });

  match container_status {
    Some(container_status_aux) => container_status_aux.state.clone(),
    None => None,
  }
}

pub async fn attach_cfs_session_container_target_k8s_service_name(
  client: kube::Client,
  cfs_session_name: &str,
) -> Result<AttachedProcess, Error> {
  let pods_fabric: Api<Pod> = Api::namespaced(client.clone(), "services");

  let params = kube::api::ListParams::default()
    .limit(1)
    .labels(format!("cfsession={}", cfs_session_name).as_str());

  let mut pods = pods_fabric.list(&params).await.map_err(|e| {
    Error::K8sError(format!("ERROR - kubernetes: Reason:\n{e}"))
  })?;

  let mut i = 0;
  let max = 30;

  // Waiting for pod to start
  while pods.items.is_empty() && i <= max {
    println!(
            "Pod for cfs session {} not ready. Trying again in 2 secs. Attempt {} of {}",
            cfs_session_name,
            i + 1,
            max
        );
    i += 1;
    tokio::time::sleep(time::Duration::from_secs(2)).await;
    pods = pods_fabric
      .list(&params)
      .await
      .map_err(|e| Error::K8sError(format!("ERROR - Kubernetes: {}", e)))?;
  }

  if pods.items.is_empty() {
    return Err(Error::K8sError(format!(
      "Pod for cfs session {} not ready. Aborting operation",
      cfs_session_name
    )));
  }

  let console_operator_pod = &pods.items[0].clone();

  let console_operator_pod_name =
    console_operator_pod.metadata.name.clone().unwrap();

  let attached = pods_fabric
        .exec(
            &console_operator_pod_name,
            vec![
                "sh",
                "-c",
                "cat /inventory/hosts/01-cfs-generated.yaml | grep cray-ims- | head -n 1",
            ],
            &AttachParams::default()
                .container("cray-console-operator")
                .stderr(false),
        )
        .await
        .unwrap();

  let mut output = get_output(attached).await;
  log::info!("{output}");

  output = output.trim().to_string();

  println!("{output}");

  output
    .strip_prefix("ansible_host: ")
    .unwrap()
    .strip_suffix("-service.ims.svc.cluster.local")
    .unwrap();

  println!("{output}");

  let ansible_target_container_label = output + "-customize";

  println!("{ansible_target_container_label}");

  // Find ansible target container

  let pods_fabric: Api<Pod> = Api::namespaced(client, "ims");

  let params = kube::api::ListParams::default()
    .limit(1)
    .labels(format!("job-name={}", ansible_target_container_label).as_str());

  let mut pods = pods_fabric.list(&params).await.map_err(|e| {
    Error::K8sError(format!("ERROR - kubernetes: Reason:\n{e}"))
  })?;

  let mut i = 0;
  let max = 30;

  // Waiting for pod to start
  while pods.items.is_empty() && i <= max {
    println!(
            "Pod for cfs session {} not ready. Trying again in 2 secs. Attempt {} of {}",
            cfs_session_name,
            i + 1,
            max
        );
    i += 1;
    tokio::time::sleep(time::Duration::from_secs(2)).await;
    pods = pods_fabric.list(&params).await.unwrap();
  }

  if pods.items.is_empty() {
    return Err(Error::K8sError(format!(
      "Pod for cfs session {} not ready. Aborting operation",
      cfs_session_name
    )));
  }

  let console_operator_pod = &pods.items[0].clone();

  log::info!("Connecting to console ansible target container");

  let console_operator_pod_name =
    console_operator_pod.metadata.name.clone().unwrap();

  let command = vec!["bash"]; // Enter the container and open conman to access node's console
                              // let command = vec!["bash"]; // Enter the container and open bash to start an interactive
                              // terminal session

  pods_fabric
    .exec(
      &console_operator_pod_name,
      command,
      &AttachParams::default()
        .container("sshd")
        .stdin(true)
        .stdout(true)
        .stderr(false) // Note to self: tty and stderr cannot both be true
        .tty(true),
    )
    .await
    .map_err(|e| {
      Error::K8sError(format!(
        "Error attaching to container 'sshd' in pod {}.\nReason:\n{}\n. Exit",
        console_operator_pod_name, e
      ))
    })
}

pub async fn get_output(mut attached: AttachedProcess) -> String {
  let stdout = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());
  let out = stdout
    .filter_map(|r| async {
      r.ok().and_then(|v| String::from_utf8(v.to_vec()).ok())
    })
    .collect::<Vec<_>>()
    .await
    .join("");
  attached.join().await.unwrap();
  out
}

pub async fn delete_session_pod(
  shasta_token: &str,
  vault_base_url: &str,
  site_name: &str,
  // vault_role_id: &str,
  k8s_api_url: &str,
  cfs_session_name: &str,
) -> Result<(), Error> {
  let shasta_k8s_secrets = fetch_shasta_k8s_secrets_from_vault(
    vault_base_url,
    shasta_token,
    site_name,
    // vault_role_id,
  )
  .await?;

  let client = get_client(k8s_api_url, shasta_k8s_secrets).await?;

  let pods_api: kube::Api<Pod> = kube::Api::namespaced(client, "services");

  let params = kube::api::ListParams::default()
    .limit(1)
    .labels(format!("cfsession={}", cfs_session_name).as_str());

  let pods = pods_api
    .list(&params)
    .await
    .map_err(|e| Error::K8sError(e.to_string()))?;
  let cfs_session_pod = &pods.items[0].clone();

  let cfs_session_pod_name = cfs_session_pod.metadata.name.clone().unwrap();
  log::info!("Pod to delete: {}", cfs_session_pod_name);

  // Delete Pod
  let dp = DeleteParams::default();
  let _ = pods_api.delete(&cfs_session_pod_name, &dp).await;

  Ok(())
}
