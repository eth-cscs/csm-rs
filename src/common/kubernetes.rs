use core::time;
use std::collections::BTreeMap;

use futures::{AsyncBufRead, AsyncBufReadExt, StreamExt, TryStreamExt};

use k8s_openapi::api::core::v1::{ConfigMap, Container, ContainerStatus, Pod};
use kube::api::DeleteParams;
use kube::runtime::reflector::Lookup;
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

use crate::common::vault::http_client::fetch_shasta_k8s_secrets_from_vault;
use crate::error::Error;
use http::Uri;
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
  let max_attempts = 3;

  let namespace = "services";

  let mut attempt = 0;

  let container_name = "git-clone";

  let mut result = i_print_init_container_logs(
    client.clone(),
    cfs_session_name,
    container_name,
    namespace,
    timestamps,
  )
  .await;

  while result.is_err() && attempt < max_attempts {
    attempt += 1;

    println!(
      "Could not get logs for init container '{}'. Trying again. Attempt {} of {}",
      container_name,
      attempt + 1,
      max_attempts
    );
    result = i_print_init_container_logs(
      client.clone(),
      cfs_session_name,
      container_name,
      namespace,
      timestamps,
    )
    .await;
  }

  let mut attempt = 0;

  let container_name = "inventory";

  let mut result = i_print_container_logs(
    client.clone(),
    cfs_session_name,
    container_name,
    namespace,
    timestamps,
  )
  .await;

  while result.is_err() && attempt < max_attempts {
    attempt += 1;

    println!(
      "Could not get logs for init container '{}'. Trying again. Attempt {} of {}",
      container_name,
      attempt + 1,
      max_attempts
    );
    result = i_print_init_container_logs(
      client.clone(),
      cfs_session_name,
      container_name,
      namespace,
      timestamps,
    )
    .await;
  }

  let mut attempt = 0;

  let container_name = "ansible";

  let mut result = i_print_container_logs(
    client.clone(),
    cfs_session_name,
    container_name,
    namespace,
    timestamps,
  )
  .await;

  while result.is_err() && attempt < max_attempts {
    println!(
      "Could not get logs from container '{}'. Trying again. Attempt {} of {}",
      container_name, attempt, max_attempts
    );

    attempt += 1;

    result = i_print_container_logs(
      client.clone(),
      cfs_session_name,
      container_name,
      namespace,
      timestamps,
    )
    .await;
  }

  let mut attempt = 0;

  let container_name = "teardown";

  let mut result = i_print_container_logs(
    client.clone(),
    cfs_session_name,
    container_name,
    namespace,
    timestamps,
  )
  .await;

  while result.is_err() && attempt < max_attempts {
    println!(
      "Could not get logs from container '{}'. Trying again. Attempt {} of {}",
      container_name, attempt, max_attempts
    );

    attempt += 1;

    result = i_print_container_logs(
      client.clone(),
      cfs_session_name,
      container_name,
      namespace,
      timestamps,
    )
    .await;
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

pub async fn i_print_init_container_logs(
  client: kube::Client,
  cfs_session_name: &str,
  init_container_name: &str,
  namespace: &str,
  timestamps: bool,
) -> Result<(), Error> {
  let mut log_stream = get_init_container_logs_stream(
    client,
    cfs_session_name,
    init_container_name,
    namespace,
    format!("cfsession={}", cfs_session_name).as_str(),
    timestamps,
  )
  .await?
  .lines();

  while let Some(line) = log_stream.try_next().await? {
    println!("{}", line);
  }

  Ok(())
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

pub async fn i_print_container_logs(
  client: kube::Client,
  cfs_session_name: &str,
  container_name: &str,
  namespace: &str,
  timestamps: bool,
) -> Result<(), Error> {
  let mut log_stream = get_container_logs_stream(
    client,
    cfs_session_name,
    container_name,
    namespace,
    format!("cfsession={}", cfs_session_name).as_str(),
    timestamps,
  )
  .await?
  .lines();

  while let Some(line) = log_stream.try_next().await? {
    println!("{}", line);
  }

  Ok(())
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

pub async fn get_cfs_session_container_teardown_logs_stream(
  client: kube::Client,
  cfs_session_name: &str,
  timestamps: bool,
) -> Result<impl AsyncBufRead, Error> {
  get_container_logs_stream(
    client,
    cfs_session_name,
    "teardown",
    "services",
    format!("cfsession={}", cfs_session_name).as_str(),
    timestamps,
  )
  .await
}

pub fn get_init_container<'a>(
  pod: &'a Pod,
  name: &str,
) -> Option<&'a Container> {
  pod.spec.as_ref().and_then(|pod_spec| {
    pod_spec
      .init_containers
      .as_ref()
      .and_then(|init_container_vec| {
        init_container_vec
          .iter()
          .find(|container| container.name == name)
      })
  })
}

pub fn init_container_status<'a>(
  pod: &'a Pod,
  container_name: &str,
) -> Option<&'a ContainerStatus> {
  pod.status.as_ref().and_then(|pod_status| {
    pod_status
      .init_container_statuses
      .as_ref()
      .and_then(|status_vec| {
        status_vec
          .iter()
          .find(|container_status| container_status.name == container_name)
      })
  })
}

