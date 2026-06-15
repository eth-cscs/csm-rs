//! progenitor-generated CFS client. Output of `build.rs`, included verbatim.
//!
//! `pub(crate)` because only the wrapper layer in `crate::cfs::wrapper`
//! and per-resource `types.rs` re-export aliases are allowed to touch
//! the generated symbols. Public consumers go through `ShastaClient`.
#![allow(
  dead_code,
  clippy::all,
  missing_docs,
  non_camel_case_types,
  non_snake_case,
  // unused_comparisons: generated `if value.len() < 0usize` checks in
  // serde min_length validators are always-false; rustc would warn
  // (lint is rustc, not clippy, so `clippy::all` doesn't cover it).
  unused_comparisons,
  unused_imports
)]
include!(concat!(env!("OUT_DIR"), "/cfs_generated.rs"));
