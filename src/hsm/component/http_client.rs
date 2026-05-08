use serde_json::Value;

use crate::{error::Error, hsm::component::types::Component};

use super::types::{
  ComponentArray, ComponentArrayPostArray, ComponentArrayPostByNidQuery,
  ComponentArrayPostQuery, ComponentPut,
};

pub async fn get_all(
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  auth_token: &str,
  nid_only: Option<&str>,
) -> Result<ComponentArray, Error> {
  get(
    base_url, root_cert, socks5_proxy, auth_token, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, nid_only,
  )
  .await
}

pub async fn get_all_nodes(
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  auth_token: &str,
  nid_only: Option<&str>,
) -> Result<ComponentArray, Error> {
  get(
    base_url,
    root_cert,
    socks5_proxy,
    auth_token,
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
    None,
    nid_only,
  )
  .await
}

pub fn filter(component_vec: &mut Vec<Component>, xname_list: &[String]) {
  component_vec.retain(|component| {
    if let Some(xname) = &component.id {
      xname_list.contains(xname)
    } else {
      false
    }
  });
}

pub async fn get_and_filter(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname_list: &[String],
) -> Result<Vec<Component>, Error> {
  let mut component_vec = get_all(base_url, root_cert, socks5_proxy, auth_token, None)
    .await?
    .components
    .unwrap_or_default();

  filter(&mut component_vec, xname_list);

  Ok(component_vec)
}

pub async fn get(
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  auth_token: &str,
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
  role_only: Option<&str>,
  nid_only: Option<&str>,
) -> Result<ComponentArray, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

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
    ("roleonly", role_only),
    ("nidonly", nid_only),
  ];

  if let Some(mut nid_vec_query) = nid_vec_query.take() {
    query_params.append(&mut nid_vec_query);
  }

  let api_url: String =
    format!("{}/{}", base_url, "smd/hsm/v2/State/Components");

  let response = client
    .get(api_url)
    .query(&query_params)
    .bearer_auth(auth_token)
    .send()
    .await?;

  if !response.status().is_success() {
    match response.status() {
      reqwest::StatusCode::UNAUTHORIZED => {
        let error_payload = response.text().await?;
        let error = Error::Message(error_payload);
        return Err(error);
      }
      _ => {
        let error_payload = response.text().await?;
        let error = Error::Message(error_payload);
        return Err(error);
      }
    }
  }

  response
    .json::<ComponentArray>()
    .await
    .map_err(|e| Error::NetError(e))
}

pub async fn get_one(
  base_url: &str,
  auth_token: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
) -> Result<Component, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url: String =
    format!("{}/{}/{}", base_url, "hsm/v2/State/Components", xname);

  let response = client.get(api_url).bearer_auth(auth_token).send().await?;

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

  response
    .json()
    .await
    .map_err(|error| Error::NetError(error))
}

pub async fn post(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  component: ComponentArrayPostArray,
) -> Result<(), Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url: String = base_url.to_owned() + "/hsm/v2/State/Components";

  let response = client
    .post(api_url)
    .bearer_auth(auth_token)
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

pub async fn post_query(
  base_url: &str,
  auth_token: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  component: ComponentArrayPostQuery,
) -> Result<ComponentArray, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url: String = base_url.to_owned() + "/hsm/v2/State/Components";

  let response = client
    .post(api_url)
    .bearer_auth(auth_token)
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

  response
    .json()
    .await
    .map_err(|error| Error::NetError(error))
}

pub async fn post_bynid_query(
  base_url: &str,
  auth_token: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  component: ComponentArrayPostByNidQuery,
) -> Result<ComponentArray, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url: String =
    base_url.to_owned() + "/hsm/v2/State/Components/ByNID/Query";

  let response = client
    .post(api_url)
    .bearer_auth(auth_token)
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

  response
    .json()
    .await
    .map_err(|error| Error::NetError(error))
}

pub async fn put(
  base_url: &str,
  auth_token: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
  component: ComponentPut,
) -> Result<(), Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url: String =
    format!("{}/{}/{}", base_url, "hsm/v2/State/Components/", xname);

  let response = client
    .put(api_url)
    .bearer_auth(auth_token)
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

  response
    .json()
    .await
    .map_err(|error| Error::NetError(error))
}

pub async fn delete_one(
  base_url: &str,
  auth_token: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
  xname: &str,
) -> Result<Value, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url: String =
    format!("{}/{}/{}", base_url, "hsm/v2/State/Components", xname);

  let response = client
    .delete(api_url)
    .bearer_auth(auth_token)
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

  response
    .json()
    .await
    .map_err(|error| Error::NetError(error))
}

pub async fn delete(
  base_url: &str,
  auth_token: &str,
  root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Value, Error> {
  let client_builder = reqwest::Client::builder()
    .add_root_certificate(reqwest::Certificate::from_pem(root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
  };

  let api_url: String = format!("{}/{}", base_url, "hsm/v2/State/Componnets");

  let response = client
    .delete(api_url)
    .bearer_auth(auth_token)
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

  response
    .json()
    .await
    .map_err(|error| Error::NetError(error))
}