pub fn is_init_container_state_unkown(pod: &Pod, container_name: &str) -> bool {
  init_container_status(pod, container_name)
    .is_some_and(|container_status| container_status.state.as_ref().is_none())
}

pub fn is_init_container_state_waiting(
  pod: &Pod,
  container_name: &str,
) -> bool {
  init_container_status(pod, container_name).is_some_and(|container_status| {
    container_status
      .state
      .as_ref()
      .is_some_and(|container_state| container_state.waiting.is_some())
  })
}

pub fn get_container<'a>(pod: &'a Pod, name: &str) -> Option<&'a Container> {
  pod.spec.as_ref().and_then(|pod_spec| {
    pod_spec
      .containers
      .iter()
      .find(|container| container.name == name)
  })
}

pub fn container_status<'a>(
  pod: &'a Pod,
  container_name: &str,
) -> Option<&'a ContainerStatus> {
  pod
    .status
    .as_ref()
    .unwrap()
    .container_statuses
    .as_ref()
    .and_then(|status_vec| {
      status_vec
        .iter()
        .find(|container_status| container_status.name == container_name)
    })
}

pub fn is_container_state_unkown(pod: &Pod, container_name: &str) -> bool {
  init_container_status(pod, container_name)
    .is_some_and(|container_status| container_status.state.as_ref().is_none())
}

pub fn is_container_state_waiting(pod: &Pod, container_name: &str) -> bool {
  container_status(pod, container_name).is_some_and(|container_status| {
    container_status
      .state
      .as_ref()
      .is_some_and(|container_state| container_state.waiting.is_some())
  })
}

