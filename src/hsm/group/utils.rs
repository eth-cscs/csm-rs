use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::{
  error::Error,
  hsm::{self, group::types::Group},
  node::utils::validate_xnames_format_and_membership_agaisnt_single_hsm,
};

use super::{
  http_client::{self, delete_member, post_member},
  types::Member,
};

pub async fn get_group_available(
  shasta_auth_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
) -> Result<Vec<Group>, Error> {
  const ADMIN_ROLE_NAME: &str = "pa_admin";

  let mut group_vec = hsm::group::http_client::get_all(
    shasta_auth_token,
    shasta_base_url,
    shasta_root_cert,
  )
  .await
  .map_err(|e| Error::Message(e.to_string()))?;

  // Get HSM groups/Keycloak roles the user has access to from JWT token
  let realm_access_role_vec =
    crate::common::jwt_ops::get_roles(shasta_auth_token)?;

  if !realm_access_role_vec.contains(&ADMIN_ROLE_NAME.to_string()) {
    let available_groups_name = get_group_name_available(
      shasta_auth_token,
      shasta_base_url,
      shasta_root_cert,
    )
    .await?;

    group_vec.retain(|group| available_groups_name.contains(&group.label));

    // Remove site wide HSM groups like 'alps', 'prealps', 'alpsm', etc because they pollute
    // the roles to check if a user has access to individual compute nodes
    //FIXME: Get rid of this by making sure CSM admins don't create HSM groups for system
    //wide operations instead of using roles
    let realm_access_role_filtered_vec =
      hsm::group::hacks::filter_system_hsm_groups(group_vec.clone());

    Ok(realm_access_role_filtered_vec)
  } else {
    Ok(group_vec)
  }
}

pub async fn get_group_name_available(
  shasta_auth_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
) -> Result<Vec<String>, Error> {
  log::debug!("Get HSM names available from JWT or all");

  const ADMIN_ROLE_NAME: &str = "pa_admin";

  // Get HSM groups/Keycloak roles the user has access to from JWT token
  let realm_access_role_vec =
    crate::common::jwt_ops::get_roles(shasta_auth_token)?;

  if !realm_access_role_vec.contains(&ADMIN_ROLE_NAME.to_string()) {
    log::debug!("User is not admin, getting HSM groups available from JWT");

    // remove keycloak roles not related with HSM groups
    /* realm_access_role_vec
    .retain(|role| !role.eq("offline_access") && !role.eq("uma_authorization")); */
    let realm_access_role_vec = hsm::group::hacks::filter_keycloak_roles(
      realm_access_role_vec
        .iter()
        .map(String::as_str)
        .collect::<Vec<&str>>()
        .as_slice(),
    );

    // Remove site wide HSM groups like 'alps', 'prealps', 'alpsm', etc because they pollute
    // the roles to check if a user has access to individual compute nodes
    //FIXME: Get rid of this by making sure CSM admins don't create HSM groups for system
    //wide operations instead of using roles
    let mut realm_access_role_filtered_vec =
      hsm::group::hacks::filter_system_hsm_group_names(
        realm_access_role_vec.clone(),
      );

    realm_access_role_filtered_vec.sort();

    Ok(realm_access_role_vec)
  } else {
    log::debug!("User is admin, getting all HSM groups in the system");
    let all_hsm_groups_rslt = hsm::group::http_client::get_all(
      shasta_auth_token,
      shasta_base_url,
      shasta_root_cert,
    )
    .await;

    let mut all_hsm_groups = all_hsm_groups_rslt?
      .iter()
      .map(|hsm_value| hsm_value.label.clone())
      .collect::<Vec<String>>();

    all_hsm_groups.sort();

    Ok(all_hsm_groups)
  }
}

