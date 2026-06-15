//! Apply a hardware pattern to (re)compose an HSM group from a parent
//! group.
//!
//! Submodules:
//!
//! - [`command`] — the entry-point `exec` function.
//! - [`utils`] — building blocks (component counting, pattern matching).

pub mod command;
#[cfg(test)]
mod tests;
pub mod utils;

#[doc(inline)]
pub use command::exec;
