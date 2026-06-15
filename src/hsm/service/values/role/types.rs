//! Re-export of the progenitor-generated role-values schema.
//!
//! The endpoint `GET /service/values/role` returns the schema
//! `Values.1.0.0_RoleArray`, mangled by progenitor/typify to
//! `Values100RoleArray` (see Section A of the progenitor output
//! reference doc). The old hand-written `Role { role: Vec<String> }`
//! struct collapses to the same JSON shape as the generated
//! `Values100RoleArray { role: Vec<HmsRole100> }` because
//! `HmsRole100` is a `serde(transparent)`-equivalent newtype around
//! `String`.

pub use crate::hsm::generated::types::Values100RoleArray as Role;