pub async fn get_init_container_logs_stream(
  client: kube::Client,
  cfs_session_name: &str,
  init_container_name: &str,
  namespace: &str,
  label_selector: &str,
  timestamps: bool,
) -> Result<impl AsyncBufRead, Error> {
  let pods_api: Api<Pod> = Api::namespaced(client, namespace);

  let params = kube::api::ListParams::default()
    .limit(1)
    .labels(label_selector);

  let mut cfs_session_pods = pods_api
    .list(&params)
    .await
    .map_err(|e| Error::K8sError(format!("{e}")))?;

  let mut i = 0;
  let max = 150;
  let delay_secs = 2;

  // Waiting for pod to start
  while cfs_session_pods.items.is_empty() && i <= max {
    println!(
      "Waiting k8s to create pod for cfs session '{}'. Trying again in {} secs. Attempt {} of {}",
      cfs_session_name,
      delay_secs,
      i + 1,
      max
    );

    i += 1;

    tokio::time::sleep(time::Duration::from_secs(delay_secs)).await;

    cfs_session_pods = pods_api
      .list(&params)
      .await
      .map_err(|e| Error::K8sError(format!("{e}")))?;
  }

  if cfs_session_pods.items.is_empty() {
    return Err(Error::K8sError(format!(
      "Pod for cfs session {} missing. Aborting operation",
      cfs_session_name
    )));
  }

  if cfs_session_pods.items.len() > 1 {
    return Err(Error::K8sError(format!(
      "Multiple pods found for cfs session '{}'. Using the first one.",
      cfs_session_name
    )));
  }

  let cfs_session_pod = cfs_session_pods.items.first().unwrap();

  let cfs_session_pod_name = cfs_session_pod.name().unwrap();

  log::info!(
    "Fetching logs from init container '{}' in namespace/pod '{}/{}'",
    init_container_name,
    namespace,
    cfs_session_pod_name,
  );

  let init_container_opt =
    get_init_container(cfs_session_pod, init_container_name);

  if init_container_opt.is_none() {
    return Err(Error::K8sError(format!(
      "Init container '{}' not found in pod '{}'",
      init_container_name, cfs_session_pod_name,
    )));
  }

  // Waiting for init container to start
  let init_container = get_init_container(cfs_session_pod, init_container_name)
    .ok_or(Error::K8sError(format!(
      "Init container '{}' not found in pod '{}'",
      init_container_name, cfs_session_pod_name,
    )))?;

  let mut i = 0;
  let max = 60;

  while (is_init_container_state_unkown(cfs_session_pod, &init_container.name)
    || is_init_container_state_waiting(cfs_session_pod, &init_container.name))
    && i <= max
  {
    println!(
      "Waiting for container '{}' to be ready. Checking again in 2 secs. Attempt {} of {}",
      init_container.name,
      i + 1,
      max
    );

    i += 1;
    tokio::time::sleep(time::Duration::from_secs(2)).await;
  }

  if is_init_container_state_unkown(cfs_session_pod, &init_container.name)
    || is_init_container_state_waiting(cfs_session_pod, &init_container.name)
  {
    return Err(
      Error::K8sError(format!(
        "Init container '{}' not in 'running' state. Aborting operation",
        init_container_name
      ))
      .into(),
    );
  }

  pods_api
    .log_stream(
      cfs_session_pod_name.as_ref(),
      &kube::api::LogParams {
        follow: true,
        container: Some(init_container_name.to_string()),
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
    .labels(label_selector);

  let mut cfs_session_pods = pods_api
    .list(&params)
    .await
    .map_err(|e| Error::K8sError(format!("{e}")))?;

  let mut i = 0;
  let max = 30;
  let delay_secs = 2;

  // Waiting for pod to start
  while cfs_session_pods.items.is_empty() && i <= max {
    println!(
      "Waiting k8s to create pod for cfs session '{}'. Trying again in {} secs. Attempt {} of {}",
      cfs_session_name,
      delay_secs,
      i + 1,
      max
    );

    i += 1;

    tokio::time::sleep(time::Duration::from_secs(delay_secs)).await;

    cfs_session_pods = pods_api
      .list(&params)
      .await
      .map_err(|e| Error::K8sError(format!("{e}")))?;
  }

  if cfs_session_pods.items.is_empty() {
    return Err(Error::K8sError(format!(
      "Pod for cfs session {} missing. Aborting operation",
      cfs_session_name
    )));
  }

  if cfs_session_pods.items.len() > 1 {
    return Err(Error::K8sError(format!(
      "Multiple pods found for cfs session '{}'. Using the first one.",
      cfs_session_name
    )));
  }

  let cfs_session_pod = cfs_session_pods.items.first().unwrap();

  let cfs_session_pod_name = cfs_session_pod.name().unwrap();

  log::info!(
    "Fetching logs from container '{}' in namespace/pod '{}/{}'",
    container_name,
    namespace,
    cfs_session_pod_name,
  );

  let container_opt = get_container(cfs_session_pod, container_name);

  if container_opt.is_none() {
    return Err(Error::K8sError(format!(
      "Container '{}' not found in pod '{}'",
      container_name, cfs_session_pod_name,
    )));
  }

  // Waiting for container to start
  let container = get_container(cfs_session_pod, container_name).ok_or(
    Error::K8sError(format!(
      "Container '{}' not found in pod '{}'",
      container_name, cfs_session_pod_name,
    )),
  )?;

  let mut i = 0;
  let max = 600;

  while (is_container_state_unkown(cfs_session_pod, &container.name)
    || is_container_state_waiting(cfs_session_pod, &container.name))
    && i <= max
  {
    println!(
      "Waiting for container '{}' to be ready. Checking again in 2 secs. Attempt {} of {}",
      container.name,
      i + 1,
      max
    );

    i += 1;
    tokio::time::sleep(time::Duration::from_secs(2)).await;
  }

  if is_container_state_unkown(cfs_session_pod, &container.name)
    || is_container_state_waiting(cfs_session_pod, &container.name)
  {
    return Err(
      Error::K8sError(format!(
        "Container '{}' not ready. Aborting operation",
        container_name
      ))
      .into(),
    );
  }

  pods_api
    .log_stream(
      cfs_session_pod_name.as_ref(),
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
      },
    )
    .await
    .map_err(|e| Error::K8sError(format!("{e}")))
}

pub fn get_container_status(
  pod: &k8s_openapi::api::core::v1::Pod,
  container_name: &String,
) -> Option<ContainerStatus> {
  pod
    .status
    .clone()
    .unwrap()
    .container_statuses
    .and_then(|status_vec| {
      status_vec
        .into_iter()
        .find(|container_status| container_status.name.eq(container_name))
    })
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
