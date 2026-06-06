//! Apply a SAT (System Admin Toolkit) YAML file against a Shasta system.
//!
//! Submodules:
//!
//! - [`command`] — the entry-point `exec` function.
//! - [`utils`] — section-level helpers (configurations, images, session
//!   templates) used by the workflow.

pub mod command;
#[cfg(test)]
pub mod tests;
pub mod utils;

#[doc(inline)]
pub use command::exec;
