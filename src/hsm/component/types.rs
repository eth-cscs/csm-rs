//! Re-exports of the progenitor-generated `Component`-family schemas.
//!
//! Behaviour deltas vs. the previous hand-written types — driven by the
//! OpenAPI schema being the source of truth:
//!
//! - `Component.id`: was `Option<String>`, now `Option<XName100>`
//!   (newtype around `String` with `Deref<Target = String>` +
//!   `From<XName100> for String`). Callers comparing to a plain `&str`
//!   go via `&*component.id` (deref) or `id.0.as_str()`.
//! - `Component.type`: was `Option<String>`, now
//!   `Option<HmsType100>` (a `Copy` enum with `Display` showing the wire
//!   name). The field is renamed `type_` because `type` is a Rust
//!   keyword (the generated struct uses `#[serde(rename = "Type")]` so
//!   the wire shape is unchanged).
//! - `Component.state`: was `Option<String>`, now `Option<HmsState100>`
//!   (a `Copy` enum with `Display`).
//! - `Component.flag`: was `Option<String>`, now `Option<HmsFlag100>`.
//! - `Component.role`: was `Option<String>`, now `Option<HmsRole100>`
//!   (a newtype around `String` — the upstream schema is open-set, see
//!   `GET /service/values/role`).
//! - `Component.sub_role`: was `Option<String>`, now
//!   `Option<HmsSubRole100>` (a newtype around `String`).
//! - `Component.arch`: was `Option<String>`, now `Option<HmsArch100>`.
//! - `Component.class`: was `Option<String>`, now `Option<HmsClass100>`.
//! - `Component.net_type`: was `Option<String>`, now `Option<NetType100>`.
//! - `Component.nid`: was `Option<usize>`, now `Option<i64>` — matches the
//!   OpenAPI `type: integer` (no `minimum: 0`), so the generated `i64`
//!   is the schema-faithful shape; the previous `usize` was a misread.
//!
//! - `ComponentArray.components`: was `Option<Vec<Component>>`, now
//!   `Vec<Component100Component>` (the spec uses `#[serde(default)]` for
//!   an absent `Components` array — the wire shape is unchanged because
//!   both forms serialise to an absent property when empty, but call
//!   sites doing `.unwrap_or_default()` need to drop the unwrap).
//!
//! - `ComponentCreate.id`: was `String` (pub(super) field), now
//!   `XNameRw100` (the read-write xname newtype required by the upstream
//!   `Component.1.0.0_ComponentCreate` schema).
//! - `ComponentCreate.state`: was `String`, now `HmsState100`.
//! - Other `ComponentCreate` enum-shaped fields are wrapped the same way
//!   as their `Component` counterparts.
//!
//! - `ComponentArrayPostQuery`: was a flat hand-rolled struct with all
//!   scalar `Option<String>` filter fields. The generated form makes the
//!   array filters (`arch`, `class`, `enabled`, `flag`, `nid`, `role`,
//!   `softwarestatus`, `state`, `subrole`, `subtype`, `type_`) into
//!   `Vec<String>` with `#[serde(default, skip_serializing_if =
//!   "Vec::is_empty")]`, matching the OpenAPI declaration. The previous
//!   single-scalar shape silently coerced one value; callers wanting
//!   multi-value semantics should push to the vec directly. Note the
//!   `component_ids` field is now `component_i_ds: Vec<XNameForQuery100>`
//!   because typify mangles `ComponentIDs` field-by-field
//!   (`I`+`D`+`s`) — wire name unchanged via serde rename.
//!
//! - `ComponentArrayPostByNidQuery.nid_ranges`: was `Vec<String>`, now
//!   `Vec<NidRange100>` (newtype around `String`).
//!
//! - `ComponentPut.component`: was `pub(super)` (effectively private to
//!   the http_client), now a regular public field on
//!   `Component100Put`.

pub use crate::hsm::generated::types::{
  Component100Component as Component,
  Component100ComponentCreate as ComponentCreate,
  Component100Put as ComponentPut,
  ComponentArrayComponentArray as ComponentArray,
  ComponentArrayPostArray, ComponentArrayPostByNidQuery,
  ComponentArrayPostQuery, HmsArch100, HmsClass100, HmsFlag100, HmsRole100,
  HmsState100, HmsSubRole100, HmsType100, NetType100, NidRange100,
  XName100, XNameForQuery100, XNamePartition100, XNameRw100,
};
