use manta_backend_dispatcher::{
  error::Error, interfaces::authentication::AuthenticationTrait,
};

use super::Csm;
use crate::common::authentication::{self, get_token_from_shasta_endpoint};

impl AuthenticationTrait for Csm {
  async fn get_api_token(
    &self,
    username: &str,
    password: &str,
  ) -> Result<String, Error> {
    // FIXME: this is not nice but authentication/authorization will potentially move out to an
    // external crate since this is type of logic is external to each site ...
    let base_url = self
      .base_url
      .strip_suffix("/apis")
      .unwrap_or(&self.base_url);

    let keycloak_base_url = base_url.to_string() + "/keycloak";

    /* authentication::get_api_token(
      &self.base_url,
      &self.root_cert,
      &keycloak_base_url,
      site_name,
    )
    .await
    .map_err(|e| Error::Message(e.to_string())) */

    let token = get_token_from_shasta_endpoint(
      &keycloak_base_url,
      &self.root_cert,
      username,
      password,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))?;

    self.validate_api_token(&token).await?;

    Ok(token)
  }

  async fn validate_api_token(&self, token: &str) -> Result<(), Error> {
    authentication::validate_api_token(&self.base_url, token, &self.root_cert)
      .await
      .map_err(|e| Error::Message(e.to_string()))
  }
}
