//! HSM components — individual hardware components (nodes, BMCs, …) and
//! their identifiers/state.
//!
//! Submodules:
//!
//! - [`types`] — re-exports of the progenitor-generated request/response
//!   shapes for `/State/Components`.
//!
//! The `ShastaClient` methods for `/smd/hsm/v2/State/Components` live in
//! `crate::hsm::wrapper::component`. That wrapper file documents per
//! method why each one stays on raw reqwest rather than routing through
//! the generated progenitor client.

pub mod types;

use types::Component;

/// In-place retain of components whose `id` is in `xname_list`.
///
/// The `id` field on the generated `Component` is `Option<XName100>`;
/// `XName100(pub String)` derefs to `String`, so the inner `.0` is the
/// raw xname string we compare against `xname_list`.
pub fn filter(component_vec: &mut Vec<Component>, xname_list: &[String]) {
  component_vec.retain(|component| {
    if let Some(xname) = &component.id {
      xname_list.contains(&xname.0)
    } else {
      false
    }
  });
}

/// Bidirectional `From` impls between [`types`] and the dispatcher's
/// HSM component mirror types. Gated behind the `manta-dispatcher`
/// Cargo feature.
#[cfg(feature = "manta-dispatcher")]
mod dispatcher_conv;
