//! `ShastaClient` methods for `/smd/hsm/v2/State/Components`.

use serde_json::Value;

use crate::{
  ShastaClient,
  common::http,
  error::Error,
  hsm::{component::types::Component, types::HsmActionResponse},
};

use super::types::{
  ComponentArray, ComponentArrayPostArray, ComponentArrayPostByNidQuery,
  ComponentArrayPostQuery, ComponentPut,
};

/// In-place retain of components whose `id` is in `xname_list`.
pub fn filter(component_vec: &mut Vec<Component>, xname_list: &[String]) {
  component_vec.retain(|component| {
    if let Some(xname) = &component.id {
      xname_list.contains(xname)
    } else {
      false
    }
  });
}

impl ShastaClient {
  /// Fetch all HSM components. `nid_only` toggles the lightweight nid-only response.
  pub async fn hsm_component_get_all(
    &self,
    token: &str,
    nid_only: Option<&str>,
  ) -> Result<ComponentArray, Error> {
    self
      .hsm_component_get(
        token, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, nid_only,
      )
      .await
  }

  /// Fetch all HSM components with `Type=Node`.
  pub async fn hsm_component_get_all_nodes(
    &self,
    token: &str,
    nid_only: Option<&str>,
  ) -> Result<ComponentArray, Error> {
    self
      .hsm_component_get(
        token,
        None,
        Some("Node"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        nid_only,
      )
      .await
  }

  /// `GET /hsm/v2/State/Components` then filter the result down to
  /// components whose xname appears in `xname_list`.
  pub async fn hsm_component_get_and_filter(
    &self,
    token: &str,
    xname_list: &[String],
  ) -> Result<Vec<Component>, Error> {
    let mut component_vec = self
      .hsm_component_get_all(token, None)
      .await?
      .components
      .unwrap_or_default();

    filter(&mut component_vec, xname_list);

    Ok(component_vec)
  }

  /// `GET /hsm/v2/State/Components` with the full set of HSM query
  /// parameters (id, type, state, flag, role, subrole, enabled,
  /// software status, subtype, arch, class, nid, nid range, partition,
  /// group, and the `*only` projection toggles).
  #[allow(clippy::too_many_arguments)]
  pub async fn hsm_component_get(
    &self,
    token: &str,
    id: Option<&str>,
    r#type: Option<&str>,
    state: Option<&str>,
    flag: Option<&str>,
    role: Option<&str>,
    subrole: Option<&str>,
    enabled: Option<&str>,
    software_status: Option<&str>,
    subtype: Option<&str>,
    arch: Option<&str>,
    class: Option<&str>,
    nid: Option<&str>,
    nid_start: Option<&str>,
    nid_end: Option<&str>,
    partition: Option<&str>,
    group: Option<&str>,
    state_only: Option<&str>,
    flag_only: Option<&str>,
    // role_only: Option<&str>,
    nid_only: Option<&str>,
  ) -> Result<ComponentArray, Error> {
    let mut nid_vec_query = nid.map(|nids| {
      nids
        .split(",")
        .map(|nid| ("nid", Some(nid)))
        .collect::<Vec<(&str, Option<&str>)>>()
    });

    let mut query_params = vec![
      ("id", id),
      ("type", r#type),
      ("state", state),
      ("flag", flag),
      ("role", role),
      ("subrole", subrole),
      ("enabled", enabled),
      ("softwarestatus", software_status),
      ("subtype", subtype),
      ("arch", arch),
      ("class", class),
      ("nidstart", nid_start),
      ("nidend", nid_end),
      ("partition", partition),
      ("group", group),
      ("stateonly", state_only),
      ("flagonly", flag_only),
      ("nidonly", nid_only),
    ];

    if let Some(mut nid_vec_query) = nid_vec_query.take() {
      query_params.append(&mut nid_vec_query);
    }

    let api_url = format!("{}/smd/hsm/v2/State/Components", self.base_url());

    let response = self
      .http()
      .get(api_url)
      .query(&query_params)
      .bearer_auth(token)
      .send()
      .await?;

    http::handle_json_or_text_response(response).await
  }

  /// `GET /hsm/v2/State/Components/{xname}` — fetch a single component.
  pub async fn hsm_component_get_one(
    &self,
    token: &str,
    xname: &str,
  ) -> Result<Component, Error> {
    let api_url =
      format!("{}/hsm/v2/State/Components/{}", self.base_url(), xname);

    let response = self.http().get(api_url).bearer_auth(token).send().await?;

    if !response.status().is_success() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          return Err(Error::Message(response.text().await?));
        }
        _ => {
          let status = response.status().as_u16();
          let url = response.url().to_string();
          let payload = response.json::<Value>().await?;
          return Err(Error::csm_from_response(
            "GET",
            &url,
            status,
            payload,
          ));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  /// `POST /hsm/v2/State/Components` — create components in bulk.
  pub async fn hsm_component_post(
    &self,
    token: &str,
    component: ComponentArrayPostArray,
  ) -> Result<(), Error> {
    let api_url = format!("{}/hsm/v2/State/Components", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&component)
      .send()
      .await?;

    if !response.status().is_success() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          return Err(Error::Message(response.text().await?));
        }
        _ => {
          let status = response.status().as_u16();
          let url = response.url().to_string();
          let payload = response.json::<Value>().await?;
          return Err(Error::csm_from_response(
            "POST",
            &url,
            status,
            payload,
          ));
        }
      }
    }

    Ok(())
  }

