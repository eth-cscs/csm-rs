//! `AuthenticationTrait` impl for [`super::Csm`].

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
    let keycloak_base_url = self.keycloak_base_url();

    let token = get_token_from_shasta_endpoint(
      &keycloak_base_url,
      &self.root_cert,
      username,
      password,
    )
    .await
    .map_err(Error::from)?;

    authentication::validate_api_token(&self.base_url, &token, &self.root_cert)
      .await?;

    Ok(token)
  }

  async fn exchange_api_token(
    &self,
    subject_token: &str,
  ) -> Result<String, Error> {
    let token_exchange = self.token_exchange.as_ref().ok_or_else(|| {
      Error::Message(
        "token exchange is not configured for this CSM site".to_string(),
      )
    })?;

    authentication::exchange_token(
      self.client.http(),
      &self.keycloak_base_url(),
      token_exchange,
      subject_token,
    )
    .await
    .map_err(Error::from)
  }
}
