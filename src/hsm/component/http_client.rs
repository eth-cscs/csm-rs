//! `ShastaClient` methods for `/smd/hsm/v2/State/Components`.

use serde_json::Value;

use crate::{
  ShastaClient, common::http, error::Error, hsm::component::types::Component,
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
    nid_only: Option<&str>,
  ) -> Result<ComponentArray, Error> {
    self
      .hsm_component_get(
        None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, nid_only,
      )
      .await
  }

  /// Fetch all HSM components with `Type=Node`.
  pub async fn hsm_component_get_all_nodes(
    &self,
    nid_only: Option<&str>,
  ) -> Result<ComponentArray, Error> {
    self
      .hsm_component_get(
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

  pub async fn hsm_component_get_and_filter(
    &self,
    xname_list: &[String],
  ) -> Result<Vec<Component>, Error> {
    let mut component_vec = self
      .hsm_component_get_all(None)
      .await?
      .components
      .unwrap_or_default();

    filter(&mut component_vec, xname_list);

    Ok(component_vec)
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn hsm_component_get(
    &self,
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
      .bearer_auth(self.token())
      .send()
      .await?;

    http::handle_json_or_text_response(response).await
  }

  pub async fn hsm_component_get_one(
    &self,
    xname: &str,
  ) -> Result<Component, Error> {
    let api_url =
      format!("{}/hsm/v2/State/Components/{}", self.base_url(), xname);

    let response = self
      .http()
      .get(api_url)
      .bearer_auth(self.token())
      .send()
      .await?;

    if !response.status().is_success() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          return Err(Error::Message(response.text().await?));
        }
        _ => {
          return Err(Error::CsmError(response.json::<Value>().await?));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  pub async fn hsm_component_post(
    &self,
    component: ComponentArrayPostArray,
  ) -> Result<(), Error> {
    let api_url = format!("{}/hsm/v2/State/Components", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(self.token())
      .json(&component)
      .send()
      .await?;

    if !response.status().is_success() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          return Err(Error::Message(response.text().await?));
        }
        _ => {
          return Err(Error::CsmError(response.json::<Value>().await?));
        }
      }
    }

    Ok(())
  }

  pub async fn hsm_component_post_query(
    &self,
    component: ComponentArrayPostQuery,
  ) -> Result<ComponentArray, Error> {
    let api_url = format!("{}/hsm/v2/State/Components", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(self.token())
      .json(&component)
      .send()
      .await?;

    if !response.status().is_success() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          return Err(Error::Message(response.text().await?));
        }
        _ => {
          return Err(Error::CsmError(response.json::<Value>().await?));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  pub async fn hsm_component_post_bynid_query(
    &self,
    component: ComponentArrayPostByNidQuery,
  ) -> Result<ComponentArray, Error> {
    let api_url =
      format!("{}/hsm/v2/State/Components/ByNID/Query", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(self.token())
      .json(&component)
      .send()
      .await?;

    if !response.status().is_success() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          return Err(Error::Message(response.text().await?));
        }
        _ => {
          return Err(Error::CsmError(response.json::<Value>().await?));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  pub async fn hsm_component_put(
    &self,
    xname: &str,
    component: ComponentPut,
  ) -> Result<(), Error> {
    let api_url =
      format!("{}/hsm/v2/State/Components/{}", self.base_url(), xname);

    let response = self
      .http()
      .put(api_url)
      .bearer_auth(self.token())
      .json(&component)
      .send()
      .await?;

    if !response.status().is_success() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          return Err(Error::Message(response.text().await?));
        }
        _ => {
          return Err(Error::CsmError(response.json::<Value>().await?));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  pub async fn hsm_component_delete_one(
    &self,
    xname: &str,
  ) -> Result<Value, Error> {
    let api_url =
      format!("{}/hsm/v2/State/Components/{}", self.base_url(), xname);

    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(self.token())
      .send()
      .await?;

    if !response.status().is_success() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          return Err(Error::Message(response.text().await?));
        }
        _ => {
          return Err(Error::CsmError(response.json::<Value>().await?));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  pub async fn hsm_component_delete(&self) -> Result<Value, Error> {
    let api_url = format!("{}/hsm/v2/State/Componnets", self.base_url());

    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(self.token())
      .send()
      .await?;

    if !response.status().is_success() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          return Err(Error::Message(response.text().await?));
        }
        _ => {
          return Err(Error::CsmError(response.json::<Value>().await?));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }
}
