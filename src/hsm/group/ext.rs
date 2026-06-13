//! Convenience methods previously defined as inherent `impl Group` in
//! `types.rs`. Now exposed as a trait because `Group` is a generated
//! type and an inherent `impl` block in this crate would collide with
//! the generated impls.
//!
//! Callers add `use csm_rs::hsm::group::GroupExt;` (or rely on a glob
//! import of `hsm::group::*`) to keep using these.
//!
//! Trait method naming notes:
//!
//! - `new_with_members` replaces the old inherent `Group::new(...)`.
//!   You cannot call `Self::new(...)` through a trait without a turbofish
//!   so the name is forced to differ from any plain `new`. There were
//!   no production callers of `Group::new` — only the in-crate test in
//!   `tests.rs` — so the rename ripple is minimal.
//! - `get_members`/`get_members_opt`/`add_xnames` keep their existing
//!   names so call sites only need `use GroupExt;` rather than a rename.

use super::types::{Group, Members, ResourceName, XNameRw100};

/// Convenience methods on the progenitor-generated [`Group`] type.
pub trait GroupExt: Sized {
  /// Build a new `Group` with `label` and an optional `members.ids` list.
  ///
  /// `label` is wrapped in the generated `ResourceName(pub String)`
  /// newtype directly. The upstream OpenAPI schema declared a regex
  /// pattern (`^[0-9a-f_\-.]{1,}$`) that rejected real-world labels
  /// like `zinal` or `x1000c0s0b0n0`; we stripped it in
  /// `csm_api_docs.openapi3.json` so this constructor accepts any
  /// string — same as the historical `Group::new`.
  fn new_with_members(label: &str, member_vec_opt: Option<Vec<&str>>) -> Self;

  /// All member xnames as owned `Vec<String>`; empty if the field is absent.
  fn get_members(&self) -> Vec<String>;

  /// Same as `get_members` but distinguishes "no members" (returns
  /// `Some(empty)` when the `members` object is present but its `ids`
  /// array is empty) from "field missing" (returns `None`).
  fn get_members_opt(&self) -> Option<Vec<String>>;

  /// Append `xnames` to `members.ids`. Returns the resulting list.
  /// If the `members` field was absent it is created; if its `ids`
  /// array was empty it is extended in place.
  fn add_xnames(&mut self, xnames: &[String]) -> Vec<String>;
}

impl GroupExt for Group {
  fn new_with_members(label: &str, member_vec_opt: Option<Vec<&str>>) -> Self {
    let members_opt = member_vec_opt.map(|member_vec| Members {
      ids: member_vec
        .iter()
        .map(|&id| XNameRw100(id.to_string()))
        .collect(),
    });
    Self {
      // `ResourceName` is a public newtype with an infallible
      // `From<String>` (the upstream regex was stripped in Task 0; see
      // the comment on the schema in `csm_api_docs.openapi3.json`), so
      // any string the caller supplies round-trips.
      label: ResourceName(label.to_string()),
      description: None,
      tags: vec![],
      members: members_opt,
      exclusive_group: None,
    }
  }

  fn get_members(&self) -> Vec<String> {
    match self.members.as_ref() {
      Some(members) => members.ids.iter().map(|x| x.0.clone()).collect(),
      None => Vec::new(),
    }
  }

  fn get_members_opt(&self) -> Option<Vec<String>> {
    self
      .members
      .as_ref()
      .map(|members| members.ids.iter().map(|x| x.0.clone()).collect())
  }

  fn add_xnames(&mut self, xnames: &[String]) -> Vec<String> {
    let members = self.members.get_or_insert_with(|| Members { ids: vec![] });
    members
      .ids
      .extend(xnames.iter().cloned().map(XNameRw100));
    self.get_members()
  }
}
