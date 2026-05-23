use serde_json::Value;

use crate::{
  common::http,
  error::Error,
  hsm::group::types::{Group, Member, Members},
};

pub async fn get_raw(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  group_name_opt: Option<&String>,
) -> Result<reqwest::Response, Error> {
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;

  let api_url = if let Some(group_name) = group_name_opt {
    format!("{}/smd/hsm/v2/groups/{}", shasta_base_url, group_name)
  } else {
    format!("{}/smd/hsm/v2/groups", shasta_base_url)
  };

  client
    .get(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)
}

pub async fn get_one(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  label: &str,
) -> Result<Group, Error> {
  let client = http::build_client(root_cert, socks5_proxy)?;
  let api_url = format!("{}/smd/hsm/v2/groups/{}", base_url, label);

  let response = client.get(api_url).bearer_auth(auth_token).send().await?;

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

pub async fn get(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  label_vec_opt: Option<&[String]>,
  tag_vec_opt: Option<&[String]>,
) -> Result<Vec<Group>, Error> {
  let client = http::build_client(root_cert, socks5_proxy)?;
  let api_url = format!("{}/smd/hsm/v2/groups", base_url);

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

  let response = client
    .get(api_url)
    .query(query.as_slice())
    .bearer_auth(auth_token)
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

pub async fn get_all(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Vec<Group>, Error> {
  get(shasta_token, shasta_base_url, shasta_root_cert, socks5_proxy, None, None).await
}

pub async fn get_hsm_group_vec(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  hsm_group_name_opt: Option<&String>,
) -> Result<Vec<Group>, Error> {
  let json_response =
    get_all(shasta_token, shasta_base_url, shasta_root_cert, socks5_proxy).await?;

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

pub async fn post(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  group: Group,
) -> Result<String, Error> {
  log::info!("Add/Create HSM group");
  log::debug!("Add HSM group payload:\n{:#?}", group);

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!("{}/smd/hsm/v2/groups", shasta_base_url);

  let response = client
    .post(api_url)
    .bearer_auth(shasta_token)
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

pub async fn create_new_group(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
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

  let add_group_rslt = post(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    socks5_proxy,
    group.clone(),
  )
  .await;

  log::info!("Group created: {:?}", add_group_rslt);

  Ok(group)
}

pub async fn delete_group(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  hsm_group_name: &String,
) -> Result<Value, Error> {
  log::info!("Delete HSM group '{}'", hsm_group_name);

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let url_api =
    format!("{}/smd/hsm/v2/groups/{}", shasta_base_url, hsm_group_name);

  let response = client
    .delete(url_api)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  http::handle_json_or_text_response(response).await
}

pub async fn post_member(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  hsm_group_name: &str,
  member: Member,
) -> Result<Value, Error> {
  log::info!("Add members {:?} to group '{}'", member, hsm_group_name);
  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!(
    "{}/smd/hsm/v2/groups/{}/members",
    shasta_base_url, hsm_group_name
  );

  let response = client
    .post(api_url)
    .bearer_auth(shasta_token)
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

pub async fn delete_member(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  hsm_group_name: &str,
  member_id: &str,
) -> Result<(), Error> {
  log::info!("Delete member {}/{}", hsm_group_name, member_id);

  let client = http::build_client(shasta_root_cert, socks5_proxy)?;
  let api_url = format!(
    "{}/smd/hsm/v2/groups/{}/members/{}",
    shasta_base_url, hsm_group_name, member_id
  );

  let response = client
    .delete(api_url)
    .bearer_auth(shasta_token)
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
