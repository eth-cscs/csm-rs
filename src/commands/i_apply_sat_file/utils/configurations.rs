use std::collections::BTreeMap;

use crate::{
  cfs::{
    self,
    configuration::http_client::v2::types::{
      cfs_configuration_request::CfsConfigurationRequest,
      cfs_configuration_response::CfsConfigurationResponse,
    },
  },
  error::Error,
};

use super::{configuration, image, sessiontemplate};

#[allow(clippy::too_many_arguments)]
pub async fn create_cfs_configuration_from_sat_file(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  gitea_base_url: &str,
  gitea_token: &str,
  cray_product_catalog: &BTreeMap<String, String>,
  sat_file_configuration_yaml: &serde_yaml::Value,
  dry_run: bool,
  site_name: &str,
  overwrite: bool,
) -> Result<CfsConfigurationResponse, Error> {
  log::debug!(
    "Convert CFS configuration in SAT file (yaml):\n{:#?}",
    sat_file_configuration_yaml
  );

  let (cfs_configuration_name, cfs_configuration) =
    CfsConfigurationRequest::from_sat_file_serde_yaml(
      shasta_root_cert,
      gitea_base_url,
      gitea_token,
      sat_file_configuration_yaml,
      cray_product_catalog,
      site_name,
      socks5_proxy,
    )
    .await?;

  if dry_run {
    log::info!(
      "Dry run mode: Create CFS configuration:\n{}",
      serde_json::to_string_pretty(&cfs_configuration)?
    );

    // Generate mock CFS configuration
    let cfs_configuration = CfsConfigurationResponse {
      name: cfs_configuration_name,
      last_updated: "".to_string(),
      layers: Vec::new(),
      additional_inventory: None,
    };

    // Return mock CFS configuration
    Ok(cfs_configuration)
  } else {
    cfs::configuration::utils::create_new_configuration(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      &cfs_configuration,
      &cfs_configuration_name,
      overwrite,
    )
    .await
  }
}

#[allow(clippy::too_many_arguments)]
pub async fn create_cfs_configuration_struct_from_sat_file(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  gitea_base_url: &str,
  gitea_token: &str,
  cray_product_catalog: &BTreeMap<String, String>,
  sat_file_configuration_yaml: &configuration::Configuration,
  dry_run: bool,
  site_name: &str,
  overwrite: bool,
) -> Result<CfsConfigurationResponse, Error> {
  log::debug!(
    "Convert CFS configuration in SAT file (yaml):\n{:#?}",
    sat_file_configuration_yaml
  );

  let (cfs_configuration_name, cfs_configuration) =
    CfsConfigurationRequest::from_sat_file_struct_serde_yaml(
      shasta_root_cert,
      gitea_base_url,
      gitea_token,
      sat_file_configuration_yaml,
      cray_product_catalog,
      site_name,
      socks5_proxy,
    )
    .await?;

  if dry_run {
    log::info!(
      "Dry run mode: Create CFS configuration:\n{}",
      serde_json::to_string_pretty(&cfs_configuration)?
    );

    // Generate mock CFS configuration
    let cfs_configuration = CfsConfigurationResponse {
      name: cfs_configuration_name,
      last_updated: "".to_string(),
      layers: Vec::new(),
      additional_inventory: None,
    };

    // Return mock CFS configuration
    Ok(cfs_configuration)
  } else {
    cfs::configuration::utils::create_new_configuration(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      &cfs_configuration,
      &cfs_configuration_name,
      overwrite,
    )
    .await
  }
}

pub fn validate_sat_file_configurations_section(
  configuration_yaml_vec: &[configuration::Configuration],
  image_yaml_vec_opt: &[image::Image],
  sessiontemplate_yaml_vec_opt: &[sessiontemplate::SessionTemplate],
) -> Result<(), Error> {
  // Validate 'configurations' sections
  if !configuration_yaml_vec.is_empty()
    && image_yaml_vec_opt.is_empty()
    && sessiontemplate_yaml_vec_opt.is_empty()
  {
    return Err(Error::Message(
        "Incorrect SAT file. Please define either an 'images' or a 'session_templates' section. Exit"
            .to_string(),
      ));
  }

  Ok(())
}