/// Add a list of xnames to target HSM group
/// Returns the new list of nodes in target HSM group
pub async fn add_member(
  auth_token: &str,
  base_url: &str,
  root_cert: &[u8],
  group_label: &str,
  new_member: &str,
) -> Result<Vec<String>, Error> {
  // Get HSM group from CSM
  let group_vec = hsm::group::http_client::get(
    auth_token,
    base_url,
    root_cert,
    Some(&[&group_label]),
    None,
  )
  .await?;

  // Check if HSM group found
  if let Some(group) = group_vec.first().cloned().as_mut() {
    // Update HSM group with new memebers
    // Create Member struct
    let new_member = new_member.to_string();
    let member = crate::hsm::group::types::Member {
      id: Some(new_member.clone()),
    };

    // Update HSM group in CSM
    let _ = crate::hsm::group::http_client::post_member(
      auth_token,
      base_url,
      root_cert,
      group_label,
      member,
    )
    .await?;

    // Generate list of updated group members
    group.get_members().push(new_member);

    Ok(group.get_members())
  } else {
    // HSM group not found, throw an error
    Err(Error::Message(format!(
      "No HSM group '{}' found",
      group_label
    )))
  }
}

/// Removes list of xnames from  HSM group
pub async fn remove_hsm_members(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  target_hsm_group_name: &str,
  new_target_hsm_members: Vec<&str>,
  dryrun: bool,
) -> Result<Vec<String>, Error> {
  // Check nodes are valid xnames and they belong to parent HSM group
  if let Ok(false) = validate_xnames_format_and_membership_agaisnt_single_hsm(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    new_target_hsm_members.as_slice(),
    Some(&target_hsm_group_name.to_string()),
  )
  .await
  {
    let error_msg =
      format!("Nodes '{}' not valid", new_target_hsm_members.join(", "));
    return Err(Error::Message(error_msg));
  }

  // get list of parent HSM group members
  let mut target_hsm_group_member_vec: Vec<String> =
    get_member_vec_from_hsm_group_name(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      target_hsm_group_name,
    )
    .await?;

  target_hsm_group_member_vec.retain(|parent_member| {
    !new_target_hsm_members.contains(&parent_member.as_str())
  });

  target_hsm_group_member_vec.sort();
  target_hsm_group_member_vec.dedup();

  // *********************************************************************************************************
  // UPDATE HSM GROUP MEMBERS IN CSM
  if dryrun {
    println!(
      "Remove following nodes from HSM group {}:\n{:?}",
      target_hsm_group_name, new_target_hsm_members
    );

    println!("dry-run enabled, changes not persisted.");
  } else {
    for xname in new_target_hsm_members {
      let _ = delete_member(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        target_hsm_group_name,
        xname,
      )
      .await;
    }
  }

  Ok(target_hsm_group_member_vec)
}

/// Moves list of xnames from parent to target HSM group
pub async fn migrate_hsm_members(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  target_hsm_group_name: &str,
  parent_hsm_group_name: &str,
  new_target_hsm_members: &[&str],
  nodryrun: bool,
) -> Result<(Vec<String>, Vec<String>), Error> {
  // Check nodes are valid xnames and they belong to parent HSM group
  if let Ok(false) = validate_xnames_format_and_membership_agaisnt_single_hsm(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    new_target_hsm_members,
    Some(&parent_hsm_group_name.to_string()),
  )
  .await
  {
    let error_msg =
      format!("Nodes '{}' not valid", new_target_hsm_members.join(", "));
    return Err(Error::Message(error_msg));
  }

  // get list of target HSM group members
  let mut target_hsm_group_member_vec: Vec<String> =
    get_member_vec_from_hsm_group_name(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      target_hsm_group_name,
    )
    .await?;

  // merge HSM group list with the list of xnames provided by the user
  target_hsm_group_member_vec
    .extend(new_target_hsm_members.iter().cloned().map(str::to_string));

  target_hsm_group_member_vec.sort();
  target_hsm_group_member_vec.dedup();

  // get list of parent HSM group members
  let mut parent_hsm_group_member_vec: Vec<String> =
    get_member_vec_from_hsm_group_name(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      parent_hsm_group_name,
    )
    .await?;

  parent_hsm_group_member_vec.retain(|parent_member| {
    !target_hsm_group_member_vec.contains(parent_member)
  });

  parent_hsm_group_member_vec.sort();
  parent_hsm_group_member_vec.dedup();

  // *********************************************************************************************************
  // UPDATE HSM GROUP MEMBERS IN CSM
  if !nodryrun {
    let target_hsm_group = serde_json::json!({
        "label": target_hsm_group_name,
        "decription": "",
        "members": target_hsm_group_member_vec,
        "tags": []
    });

    println!(
      "Target HSM group:\n{}",
      serde_json::to_string_pretty(&target_hsm_group)?
    );

    let parent_hsm_group = serde_json::json!({
        "label": parent_hsm_group_name,
        "decription": "",
        "members": parent_hsm_group_member_vec,
        "tags": []
    });

    println!(
      "Parent HSM group:\n{}",
      serde_json::to_string_pretty(&parent_hsm_group)?
    );

    println!("dry-run enabled, changes not persisted.");
  } else {
    for xname in new_target_hsm_members {
      let member = Member {
        id: Some(xname.to_string()),
      };

      let _ = post_member(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        target_hsm_group_name,
        member,
      )
      .await;

      let _ = delete_member(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        parent_hsm_group_name,
        xname,
      )
      .await;
    }
  }

  Ok((target_hsm_group_member_vec, parent_hsm_group_member_vec))
}

