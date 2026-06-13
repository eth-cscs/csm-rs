//! HSM groups — named collections of components used by CFS, BOS, and
//! related services as targets.
//!
//! Submodules:
//!
//! - `wrapper/group.rs` — `ShastaClient` methods for
//!   `/smd/hsm/v2/groups` (the file lives under `src/hsm/wrapper/`).
//! - [`types`] — re-exports of the progenitor-generated request/response
//!   shapes.
//! - [`ext`] — `GroupExt` trait with the convenience methods that used
//!   to be inherent on `Group`.
//! - [`utils`] — composed helpers (membership unions, substring lookup).
//! - [`hacks`] — workarounds for CSM behaviour that doesn't fit cleanly
//!   into the rest of the surface.

/// `GroupExt` trait with the convenience methods (`new_with_members`,
/// `get_members`, `get_members_opt`, `add_xnames`) that used to live as
/// an inherent `impl Group` block. Re-exported at the module root so a
/// glob import keeps working.
pub mod ext;
pub use ext::GroupExt;

/// Workarounds for CSM HSM behaviour that does not fit cleanly into
/// the rest of the surface.
pub mod hacks;
/// Integration-style tests for the HSM group namespace.
#[cfg(test)]
pub mod tests;
pub mod types;
pub mod utils;

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// HSM group mirror types. Gated behind the `manta-dispatcher`
/// Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;
