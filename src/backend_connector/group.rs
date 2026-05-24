use std::collections::HashMap;

use manta_backend_dispatcher::{
  error::Error, interfaces::hsm::group::GroupTrait,
  types::Group as FrontEndGroup,
};
use serde_json::Value;

use super::Csm;
use crate::hsm::{self, group::types::Member};

impl GroupTrait for Csm {
  async fn get_group_available(
    &self,
    auth_token: &str,
  ) -> Result<Vec<FrontEndGroup>, Error> {
    let hsm_group_vec = hsm::group::utils::get_group_available(
      auth_token,
      &self.base_url,
      &self.root_cert,
      self.socks5_proxy.as_deref(),
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))?;

    // Convert all HSM groups from mesa to infra
    let hsm_group_backend_vec = hsm_group_vec
      .into_iter()
      .map(hsm::group::types::Group::into)
      .collect();

    Ok(hsm_group_backend_vec)
  }

  async fn get_group_name_available(
    &self,
    auth_token: &str,
  ) -> Result<Vec<String>, Error> {
    hsm::group::utils::get_group_name_available(
      auth_token,
      &self.base_url,
      &self.root_cert,
      self.socks5_proxy.as_deref(),
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn add_group(
    &self,
    auth_token: &str,
    group: FrontEndGroup,
  ) -> Result<FrontEndGroup, Error> {
    let group_csm = self
      .shasta_client(auth_token)?
      .hsm_group_post(group.clone().into())
      .await
      .map_err(|e| Error::Message(e.to_string()))?;

    // let group: FrontEndGroup = group_csm.into();
    log::info!("Group created: {}", group_csm);

    Ok(group)
  }

  // FIXME: rename function to 'get_hsm_group_members'
  async fn get_member_vec_from_group_name_vec(
    &self,
    auth_token: &str,
    hsm_group_name_vec: &[String],
  ) -> Result<Vec<String>, Error> {
    // FIXME: try to merge functions get_member_vec_from_hsm_name_vec_2 and get_member_vec_from_hsm_name_vec
    hsm::group::utils::get_member_vec_from_hsm_name_vec(
      auth_token,
      &self.base_url,
      &self.root_cert,
      self.socks5_proxy.as_deref(),
      hsm_group_name_vec,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn get_group_map_and_filter_by_group_vec(
    &self,
    auth_token: &str,
    hsm_name_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    hsm::group::utils::get_hsm_map_and_filter_by_hsm_name_vec(
      auth_token,
      &self.base_url,
      &self.root_cert,
      self.socks5_proxy.as_deref(),
      hsm_name_vec,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn get_group_map_and_filter_by_member_vec(
    &self,
    auth_token: &str,
    member_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    hsm::group::utils::get_hsm_group_map_and_filter_by_hsm_group_member_vec(
      auth_token,
      &self.base_url,
      &self.root_cert,
      self.socks5_proxy.as_deref(),
      member_vec,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn get_all_groups(
    &self,
    auth_token: &str,
  ) -> Result<Vec<FrontEndGroup>, Error> {
    // Get all HSM groups
    let hsm_group_backend_vec = self
      .shasta_client(auth_token)?
      .hsm_group_get_all()
      .await
      .map_err(|e| Error::Message(e.to_string()))?;

    // Convert all HSM groups from mesa to infra
    let hsm_group_vec = hsm_group_backend_vec
      .into_iter()
      .map(hsm::group::types::Group::into)
      .collect();

    Ok(hsm_group_vec)
  }

  async fn get_group(
    &self,
    auth_token: &str,
    hsm_name: &str,
  ) -> Result<FrontEndGroup, Error> {
    // Get all HSM groups
    let hsm_group_backend_vec = self
      .shasta_client(auth_token)?
      .hsm_group_get(Some(&[hsm_name.to_string()]), None)
      .await
      .map_err(|e| Error::Message(e.to_string()))?;

    // Error if more than one HSM group found
    if hsm_group_backend_vec.len() > 1 {
      return Err(Error::Message(format!(
        "ERROR - multiple HSM groups with name '{}' found. Exit",
        hsm_name
      )));
    }

    // Length-1 guaranteed: the >1 case errored above; the 0 case would
    // have been errored by the upstream get() with GroupNotFound.
    let hsm_group_backend = hsm_group_backend_vec
      .first()
      .expect("exactly one HSM group: checked above")
      .to_owned();

    let hsm_group: FrontEndGroup = hsm_group_backend.into();

    Ok(hsm_group)
  }

  async fn get_groups(
    &self,
    auth_token: &str,
    hsm_name_vec_opt: Option<&[String]>,
  ) -> Result<Vec<FrontEndGroup>, Error> {
    // Get all HSM groups
    let hsm_group_backend_vec = self
      .shasta_client(auth_token)?
      .hsm_group_get(hsm_name_vec_opt, None)
      .await
      .map_err(|e| Error::Message(e.to_string()))?;

    // Convert from HsmGroup (silla) to HsmGroup (infra)
    let mut hsm_group_vec = Vec::new();
    for hsm_group_backend in hsm_group_backend_vec {
      let hsm_group: FrontEndGroup = hsm_group_backend.into();
      hsm_group_vec.push(hsm_group);
    }

    Ok(hsm_group_vec)
  }

  async fn delete_group(
    &self,
    auth_token: &str,
    label: &str,
  ) -> Result<Value, Error> {
    self
      .shasta_client(auth_token)?
      .hsm_group_delete_group(&label.to_string())
      .await
      .map_err(|e| Error::Message(e.to_string()))
  }

  async fn get_hsm_map_and_filter_by_hsm_name_vec(
    &self,
    shasta_token: &str,
    hsm_name_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    hsm::group::utils::get_hsm_map_and_filter_by_hsm_name_vec(
      shasta_token,
      &self.base_url,
      &self.root_cert,
      self.socks5_proxy.as_deref(),
      hsm_name_vec,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }

  async fn post_member(
    &self,
    auth_token: &str,
    group_label: &str,
    xname: &str,
  ) -> Result<Value, Error> {
    let member = Member {
      id: Some(xname.to_string()),
    };

    self
      .shasta_client(auth_token)?
      .hsm_group_post_member(group_label, member)
      .await
      .map_err(|e| Error::Message(e.to_string()))
  }

  async fn add_members_to_group(
    &self,
    auth_token: &str,
    group_label: &str,
    new_members: &[&str],
  ) -> Result<Vec<String>, Error> {
    let mut sol: Vec<String> = Vec::new();

    for new_member in new_members {
      sol = hsm::group::utils::add_member(
        auth_token,
        &self.base_url,
        &self.root_cert,
        self.socks5_proxy.as_deref(),
        group_label,
        new_member,
      )
      .await
      .map_err(|e| Error::Message(e.to_string()))?;
    }

    Ok(sol)
  }

  async fn delete_member_from_group(
    &self,
    auth_token: &str,
    group_label: &str,
    xname: &str,
  ) -> Result<(), Error> {
    self
      .shasta_client(auth_token)?
      .hsm_group_delete_member(group_label, xname)
      .await
      .map_err(|e| Error::Message(e.to_string()))
  }

  async fn update_group_members(
    &self,
    auth_token: &str,
    group_name: &str,
    members_to_remove: &[&str],
    members_to_add: &[&str],
  ) -> Result<(), Error> {
    hsm::group::utils::update_hsm_group_members(
      auth_token,
      &self.base_url,
      &self.root_cert,
      self.socks5_proxy.as_deref(),
      group_name,
      members_to_remove,
      members_to_add,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }

  // HSM/GROUP
  async fn migrate_group_members(
    &self,
    shasta_token: &str,
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    new_target_hsm_members: &[&str],
    dryrun: bool
  ) -> Result<(Vec<String>, Vec<String>), Error> {
    hsm::group::utils::migrate_hsm_members(
      shasta_token,
      &self.base_url,
      &self.root_cert,
      self.socks5_proxy.as_deref(),
      target_hsm_group_name,
      parent_hsm_group_name,
      new_target_hsm_members,
      dryrun,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }
}
