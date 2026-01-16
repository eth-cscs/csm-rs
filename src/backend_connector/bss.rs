use manta_backend_dispatcher::{
  error::Error, interfaces::bss::BootParametersTrait,
  types::bss::BootParameters as FrontEndBootParameters,
};

use super::Csm;
use crate::bss;

impl BootParametersTrait for Csm {
  async fn get_all_bootparameters(
    &self,
    auth_token: &str,
  ) -> Result<Vec<FrontEndBootParameters>, Error> {
    let boot_parameter_vec =
      bss::http_client::get_all(auth_token, &self.base_url, &self.root_cert)
        .await
        .map_err(|e| Error::Message(e.to_string()))?;

    let boot_parameter_infra_vec = boot_parameter_vec
      .into_iter()
      .map(|boot_parameter| boot_parameter.into())
      .collect();

    Ok(boot_parameter_infra_vec)
  }

  async fn get_bootparameters(
    &self,
    auth_token: &str,
    nodes: &[String],
  ) -> Result<Vec<FrontEndBootParameters>, Error> {
    let boot_parameter_vec = bss::http_client::get_multiple(
      auth_token,
      &self.base_url,
      &self.root_cert,
      nodes,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))?;

    let boot_parameter_infra_vec = boot_parameter_vec
      .into_iter()
      .map(|boot_parameter| boot_parameter.into())
      .collect();

    Ok(boot_parameter_infra_vec)
  }

  async fn add_bootparameters(
    &self,
    auth_token: &str,
    boot_parameters: &FrontEndBootParameters,
  ) -> Result<(), Error> {
    bss::http_client::post(
      &self.base_url,
      auth_token,
      &self.root_cert,
      boot_parameters.clone().into(),
    )
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn update_bootparameters(
    &self,
    auth_token: &str,
    boot_parameter: &FrontEndBootParameters,
  ) -> Result<(), Error> {
    bss::http_client::patch(
      &self.base_url,
      auth_token,
      &self.root_cert,
      &boot_parameter.clone().into(),
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn delete_bootparameters(
    &self,
    _auth_token: &str,
    _boot_parameters: &FrontEndBootParameters,
  ) -> Result<String, Error> {
    Err(Error::Message(
      "Delete boot parameters command not implemented for this backend"
        .to_string(),
    ))
  }
}
