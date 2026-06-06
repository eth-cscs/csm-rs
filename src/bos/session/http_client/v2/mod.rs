//! BOS sessions v2 — `ShastaClient` methods for `/bos/v2/sessions`.

pub(crate) mod types;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// BOS v2 session mirror types. Gated behind the `manta-dispatcher`
/// Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;

use types::BosSession;

use crate::{ShastaClient, common::http, error::Error};

impl ShastaClient {
  /// `POST /bos/v2/sessions` — create a BOS session.
  pub async fn bos_session_v2_post(
    &self,
    token: &str,
    bos_session: BosSession,
  ) -> Result<BosSession, Error> {
    log::debug!(
      "Create BOS session '{}'",
      bos_session.name.as_deref().unwrap_or("unknown")
    );
    log::debug!("Create BOS session request:\n{:#?}", bos_session);

    let api_url = format!("{}/bos/v2/sessions", self.base_url());
    let created: BosSession =
      http::post_json(self.http(), &api_url, token, &bos_session).await?;

    log::debug!(
      "BOS session '{}' created successfully",
      created.name.as_deref().unwrap_or("unknown")
    );
    Ok(created)
  }

  /// `GET /bos/v2/sessions` (or `/bos/v2/sessions/{id}` if `id_opt` is
  /// supplied) — list sessions or fetch one by ID.
  pub async fn bos_session_v2_get(
    &self,
    token: &str,
    id_opt: Option<&str>,
  ) -> Result<Vec<BosSession>, Error> {
    log::debug!("Get BOS sessions '{}'", id_opt.unwrap_or("all available"));

    let api_url = if let Some(id) = id_opt {
      format!("{}/bos/v2/sessions/{}", self.base_url(), id)
    } else {
      format!("{}/bos/v2/sessions", self.base_url())
    };

    if id_opt.is_some() {
      let single: BosSession =
        http::get_json(self.http(), &api_url, token).await?;
      Ok(vec![single])
    } else {
      http::get_json(self.http(), &api_url, token).await
    }
  }

  /// `DELETE /bos/v2/sessions/{id}` — delete a BOS session.
  pub async fn bos_session_v2_delete(
    &self,
    token: &str,
    bos_session_id: &str,
  ) -> Result<(), Error> {
    let api_url =
      format!("{}/bos/v2/sessions/{}", self.base_url(), bos_session_id);
    http::delete(self.http(), &api_url, token).await
  }
}
