//! IMS `/v3/public-keys` endpoint bindings.

use serde::{Deserialize, Serialize};

use crate::{ShastaClient, common::http, error::Error};

/// IMS SSH public-key record. Mirrors the `/ims/v3/public-keys` response.
/// `id` and `created` are server-generated, so they are optional on POST.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[allow(missing_docs)]
pub struct PublicKey {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub created: Option<String>,
  pub name: String,
  pub public_key: String,
}

impl ShastaClient {
  /// Get one user public key in IMS. Returns `None` if no key matches the
  /// username or more than one matches.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn ims_public_keys_v3_get_single(
    &self,
    token: &str,
    username_opt: &str,
  ) -> Result<Option<PublicKey>, Error> {
    let mut keys =
      self.ims_public_keys_v3_get(token, Some(username_opt)).await?;
    if keys.len() == 1 {
      Ok(Some(keys.remove(0)))
    } else {
      Ok(None)
    }
  }

  /// Fetch IMS public keys, optionally filtered by `username`. Ref:
  /// <https://apidocs.svc.cscs.ch/paas/ims/operation/get_v3_image/>.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn ims_public_keys_v3_get(
    &self,
    token: &str,
    username_opt: Option<&str>,
  ) -> Result<Vec<PublicKey>, Error> {
    let api_url = format!("{}/ims/v3/public-keys", self.base_url());
    let keys: Vec<PublicKey> =
      http::get_json(self.http(), &api_url, token).await?;
    Ok(match username_opt {
      Some(username) => {
        keys.into_iter().filter(|k| k.name == username).collect()
      }
      None => keys,
    })
  }
}
