//! Re-exports of the progenitor-generated `Group`/`Members` schemas, plus
//! the hand-rolled `Member` body type used by `hsm_group_post_member`.
//!
//! Behaviour deltas vs. the previous hand-written types — driven by the
//! OpenAPI schema being the source of truth:
//!
//! - `Group.label`: was `String`, now `ResourceName` (newtype around
//!   `String` with `Deref<Target = String>` + `From<String>`). Callers
//!   that need `&str` go via `&*group.label` (deref) or `group.label.as_str()`.
//! - `Group.exclusive_group`: was `Option<String>`, now
//!   `Option<ResourceName>`. Same newtype as `label`.
//! - `Group.tags`: was `Option<Vec<String>>`, now `Vec<ResourceName>`
//!   (the spec uses `#[serde(default)]` for an absent array — there is
//!   no longer a way to distinguish "no tags field" from "empty tags",
//!   but the on-wire shape is unchanged since both serialise as an
//!   absent property when empty).
//! - `Members.ids`: was `Option<Vec<String>>`, now `Vec<XNameRw100>`.
//!   Same `#[serde(default)]` treatment as `tags`.
//!
//! The hand-rolled `Member` (singular) type is kept because the public
//! `hsm_group_post_member(label, Member)` signature accepts it; on the
//! wire it serialises to `{"id": "..."}` which is also what the generated
//! `MemberId` produces. The wrapper translates between them.

use serde::{Deserialize, Serialize};

pub use crate::hsm::generated::types::Group100 as Group;
pub use crate::hsm::generated::types::Members100 as Members;
pub use crate::hsm::generated::types::ResourceName;
pub use crate::hsm::generated::types::XNameRw100;

/// Single-member request body for `POST /groups/{label}/members`.
/// On the wire this is `{"id": "<xname>"}`; the wrapper converts to the
/// generated `MemberId` shape before delegating.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Member {
  /// Component xname id of the new/existing member; serialises as
  /// `{"id": "<xname>"}`. Optional so callers can build a default
  /// struct without specifying an id.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
}
