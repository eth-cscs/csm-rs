use serde_json::Value;

use crate::{
  ShastaClient,
  common::http,
  error::Error,
  hsm::group::types::{Group, Member, Members},
};

impl ShastaClient {
  pub async fn hsm_group_get_raw(
    &self,
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
      .bearer_auth(self.token())
      .send()
      .await
      .map_err(Error::NetError)
  }

  pub async fn hsm_group_get_one(&self, label: &str) -> Result<Group, Error> {
    let api_url = format!("{}/smd/hsm/v2/groups/{}", self.base_url(), label);

    let response = self
      .http()
      .get(api_url)
      .bearer_auth(self.token())
      .send()
      .await?;

    if let Err(e) = response.error_for_status_ref() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          let error_payload = response.text().await?;
          return Err(Error::RequestError {
            response: e,
            payload: error_payload,
          });
        }
        _ => {
          let error_payload = response.text().await?;
          return Err(Error::Message(error_payload));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  pub async fn hsm_group_get(
    &self,
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
      .bearer_auth(self.token())
      .send()
      .await?;

    if let Err(e) = response.error_for_status_ref() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          let error_payload = response.text().await?;
          return Err(Error::RequestError {
            response: e,
            payload: error_payload,
          });
        }
        _ => {
          let error_payload = response.text().await?;
          return Err(Error::Message(error_payload));
        }
      }
    }

    response.json().await.map_err(Error::NetError)
  }

  pub async fn hsm_group_get_all(&self) -> Result<Vec<Group>, Error> {
    self.hsm_group_get(None, None).await
  }

  pub async fn hsm_group_get_hsm_group_vec(
    &self,
    hsm_group_name_opt: Option<&String>,
  ) -> Result<Vec<Group>, Error> {
    let json_response = self.hsm_group_get_all().await?;

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

  pub async fn hsm_group_post(&self, group: Group) -> Result<String, Error> {
    log::info!("Add/Create HSM group");
    log::debug!("Add HSM group payload:\n{:#?}", group);

    let api_url = format!("{}/smd/hsm/v2/groups", self.base_url());

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(self.token())
      .json(&group)
      .send()
      .await?;

    log::debug!("Response:\n{:#?}", response);

    if let Err(e) = response.error_for_status_ref() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          let error_payload = response.text().await?;
          return Err(Error::RequestError {
            response: e,
            payload: error_payload,
          });
        }
        _ => {
          let error_payload = response.json().await?;
          return Err(Error::Message(error_payload));
        }
      }
    }

    response.text().await.map_err(Error::NetError)
  }

  pub async fn hsm_group_create_new_group(
    &self,
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

    let add_group_rslt = self.hsm_group_post(group.clone()).await;

    log::info!("Group created: {:?}", add_group_rslt);

    Ok(group)
  }

  pub async fn hsm_group_delete_group(
    &self,
    hsm_group_name: &String,
  ) -> Result<Value, Error> {
    log::info!("Delete HSM group '{}'", hsm_group_name);

    let url_api =
      format!("{}/smd/hsm/v2/groups/{}", self.base_url(), hsm_group_name);

    let response = self
      .http()
      .delete(url_api)
      .bearer_auth(self.token())
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  pub async fn hsm_group_post_member(
    &self,
    hsm_group_name: &str,
    member: Member,
  ) -> Result<Value, Error> {
    log::info!("Add members {:?} to group '{}'", member, hsm_group_name);
    let api_url = format!(
      "{}/smd/hsm/v2/groups/{}/members",
      self.base_url(),
      hsm_group_name
    );

    let response = self
      .http()
      .post(api_url)
      .bearer_auth(self.token())
      .json(&member)
      .send()
      .await?;

    if let Err(e) = response.error_for_status_ref() {
      match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
          let error_payload = response.text().await?;
          return Err(Error::RequestError {
            response: e,
            payload: error_payload,
          });
        }
        _ => {
          let error_payload = response.text().await?;
          return Err(Error::Message(error_payload));
        }
      }
    }

    response
      .json::<Value>()
      .await
      .map_err(|e| Error::Message(e.to_string()))
  }

  pub async fn hsm_group_delete_member(
    &self,
    hsm_group_name: &str,
    member_id: &str,
  ) -> Result<(), Error> {
    log::info!("Delete member {}/{}", hsm_group_name, member_id);

    let api_url = format!(
      "{}/smd/hsm/v2/groups/{}/members/{}",
      self.base_url(),
      hsm_group_name,
      member_id
    );

    let response = self
      .http()
      .delete(api_url)
      .bearer_auth(self.token())
      .send()
      .await
      .map_err(Error::NetError)?;

    if response.status().is_success() {
      Ok(())
    } else {
      let payload = response.text().await.map_err(Error::NetError)?;
      Err(Error::Message(payload))
    }
  }
}