/// Receives 2 lists of xnames old xnames to remove from parent HSM group and new xhanges to add to target HSM group, and does just that
pub async fn update_hsm_group_members(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hsm_group_name: &str,
  old_target_hsm_group_members: &[&str],
  new_target_hsm_group_members: &[&str],
) -> Result<(), Error> {
  // Delete members
  for old_member in old_target_hsm_group_members {
    if !new_target_hsm_group_members.contains(old_member) {
      let _ = delete_member(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_name,
        old_member,
      )
      .await;
    }
  }

  // Add members
  for new_member in new_target_hsm_group_members {
    if !old_target_hsm_group_members.contains(new_member) {
      let member = Member {
        id: Some(new_member.to_string()),
      };

      let _ = post_member(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_name,
        member,
      )
      .await;
    }
  }

  Ok(())
}

// Returns a HashMap with keys being the xnames and values the list of group names each xname
// belongs to
pub async fn get_xname_map_and_filter_by_xname_vec(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  xname_vec: Vec<&str>,
) -> Result<HashMap<String, Vec<String>>, Error> {
  let hsm_group_vec =
    http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
      .await?;

  let mut xname_map: HashMap<String, Vec<String>> = HashMap::new();

  for hsm_group in hsm_group_vec {
    for xname in hsm_group.get_members() {
      if xname_vec.contains(&xname.as_str()) {
        xname_map
          .entry(xname)
          .and_modify(|group_vec| group_vec.push(hsm_group.label.clone()))
          .or_insert(vec![hsm_group.label.clone()]);
      }
    }
  }

  Ok(xname_map)
}

// Returns a HashMap with keys being the hsm names/labels the user has access a curated list of xnames
// for each hsm name as values
pub async fn get_hsm_map_and_filter_by_hsm_name_vec(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hsm_name_vec: &[&str],
) -> Result<HashMap<String, Vec<String>>, Error> {
  let hsm_group_vec =
    http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
      .await?;

  Ok(filter_by_hsm_group_name_and_convert_to_map(
    hsm_name_vec,
    &hsm_group_vec.iter().collect::<Vec<&Group>>(),
  ))
}

// Returns a HashMap with keys being the hsm names/labels the user has access a curated list of xnames
// for each hsm name as values
pub async fn get_hsm_group_map_and_filter_by_hsm_group_member_vec(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  member_vec: &[&str],
) -> Result<HashMap<String, Vec<String>>, Error> {
  let hsm_group_vec =
    http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
      .await?;

  Ok(filter_by_hsm_group_members_and_convert_to_map(
    member_vec,
    hsm_group_vec,
  ))
}

/// Given a list of HsmGroup struct and a list of Hsm group names, it will filter out those
/// not in the Hsm group names and convert from HsmGroup struct to HashMap
pub fn filter_by_hsm_group_name_and_convert_to_map(
  hsm_name_vec: &[&str],
  hsm_group_vec: &[&Group],
) -> HashMap<String, Vec<String>> {
  let mut hsm_group_map: HashMap<String, Vec<String>> = HashMap::new();

  for hsm_group in hsm_group_vec {
    if hsm_name_vec.contains(&hsm_group.label.as_str()) {
      hsm_group_map.entry(hsm_group.label.clone()).or_insert(
        hsm_group
          .members
          .clone()
          .and_then(|members| members.ids)
          .unwrap(),
      );
    }
  }

  hsm_group_map
}

