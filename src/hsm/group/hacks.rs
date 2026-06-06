//! Workarounds for CSM HSM behaviour that does not fit cleanly into
//! the rest of the surface.
//!
//! Many helpers here filter out "system-wide" HSM group labels
//! (`alps`, `prealps`, …) from access-control derivations. Long-term
//! the fix is operational, not in this code: CSM admins should stop
//! using HSM groups for system-wide scoping and use Keycloak roles
//! instead. Until that happens these filters keep the per-user
//! visible-groups list honest.

use crate::{common, error::Error, hsm};

use super::types::Group;

/// Keycloak role name that grants full admin access (bypasses HSM-group
/// scoping checks).
pub static PA_ADMIN: &str = "pa_admin";
/// HSM group labels treated as site-wide buckets — pruned from the
/// per-user visible-groups list so they don't pollute access control.
pub static SYSTEM_WIDE_HSM_GROUPS: [&str; 4] =
  ["alps", "prealps", "alpse", "alpsb"];
/// Keycloak realm roles that are infrastructural rather than HSM-group
/// names; stripped before resolving "groups visible to this user."
pub static KEYCLOAK_ROLES_TO_IGNORE: [&str; 3] = [
  "offline_access",
  "uma_authorization",
  "default-roles-shasta",
];
/// Canonical HSM component `Role` values (used to filter roles supplied
/// by the user against the closed set CSM accepts).
pub static ROLES: [&str; 6] = [
  "Compute",
  "Service",
  "System",
  "Application",
  "Storage",
  "Management",
];
/// Canonical HSM component `SubRole` values.
pub static SUBROLES: [&str; 8] = [
  "Worker",
  "Master",
  "Storage",
  "UAN",
  "Gateway",
  "LNETRouter",
  "Visualization",
  "UserDefined",
];

/// Removes 'system wide' HSM groups from the provided HSM group vector.
/// See the module-level note on why this filter exists.
#[must_use]
pub fn filter_system_hsm_groups(hsm_group_vec: Vec<Group>) -> Vec<Group> {
  hsm_group_vec
    .iter()
    .filter(|hsm_group| {
      let label = hsm_group.label.as_str();
      !SYSTEM_WIDE_HSM_GROUPS.contains(&label)
    })
    .cloned()
    .collect::<Vec<Group>>()
}

/// Removes unwanted roles thay may appear in keycloak auth/jwt token roles
pub fn filter_keycloak_roles(keycloak_roles: &[&str]) -> Vec<String> {
  keycloak_roles
    .iter()
    .filter(|role| !KEYCLOAK_ROLES_TO_IGNORE.contains(role))
    .copied()
    .map(str::to_string)
    .collect()
}

/// Removes 'system wide' group names. See the module-level note on
/// why this filter exists.
#[must_use]
pub fn filter_system_hsm_group_names(
  hsm_group_name_vec: Vec<String>,
) -> Vec<String> {
  hsm_group_name_vec
    .into_iter()
    .filter(|hsm_group_name| {
      !SYSTEM_WIDE_HSM_GROUPS.contains(&hsm_group_name.as_str())
    })
    .collect()
}

/// Removes 'roles' and 'subroles' from the provided HSM group name vector
pub fn filter_roles_and_subroles(hsm_group_name_vec: &[&str]) -> Vec<String> {
  hsm_group_name_vec
    .iter()
    .filter(|hsm_group_name| {
      !ROLES.contains(hsm_group_name) && !SUBROLES.contains(hsm_group_name)
    })
    .copied()
    .map(str::to_string)
    .collect()
}

/// Check user has access to all groups in CFS session
/// This function validates groups in CFS session against user auth token
/// Returns the list of groups in the CFS session the user does not have access to
///
/// # Errors
///
/// Returns an [`Error`] variant on CSM, transport, or
/// deserialization failure; see the crate-level `Error` enum
/// for the full set.
pub fn validate_groups_auth_token(
  cfs_group_names: &[&str],
  shasta_token: &str,
) -> Result<Vec<String>, Error> {
  let keycloak_roles = common::jwt_ops::get_roles(shasta_token)?;

  Ok(validate_groups(
    cfs_group_names,
    &keycloak_roles
      .iter()
      .map(String::as_str)
      .collect::<Vec<&str>>(),
  ))
}

