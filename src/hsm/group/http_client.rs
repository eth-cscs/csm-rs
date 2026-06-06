use crate::{
  ShastaClient,
  common::http,
  error::Error,
  hsm::{
    group::types::{Group, Member, Members},
    types::HsmActionResponse,
  },
};

impl ShastaClient {
  /// Issue the raw `GET /smd/hsm/v2/groups[/{name}]` call and return
  /// the unparsed `reqwest::Response`.
  ///
  /// Useful when the caller needs access to status codes or headers
  /// before deciding how to deserialise the body.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_group_get_raw(
    &self,
    token: &str,
    group_name_opt: Option<&String>,
  ) -> Result<reqwest::Response, Error> {
    let api_url = if let Some(group_name) = group_name_opt {
      format!("{}/smd/hsm/v2/groups/{}", self.base_url(), group_name)
    } else {
      format!("{}/smd/hsm/v2/groups", self.base_url())
    };

    self
      .http()
      .get(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)
  }

  /// Fetch a single HSM group by `label`, returning a strongly-typed
  /// [`Group`].
  ///
  /// `GET /smd/hsm/v2/groups/{label}`. Distinguishes unauthorized
  /// responses as [`Error::RequestError`] so callers can react to token
  /// problems differently from other HTTP errors.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_group_get_one(
    &self,
    token: &str,
    label: &str,
  ) -> Result<Group, Error> {
    let api_url = format!("{}/smd/hsm/v2/groups/{}", self.base_url(), label);

    let response = self.http().get(api_url).bearer_auth(token).send().await?;

    if let Err(e) = response.error_for_status_ref() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          let url = response.url().to_string();
          let error_payload = response.text().await?;
          return Err(Error::RequestError {
            response: e,
            url,
            payload: error_payload,
          });
        }
        status => {
          let status = status.as_u16();
          let url = response.url().to_string();
          let payload = response.text().await?;
          return Err(Error::csm_text_from_response(
            "GET", &url, status, payload,
          ));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  /// List HSM groups, optionally filtered by one or more labels and/or
  /// tags.
  ///
  /// `GET /smd/hsm/v2/groups?group=…&tag=…`. Each value in
  /// `label_vec_opt` and `tag_vec_opt` becomes an additional repeated
  /// query parameter.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_group_get(
    &self,
    token: &str,
    label_vec_opt: Option<&[String]>,
    tag_vec_opt: Option<&[String]>,
  ) -> Result<Vec<Group>, Error> {
    let api_url = format!("{}/smd/hsm/v2/groups", self.base_url());

    let mut query = Vec::new();

    if let Some(label_vec) = label_vec_opt {
      for label in label_vec {
        query.push(("group", label));
      }
    }
    if let Some(tag_vec) = tag_vec_opt {
      for tag in tag_vec {
        query.push(("tag", tag));
      }
    }

    let response = self
      .http()
      .get(api_url)
      .query(query.as_slice())
      .bearer_auth(token)
      .send()
      .await?;

    if let Err(e) = response.error_for_status_ref() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          let url = response.url().to_string();
          let error_payload = response.text().await?;
          return Err(Error::RequestError {
            response: e,
            url,
            payload: error_payload,
          });
        }
        status => {
          let status = status.as_u16();
          let url = response.url().to_string();
          let payload = response.text().await?;
          return Err(Error::csm_text_from_response(
            "GET", &url, status, payload,
          ));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  /// List every HSM group on the system.
  ///
  /// Convenience wrapper for `hsm_group_get(None, None)`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_group_get_all(
    &self,
    token: &str,
  ) -> Result<Vec<Group>, Error> {
    self.hsm_group_get(token, None, None).await
  }

  /// Find every HSM group whose label *contains* `hsm_group_name_opt`
  /// (substring match).
  ///
  /// Returns an empty `Vec` if `hsm_group_name_opt` is `None`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_group_get_hsm_group_vec(
    &self,
    token: &str,
    hsm_group_name_opt: Option<&String>,
  ) -> Result<Vec<Group>, Error> {
    let json_response = self.hsm_group_get_all(token).await?;

    let mut hsm_groups: Vec<Group> = Vec::new();

    if let Some(hsm_group_name) = hsm_group_name_opt {
      for hsm_group in json_response {
        if hsm_group.label.contains(hsm_group_name) {
          hsm_groups.push(hsm_group.clone());
        }
      }
    }

    Ok(hsm_groups)
  }

  /// Create a new HSM group.
  ///
  /// `POST /smd/hsm/v2/groups`. Returns the response body as text
  /// because CSM's success payload here is a plain string id, not JSON.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_group_post(
    &self,
    token: &str,
    group: Group,
  ) -> Result<String, Error> {
    log::debug!("Add/Create HSM group");
    log::debug!("Add HSM group payload:\n{:#?}", group);

    let api_url = format!("{}/smd/hsm/v2/groups", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&group)
      .send()
      .await?;

    log::debug!("Response:\n{:#?}", response);

    if let Err(e) = response.error_for_status_ref() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          let url = response.url().to_string();
          let error_payload = response.text().await?;
          return Err(Error::RequestError {
            response: e,
            url,
            payload: error_payload,
          });
        }
        status => {
          let status = status.as_u16();
          let url = response.url().to_string();
          let payload: serde_json::Value = response.json().await?;
          return Err(Error::csm_from_response(
            "POST", &url, status, payload,
          ));
        }
      }
    }

    response.text().await.map_err(Error::NetError)
  }

  /// Build a [`Group`] from its individual fields and create it via
  /// [`Self::hsm_group_post`].
  ///
  /// Returns the constructed [`Group`] regardless of the response body
  /// shape; success of the underlying POST is logged.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_group_create_new_group(
    &self,
    token: &str,
    hsm_group_name_opt: &str,
    xnames: &[String],
    exclusive: &str,
    description: &str,
    tags: &[String],
  ) -> Result<Group, Error> {
    let myxnames = Members {
      ids: Some(xnames.to_owned()),
    };

    let group = Group {
      label: hsm_group_name_opt.to_owned(),
      description: Option::from(description.to_string().clone()),
      tags: Option::from(tags.to_owned()),
      exclusive_group: Option::from(exclusive.to_string().clone()),
      members: Some(myxnames),
    };

    log::debug!("{:#?}", &group);

    let add_group_rslt = self.hsm_group_post(token, group.clone()).await;

    log::debug!("Group created: {:?}", add_group_rslt);

    Ok(group)
  }

  /// Delete an HSM group by label.
  ///
  /// `DELETE /smd/hsm/v2/groups/{hsm_group_name}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_group_delete_group(
    &self,
    token: &str,
    hsm_group_name: &str,
  ) -> Result<HsmActionResponse, Error> {
    log::debug!("Delete HSM group '{}'", hsm_group_name);
    let url_api =
      format!("{}/smd/hsm/v2/groups/{}", self.base_url(), hsm_group_name);
    let response = self
      .http()
      .delete(url_api)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;
    http::handle_json_response(response, "DELETE").await
  }

  /// Add a member (component xname) to an HSM group.
  ///
  /// `POST /smd/hsm/v2/groups/{hsm_group_name}/members`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_group_post_member(
    &self,
    token: &str,
    hsm_group_name: &str,
    member: Member,
  ) -> Result<HsmActionResponse, Error> {
    log::debug!("Add members {:?} to group '{}'", member, hsm_group_name);
    let api_url = format!(
      "{}/smd/hsm/v2/groups/{}/members",
      self.base_url(),
      hsm_group_name
    );
    let response = self
      .http()
      .post(api_url)
      .bearer_auth(token)
      .json(&member)
      .send()
      .await
      .map_err(Error::NetError)?;
    http::handle_json_response(response, "POST").await
  }

  /// Remove a member (component xname) from an HSM group.
  ///
  /// `DELETE /smd/hsm/v2/groups/{hsm_group_name}/members/{member_id}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn hsm_group_delete_member(
    &self,
    token: &str,
    hsm_group_name: &str,
    member_id: &str,
  ) -> Result<(), Error> {
    log::debug!("Delete member {}/{}", hsm_group_name, member_id);

    let api_url = format!(
      "{}/smd/hsm/v2/groups/{}/members/{}",
      self.base_url(),
      hsm_group_name,
      member_id
    );

    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(token)
      .send()
      .await
      .map_err(Error::NetError)?;

    if response.status().is_success() {
      Ok(())
    } else {
      let status = response.status().as_u16();
      let url = response.url().to_string();
      let payload = response.text().await.map_err(Error::NetError)?;
      Err(Error::csm_text_from_response("DELETE", &url, status, payload))
    }
  }
}
