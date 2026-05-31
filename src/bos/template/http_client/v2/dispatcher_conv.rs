//! Bidirectional `From` impls between csm-rs's BOS v2 session-template
//! types and the dispatcher's mirrors. Gated behind the
//! `manta-dispatcher` Cargo feature so users not on Manta don't pull the
//! dispatcher dep.

use manta_backend_dispatcher::types::bos::session_template::{
  BootSet as FrontEndBootSet, BosSessionTemplate as FrontEndBosSessionTemplate,
  Cfs as FrontEndCfs, Link as FrontEndLink,
};

use super::types::{BootSet, BosSessionTemplate, Cfs, Link};

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
