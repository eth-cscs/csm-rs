//! Bidirectional `From` impls between csm-rs's HSM group types and the
//! dispatcher's mirrors. Gated behind the `manta-dispatcher` Cargo
//! feature so users not on Manta don't pull the dispatcher dep.
//!
//! Field-shape gymnastics post-progenitor migration:
//! - `Group.label` is `ResourceName(pub String)` here, plain `String`
//!   in the dispatcher. Trivial `.0` / `ResourceName(...)` round-trip.
//! - `Group.tags` is `Vec<ResourceName>` here, `Option<Vec<String>>`
//!   in the dispatcher. Empty vec maps to `None` (the historical
//!   serialisation behaviour: an absent `tags` array deserialises as
//!   `vec![]`, and an empty vec is `skip_serializing_if = "Vec::is_empty"`
//!   on the way back out).
//! - `Members.ids` is `Vec<XNameRw100>` here, `Option<Vec<String>>`
//!   in the dispatcher. Same empty-vec-as-None convention applies.

use manta_backend_dispatcher::types::{
  Group as FrontEndGroup, Member as FrontEndMember,
};

use super::types::{Group, Members, ResourceName, XNameRw100};

impl From<FrontEndGroup> for Group {
  fn from(value: FrontEndGroup) -> Self {
    let mut member_vec = Vec::new();
    let member_vec_backend = value.get_members();

    for member in member_vec_backend {
      member_vec.push(XNameRw100(member));
    }

    let members = Members { ids: member_vec };

    Group {
      label: ResourceName(value.label),
      description: value.description,
      tags: value
        .tags
        .unwrap_or_default()
        .into_iter()
        .map(ResourceName)
        .collect(),
      members: Some(members),
      exclusive_group: value.exclusive_group.map(ResourceName),
    }
  }
}

impl From<Group> for FrontEndGroup {
  fn from(val: Group) -> Self {
    // Use the trait-provided helper for `Vec<String>` extraction.
    use super::ext::GroupExt;

    let member_vec_backend = val.get_members();
    let mut member_vec = Vec::new();
    for member in member_vec_backend {
      member_vec.push(member);
    }

    let members = FrontEndMember {
      ids: Some(member_vec),
    };

    let tags_opt = if val.tags.is_empty() {
      None
    } else {
      Some(val.tags.into_iter().map(|t| t.0).collect())
    };

    FrontEndGroup {
      label: val.label.0,
      description: val.description,
      tags: tags_opt,
      members: Some(members),
      exclusive_group: val.exclusive_group.map(|x| x.0),
    }
  }
}