/// Given a list of HsmGroup struct and a list of Hsm group members, it will filter out those
/// not in the Hsm group names and convert from HsmGroup struct to HashMap
pub fn filter_by_hsm_group_members_and_convert_to_map(
  member_vec: &[&str],
  hsm_group_vec: Vec<Group>,
) -> HashMap<String, Vec<String>> {
  let mut hsm_group_map: HashMap<String, Vec<String>> = HashMap::new();

  for hsm_group in hsm_group_vec {
    if hsm_group
      .get_members()
      .iter()
      .any(|member| member_vec.contains(&member.as_str()))
    {
      hsm_group_map
        .entry(hsm_group.label)
        .or_insert(hsm_group.members.and_then(|members| members.ids).unwrap());
    }
  }

  hsm_group_map
}

pub fn get_member_vec_from_hsm_group_value(hsm_group: &Value) -> Vec<String> {
  // Take all nodes for all hsm_groups found and put them in a Vec
  hsm_group
    .pointer("/members/ids")
    .and_then(Value::as_array)
    .and_then(|member_vec| {
      member_vec
        .iter()
        .map(|xname| xname.as_str().map(str::to_string))
        .collect()
    })
    .unwrap_or_default()
}

pub fn get_member_vec_from_hsm_group(hsm_group: &Group) -> Vec<String> {
  // Take all nodes for all hsm_groups found and put them in a Vec
  hsm_group.get_members()
}

/// Get the list of xnames which are members of a list of HSM groups.
/// eg:
/// given following HSM groups:
/// tenant_a: [x1003c1s7b0n0, x1003c1s7b0n1]
/// tenant_b: [x1003c1s7b1n0]
/// Then calling this function with hsm_name_vec: &["tenant_a", "tenant_b"] should return [x1003c1s7b0n0, x1003c1s7b0n1, x1003c1s7b1n0]
pub async fn get_member_vec_from_hsm_name_vec(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hsm_name_vec: &[&str],
) -> Result<Vec<String>, Error> {
  log::info!("Get xnames from HSM groups");
  log::debug!("Get xnames from HSM groups: {:?}", hsm_name_vec);

  let hsm_group_vec = hsm::group::http_client::get(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    Some(
      hsm_name_vec
        .iter()
        .map(|c| &**c)
        .collect::<Vec<&str>>()
        .as_slice(),
    ),
    None,
  )
  .await?;

  let mut hsm_group_member_vec: Vec<String> = Vec::new();

  for hsm_group in hsm_group_vec {
    hsm_group_member_vec.append(&mut hsm_group.get_members());
  }

  Ok(hsm_group_member_vec)
}

pub fn get_member_vec_from_hsm_group_value_vec(
  hsm_groups: &[Value],
) -> HashSet<String> {
  hsm_groups
    .iter()
    .flat_map(get_member_vec_from_hsm_group_value)
    .collect()
}

pub fn get_member_vec_from_hsm_group_vec(
  hsm_groups: &[Group],
) -> HashSet<String> {
  hsm_groups
    .iter()
    .flat_map(get_member_vec_from_hsm_group)
    .collect()
}

/// Returns a Map with nodes and the list of hsm groups that node belongs to.
/// eg "x1500b5c1n3 --> [ psi-dev, psi-dev_cn ]"
pub fn group_members_by_hsm_group_from_hsm_groups_value(
  hsm_groups: &Vec<Value>,
) -> HashMap<String, Vec<String>> {
  let mut member_hsm_map: HashMap<String, Vec<String>> = HashMap::new();
  for hsm_group_value in hsm_groups {
    let hsm_group_name = hsm_group_value
      .get("label")
      .and_then(Value::as_str)
      .map(str::to_string)
      .unwrap();
    for member in get_member_vec_from_hsm_group_value(hsm_group_value) {
      member_hsm_map
        .entry(member)
        .and_modify(|hsm_groups| hsm_groups.push(hsm_group_name.clone()))
        .or_insert_with(|| vec![hsm_group_name.clone()]);
    }
  }

  member_hsm_map
}

pub async fn get_member_vec_from_hsm_group_name(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hsm_group: &str,
) -> Result<Vec<String>, Error> {
  // Take all nodes for all hsm_groups found and put them in a Vec
  Ok(
    http_client::get_one(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group,
    )
    .await?
    .get_members(),
  )
}