  /// `POST /hsm/v2/State/Components` query — components matching the
  /// supplied criteria.
  pub async fn hsm_component_post_query(
    &self,
    token: &str,
    component: ComponentArrayPostQuery,
  ) -> Result<ComponentArray, Error> {
    let api_url = format!("{}/hsm/v2/State/Components", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&component)
      .send()
      .await?;

    if !response.status().is_success() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          return Err(Error::Message(response.text().await?));
        }
        _ => {
          let status = response.status().as_u16();
          let url = response.url().to_string();
          let payload = response.json::<Value>().await?;
          return Err(Error::csm_from_response(
            "POST",
            &url,
            status,
            payload,
          ));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  /// `POST /hsm/v2/State/Components/ByNID/Query` — components matching
  /// the supplied NID query.
  pub async fn hsm_component_post_bynid_query(
    &self,
    token: &str,
    component: ComponentArrayPostByNidQuery,
  ) -> Result<ComponentArray, Error> {
    let api_url =
      format!("{}/hsm/v2/State/Components/ByNID/Query", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&component)
      .send()
      .await?;

    if !response.status().is_success() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          return Err(Error::Message(response.text().await?));
        }
        _ => {
          let status = response.status().as_u16();
          let url = response.url().to_string();
          let payload = response.json::<Value>().await?;
          return Err(Error::csm_from_response(
            "POST",
            &url,
            status,
            payload,
          ));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  /// `PUT /hsm/v2/State/Components/{xname}` — replace a component.
  pub async fn hsm_component_put(
    &self,
    token: &str,
    xname: &str,
    component: ComponentPut,
  ) -> Result<(), Error> {
    let api_url =
      format!("{}/hsm/v2/State/Components/{}", self.base_url(), xname);

    let response = self
      .http()
      .put(api_url)
      .bearer_auth(token)
      .json(&component)
      .send()
      .await?;

    if !response.status().is_success() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          return Err(Error::Message(response.text().await?));
        }
        _ => {
          let status = response.status().as_u16();
          let url = response.url().to_string();
          let payload = response.json::<Value>().await?;
          return Err(Error::csm_from_response(
            "PUT",
            &url,
            status,
            payload,
          ));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  /// `DELETE /hsm/v2/State/Components/{xname}` — remove a single
  /// component.
  pub async fn hsm_component_delete_one(
    &self,
    token: &str,
    xname: &str,
  ) -> Result<HsmActionResponse, Error> {
    let api_url =
      format!("{}/hsm/v2/State/Components/{}", self.base_url(), xname);
    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;
    http::handle_json_response(response, "DELETE").await
  }

  /// `DELETE /hsm/v2/State/Components` — remove all components.
  pub async fn hsm_component_delete(
    &self,
    token: &str,
  ) -> Result<HsmActionResponse, Error> {
    let api_url = format!("{}/hsm/v2/State/Components", self.base_url());
    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;
    http::handle_json_response(response, "DELETE").await
  }
}
