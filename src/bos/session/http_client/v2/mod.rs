//! BOS sessions v2 — `ShastaClient` methods for `/bos/v2/sessions`.

pub mod types;

use types::BosSession;

use crate::{ShastaClient, common::http, error::Error};

impl ShastaClient {
  pub async fn bos_session_v2_post(
    &self,
    bos_session: BosSession,
  ) -> Result<BosSession, Error> {
    log::info!(
      "Create BOS session '{}'",
      bos_session.name.as_deref().unwrap_or("unknown")
    );
    log::debug!("Create BOS session request:\n{:#?}", bos_session);

    let api_url = format!("{}/bos/v2/sessions", self.base_url());
    let created: BosSession =
      http::post_json(self.http(), &api_url, self.token(), &bos_session)
        .await?;

    log::info!(
      "BOS session '{}' created successfully",
      created.name.as_deref().unwrap_or("unknown")
    );
    Ok(created)
  }

  pub async fn bos_session_v2_get(
    &self,
    id_opt: Option<&str>,
  ) -> Result<Vec<BosSession>, Error> {
    log::info!("Get BOS sessions '{}'", id_opt.unwrap_or("all available"));

    let api_url = if let Some(id) = id_opt {
      format!("{}/bos/v2/sessions/{}", self.base_url(), id)
    } else {
      format!("{}/bos/v2/sessions", self.base_url())
    };

    if id_opt.is_some() {
      let single: BosSession =
        http::get_json(self.http(), &api_url, self.token()).await?;
      Ok(vec![single])
    } else {
      http::get_json(self.http(), &api_url, self.token()).await
    }
  }

  pub async fn bos_session_v2_delete(
    &self,
    bos_session_id: &str,
  ) -> Result<(), Error> {
    let api_url =
      format!("{}/bos/v2/sessions/{}", self.base_url(), bos_session_id);
    http::delete(self.http(), &api_url, self.token()).await
  }
}
