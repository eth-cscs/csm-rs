//! progenitor-generated BSS client. Output of `build.rs`, included verbatim.
//!
//! `pub(crate)` because only the wrapper layer in `crate::bss::wrapper`
//! and `types.rs` re-export aliases are allowed to touch the generated
//! symbols. Public consumers go through `ShastaClient`.
#![allow(
  dead_code,
  clippy::all,
  missing_docs,
  non_camel_case_types,
  non_snake_case,
  unused_comparisons,
  unused_imports
)]
include!(concat!(env!("OUT_DIR"), "/bss_generated.rs"));
