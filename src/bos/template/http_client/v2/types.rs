//! Wire-format types — mirror the upstream CSM OpenAPI schema; field names and
//! shapes are dictated by the API.
#![allow(missing_docs)]

use manta_backend_dispatcher::types::bos::session_template::{
  BootSet as FrontEndBootSet, BosSessionTemplate as FrontEndBosSessionTemplate,
  Cfs as FrontEndCfs, Link as FrontEndLink,
};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rel: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub href: Option<String>,
}

impl From<FrontEndLink> for Link {
  fn from(frontend_link: FrontEndLink) -> Self {
    Self {
      rel: frontend_link.rel,
      href: frontend_link.href,
    }
  }
}

impl From<Link> for FrontEndLink {
  fn from(val: Link) -> Self {
    FrontEndLink {
      rel: val.rel,
      href: val.href,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cfs {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub configuration: Option<String>,
}

impl From<FrontEndCfs> for Cfs {
  fn from(frontend_cfs: FrontEndCfs) -> Self {
    Self {
      configuration: frontend_cfs.configuration,
    }
  }
}

impl From<Cfs> for FrontEndCfs {
  fn from(val: Cfs) -> Self {
    FrontEndCfs {
      configuration: val.configuration,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BootSet {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub path: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cfs: Option<Cfs>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub etag: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub kernel_parameters: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_list: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_roles_groups: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub node_groups: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arch: Option<String>, // TODO: use Arch enum instead
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rootfs_provider: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rootfs_provider_passthrough: Option<String>,
}

impl From<FrontEndBootSet> for BootSet {
  fn from(frontend_boot_set: FrontEndBootSet) -> Self {
    Self {
      name: frontend_boot_set.name,
      path: frontend_boot_set.path,
      cfs: frontend_boot_set.cfs.map(|cfs| cfs.into()),
      r#type: frontend_boot_set.r#type,
      etag: frontend_boot_set.etag,
      kernel_parameters: frontend_boot_set.kernel_parameters,
      node_list: frontend_boot_set.node_list,
      node_roles_groups: frontend_boot_set.node_roles_groups,
      node_groups: frontend_boot_set.node_groups,
      arch: frontend_boot_set.arch,
      rootfs_provider: frontend_boot_set.rootfs_provider,
      rootfs_provider_passthrough: frontend_boot_set
        .rootfs_provider_passthrough,
    }
  }
}

impl From<BootSet> for FrontEndBootSet {
  fn from(val: BootSet) -> Self {
    FrontEndBootSet {
      name: val.name,
      path: val.path,
      cfs: val.cfs.map(|cfs| cfs.into()),
      r#type: val.r#type,
      etag: val.etag,
      kernel_parameters: val.kernel_parameters,
      node_list: val.node_list,
      node_roles_groups: val.node_roles_groups,
      node_groups: val.node_groups,
      arch: val.arch,
      rootfs_provider: val.rootfs_provider,
      rootfs_provider_passthrough: val.rootfs_provider_passthrough,
    }
  }
}

// TODO: use strum crate to implement functions to convert to/from String
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Arch {
  X86,
  ARM,
  Other,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BosSessionTemplate {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tenant: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub enable_cfs: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cfs: Option<Cfs>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub boot_sets: Option<HashMap<String, BootSet>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub links: Option<Vec<Link>>,
}

impl From<FrontEndBosSessionTemplate> for BosSessionTemplate {
  fn from(frontend_bos_session_template: FrontEndBosSessionTemplate) -> Self {
    Self {
      name: frontend_bos_session_template.name,
      tenant: frontend_bos_session_template.tenant,
      description: frontend_bos_session_template.description,
      enable_cfs: frontend_bos_session_template.enable_cfs,
      cfs: frontend_bos_session_template.cfs.map(|cfs| cfs.into()),
      boot_sets: frontend_bos_session_template.boot_sets.map(|boot_sets| {
        boot_sets.into_iter().map(|(k, v)| (k, v.into())).collect()
      }),
      links: frontend_bos_session_template
        .links
        .map(|links| links.into_iter().map(|link| link.into()).collect()),
    }
  }
}

impl From<BosSessionTemplate> for FrontEndBosSessionTemplate {
  fn from(val: BosSessionTemplate) -> Self {
    FrontEndBosSessionTemplate {
      name: val.name,
      tenant: val.tenant,
      description: val.description,
      enable_cfs: val.enable_cfs,
      cfs: val.cfs.map(|cfs| cfs.into()),
      boot_sets: val.boot_sets.map(|boot_sets| {
        boot_sets.into_iter().map(|(k, v)| (k, v.into())).collect()
      }),
      links: val
        .links
        .map(|links| links.into_iter().map(|link| link.into()).collect()),
    }
  }
}

impl BosSessionTemplate {
  pub fn configuration_name(&self) -> Option<&str> {
    self
      .cfs
      .as_ref()
      .and_then(|cfs| cfs.configuration.as_deref())
  }

  pub fn get_target(&self) -> Vec<String> {
    let target_hsm = self.get_target_hsm();
    let target_xname = self.get_target_xname();

    [target_hsm, target_xname].concat()
  }

  /// Returns HSM group names related to the BOS sessiontemplate
  pub fn get_target_hsm(&self) -> Vec<String> {
    self
      .boot_sets
      .as_ref()
      .map(|boot_sets| {
        boot_sets
          .iter()
          .flat_map(|(_, boot_param)| {
            boot_param.node_groups.clone().unwrap_or_default()
          })
          .collect()
      })
      .unwrap_or_default()
  }

  pub fn get_target_xname(&self) -> Vec<String> {
    self
      .boot_sets
      .as_ref()
      .map(|boot_set| {
        boot_set
          .iter()
          .flat_map(|(_, boot_param)| {
            boot_param.node_list.clone().unwrap_or_default()
          })
          .collect()
      })
      .unwrap_or_default()
  }

  pub fn get_configuration(&self) -> Option<&str> {
    self
      .cfs
      .as_ref()
      .and_then(|cfs| cfs.configuration.as_deref())
  }

  pub fn get_path_vec(&self) -> Vec<String> {
    self
      .boot_sets
      .as_ref()
      .map(|boot_set| {
        boot_set
          .values()
          .map(|boot_param| boot_param.path.clone().unwrap_or_default())
          .collect()
      })
      .unwrap_or_default()
  }

  /// Returns all images path related to this BOS sessiontemplate
  pub fn images_path(&self) -> impl Iterator<Item = &str> {
    self
      .boot_sets
      .iter()
      .flatten()
      .filter_map(|(_, boot_param)| boot_param.path.as_deref())
  }

  /// Returns all images id related to this BOS sessiontemplate
  pub fn images_id(&self) -> impl Iterator<Item = &str> {
    self.images_path().map(|path| {
      path
        .trim_start_matches("s3://boot-images/")
        .trim_end_matches("/manifest.json")
    })
  }

  #[allow(clippy::too_many_arguments)]
  pub fn new_for_hsm_group(
    tenant_opt: Option<String>,
    cfs_configuration_name: String,
    bos_session_template_name: String,
    ims_image_name: String,
    ims_image_path: String,
    ims_image_type: String,
    ims_image_etag: String,
    hsm_group: String,
    kernel_params: String,
    arch_opt: Option<String>,
  ) -> Self {
    let cfs = Cfs {
      configuration: Some(cfs_configuration_name),
    };

    let boot_set = BootSet {
      name: Some(ims_image_name),
      path: Some(ims_image_path),
      r#type: Some(ims_image_type.clone()),
      etag: Some(ims_image_etag),
      kernel_parameters: Some(kernel_params),
      node_list: None,
      node_roles_groups: None,
      node_groups: Some(vec![hsm_group]),
      rootfs_provider: None,
      rootfs_provider_passthrough: None,
      cfs: Some(cfs.clone()),
      arch: arch_opt,
    };

    let mut boot_set_map = HashMap::<String, BootSet>::new();

    boot_set_map.insert(ims_image_type, boot_set);

    BosSessionTemplate {
      name: Some(bos_session_template_name),
      description: None,
      enable_cfs: Some(true),
      cfs: Some(cfs),
      boot_sets: Some(boot_set_map),
      links: None,
      tenant: tenant_opt,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn bos_template_with_boot_sets(
    boot_sets: HashMap<String, BootSet>,
  ) -> BosSessionTemplate {
    BosSessionTemplate {
      name: Some("test-template".to_string()),
      description: None,
      enable_cfs: None,
      cfs: None,
      boot_sets: Some(boot_sets),
      links: None,
      tenant: None,
    }
  }

  fn boot_set_with_path(path: Option<&str>) -> BootSet {
    BootSet {
      name: None,
      path: path.map(str::to_string),
      r#type: None,
      etag: None,
      kernel_parameters: None,
      cfs: None,
      node_list: None,
      node_roles_groups: None,
      node_groups: None,
      rootfs_provider: None,
      rootfs_provider_passthrough: None,
      arch: None,
    }
  }

  #[test]
  fn images_path_returns_empty_when_no_boot_sets() {
    let template = BosSessionTemplate {
      name: None,
      description: None,
      enable_cfs: None,
      cfs: None,
      boot_sets: None,
      links: None,
      tenant: None,
    };
    assert_eq!(template.images_path().count(), 0);
  }

  #[test]
  fn images_path_skips_boot_sets_with_no_path() {
    let mut boot_sets = HashMap::new();
    boot_sets.insert("compute".into(), boot_set_with_path(None));
    boot_sets.insert(
      "uan".into(),
      boot_set_with_path(Some("s3://boot-images/abc/manifest.json")),
    );

    let template = bos_template_with_boot_sets(boot_sets);
    let paths: Vec<&str> = template.images_path().collect();
    assert_eq!(paths, vec!["s3://boot-images/abc/manifest.json"]);
  }

  #[test]
  fn images_id_strips_s3_prefix_and_manifest_suffix() {
    let mut boot_sets = HashMap::new();
    boot_sets.insert(
      "compute".into(),
      boot_set_with_path(Some(
        "s3://boot-images/59e0180a-3fdd-4936-bba7-14ba914ffd34/manifest.json",
      )),
    );

    let template = bos_template_with_boot_sets(boot_sets);
    let ids: Vec<&str> = template.images_id().collect();
    assert_eq!(ids, vec!["59e0180a-3fdd-4936-bba7-14ba914ffd34"]);
  }

  #[test]
  fn images_id_passes_through_path_without_known_affixes() {
    // Behavior contract: trim_*_matches only removes when present;
    // foreign paths come through unchanged.
    let mut boot_sets = HashMap::new();
    boot_sets.insert(
      "compute".into(),
      boot_set_with_path(Some("https://example.com/blob")),
    );

    let template = bos_template_with_boot_sets(boot_sets);
    let ids: Vec<&str> = template.images_id().collect();
    assert_eq!(ids, vec!["https://example.com/blob"]);
  }
}
