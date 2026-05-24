use crate::{cfs::component::http_client::v3::types::Component, error::Error};

pub async fn update_component_desired_configuration(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
  desired_configuration: &str,
  enabled: bool,
) {
  let component = Component {
    id: Some(xname.to_string()),
    desired_config: Some(desired_configuration.to_string()),
    state: None,
    error_count: None,
    retry_policy: None,
    enabled: Some(enabled),
    tags: None,
    configuration_status: None,
    logs: None,
  };

  let Ok(client) = crate::ShastaClient::new(
    shasta_base_url,
    shasta_token,
    shasta_root_cert.to_vec(),
    socks5_proxy.map(str::to_owned),
  ) else {
    return;
  };

  let _ = client.cfs_component_v3_patch_component(component).await;
}

pub async fn update_component_list_desired_configuration(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xnames: &[String],
  desired_configuration: &str,
  enabled: bool,
) -> Result<(), Error> {
  let mut component_list = Vec::new();

  for xname in xnames {
    let component = Component {
      id: Some(xname.to_string()),
      desired_config: Some(desired_configuration.to_string()),
      state: None,
      error_count: None,
      retry_policy: None,
      enabled: Some(enabled),
      tags: None,
      configuration_status: None,
      logs: None,
    };

    component_list.push(component);
  }

  crate::ShastaClient::new(
    shasta_base_url,
    shasta_token,
    shasta_root_cert.to_vec(),
    socks5_proxy.map(str::to_owned),
  )?
  .cfs_component_v3_patch_component_list(component_list)
  .await?;

  Ok(())
}
