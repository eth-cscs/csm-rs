//! Bidirectional `From` impls between csm-rs's HSM group types and the
//! dispatcher's mirrors. Gated behind the `manta-dispatcher` Cargo
//! feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::{
  Group as FrontEndGroup, Member as FrontEndMember,
};

use super::types::{Group, Members};

impl From<FrontEndGroup> for Group {
  fn from(value: FrontEndGroup) -> Self {
    let mut member_vec = Vec::new();
    let member_vec_backend = value.get_members();

    for member in member_vec_backend {
      member_vec.push(member);
    }

    let members = Members {
      ids: Some(member_vec),
    };

    Group {
      label: value.label,
      description: value.description,
      tags: value.tags,
      members: Some(members),
      exclusive_group: value.exclusive_group,
    }
  }
}

impl From<Group> for FrontEndGroup {
  fn from(val: Group) -> Self {
    let mut member_vec = Vec::new();
    let member_vec_backend = val.get_members();

    for member in member_vec_backend {
      member_vec.push(member);
    }

    let members = FrontEndMember {
      ids: Some(member_vec),
    };

    FrontEndGroup {
      label: val.label,
      description: val.description,
      tags: val.tags,
      members: Some(members),
      exclusive_group: val.exclusive_group,
    }
  }
}