/// Check user has access to all groups in CFS session
/// This function validates groups in CFS session against a list of groups the user supposedly has
/// access to
/// Returns the list of groups in the CFS session the user does not have access to
pub fn validate_groups(
  cfs_group_names: &[&str],
  keycloak_roles: &[&str],
) -> Vec<String> {
  if keycloak_roles.contains(&PA_ADMIN) {
    // Admins have access to all groups
    vec![]
  } else {
    // User is not admin. Check if groups in CFS session are in user auth token
    // Remove unwanted roles from keycloak auth token
    let groups_and_roles_in_auth_token =
      hsm::group::hacks::filter_keycloak_roles(keycloak_roles);
    // Remove "roles" and "subroles" from auth token
    let site_wide_and_cluster_groups_in_auth_token =
      hsm::group::hacks::filter_roles_and_subroles(
        &groups_and_roles_in_auth_token
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>(),
      );
    // Remove "site wide" (eg: alps, realps, alpsm, alpsb, etc.) from CFS session groups
    let groups_in_user_auth_token =
      filter_system_hsm_group_names(site_wide_and_cluster_groups_in_auth_token);

    // Remove 'roles' and 'subroles' from CFS session groups
    let groups_without_roles_subroles =
      hsm::group::hacks::filter_roles_and_subroles(cfs_group_names);
    // Remove 'system wide' groups from CFS session groups
    let groups_without_system_wide =
      hsm::group::hacks::filter_system_hsm_group_names(
        groups_without_roles_subroles.clone(),
      );
    // Get list of groups in CFS session not in user auth token
    groups_without_system_wide
      .into_iter()
      .filter(|group| !groups_in_user_auth_token.contains(group))
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn group_with_label(label: &str) -> Group {
    Group {
      label: label.to_string(),
      description: None,
      tags: None,
      exclusive_group: None,
      members: None,
    }
  }

  // ---------- filter_system_hsm_groups ----------

  #[test]
  fn filter_system_hsm_groups_removes_known_system_labels() {
    let input = vec![
      group_with_label("alps"),
      group_with_label("prealps"),
      group_with_label("alpse"),
      group_with_label("alpsb"),
      group_with_label("user-group"),
    ];
    let out = filter_system_hsm_groups(input);
    let labels: Vec<&str> = out.iter().map(|g| g.label.as_str()).collect();
    assert_eq!(labels, vec!["user-group"]);
  }

  #[test]
  fn filter_system_hsm_groups_preserves_user_groups() {
    let input = vec![
      group_with_label("zinal"),
      group_with_label("muri"),
      group_with_label("daint"),
    ];
    let out = filter_system_hsm_groups(input.clone());
    assert_eq!(out.len(), 3);
  }

  #[test]
  fn filter_system_hsm_groups_empty_input_empty_output() {
    assert!(filter_system_hsm_groups(vec![]).is_empty());
  }

  // ---------- filter_keycloak_roles ----------

  #[test]
  fn filter_keycloak_roles_strips_keycloak_internal_roles() {
    let roles = vec![
      "offline_access",
      "uma_authorization",
      "default-roles-shasta",
      "zinal",
      "Compute",
    ];
    let out = filter_keycloak_roles(&roles);
    assert_eq!(out, vec!["zinal".to_string(), "Compute".to_string()]);
  }

  #[test]
  fn filter_keycloak_roles_preserves_unknown_roles() {
    let roles = vec!["my-role", "another-role"];
    let out = filter_keycloak_roles(&roles);
    assert_eq!(out.len(), 2);
  }

  #[test]
  fn filter_keycloak_roles_empty_input_empty_output() {
    assert!(filter_keycloak_roles(&[]).is_empty());
  }

  // ---------- filter_system_hsm_group_names ----------

  #[test]
  fn filter_system_hsm_group_names_removes_known_system_labels() {
    let input = vec![
      "alps".to_string(),
      "prealps".to_string(),
      "user-group".to_string(),
    ];
    let out = filter_system_hsm_group_names(input);
    assert_eq!(out, vec!["user-group".to_string()]);
  }

  // ---------- filter_roles_and_subroles ----------

  #[test]
  fn filter_roles_and_subroles_removes_roles_and_subroles() {
    // ROLES = ["Compute", "Service", "System", "Application", "Storage", "Management"]
    // SUBROLES = ["Worker", "Master", "Storage", "UAN", "Gateway", "LNETRouter", "Visualization", "UserDefined"]
    let input = vec![
      "Compute",    // role
      "Worker",     // subrole
      "Storage",    // both role and subrole
      "zinal",      // genuine group
      "my-cluster", // genuine group
    ];
    let out = filter_roles_and_subroles(&input);
    assert_eq!(out, vec!["zinal".to_string(), "my-cluster".to_string()]);
  }

  #[test]
  fn filter_roles_and_subroles_empty_input_empty_output() {
    assert!(filter_roles_and_subroles(&[]).is_empty());
  }

  // ---------- validate_groups ----------
  //
  // Existing tests in hsm/group/tests.rs cover the admin and tenant happy
  // paths. Adding edge cases here.

  #[test]
  fn validate_groups_empty_cfs_groups_returns_empty() {
    let groups_user_has =
      vec!["zinal", "offline_access", "default-roles-shasta"];
    let out = validate_groups(&[], &groups_user_has);
    assert!(out.is_empty());
  }

  #[test]
  fn validate_groups_admin_passes_unknown_groups() {
    // Admins bypass all checks.
    let cfs_groups = vec!["unknown-group", "another-unknown"];
    let auth = vec![PA_ADMIN];
    assert!(validate_groups(&cfs_groups, &auth).is_empty());
  }

  #[test]
  fn validate_groups_strips_roles_subroles_from_cfs_list() {
    // CFS groups that are pure roles/subroles never need authorization.
    let cfs_groups = vec!["Compute", "Worker"];
    let auth: Vec<&str> = vec![]; // empty auth, not admin
    assert!(validate_groups(&cfs_groups, &auth).is_empty());
  }

  #[test]
  fn validate_groups_strips_system_wide_from_cfs_list() {
    let cfs_groups = vec!["alps", "alpsb"];
    let auth: Vec<&str> = vec![];
    assert!(validate_groups(&cfs_groups, &auth).is_empty());
  }
}
