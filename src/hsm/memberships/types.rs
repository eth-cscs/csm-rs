//! Re-export of the progenitor-generated `Membership` schema.
//!
//! The endpoint `GET /memberships` returns the schema `Membership.1.0.0`,
//! mangled by progenitor/typify to `Membership100` (see Section A of the
//! progenitor output reference doc). The old hand-written struct had
//! `id: String`, `partition_name: String`, `group_labels: Vec<String>`;
//! the generated type uses `id: Option<XName100>` (newtype around
//! `String`) and `partition_name: Option<String>` because the upstream
//! OpenAPI schema declares these fields optional. Callers that need a
//! plain `String` should pattern-match the `Option` or unwrap via the
//! generated `XName100`'s `Deref<Target = String>` / `From<XName100> for
//! String` impls.

pub use crate::hsm::generated::types::Membership100 as Membership;
