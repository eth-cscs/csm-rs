//! IMS recipes — inputs from which an image is built.
//!
//! Submodules:
//!
//! - `http_client` — `ShastaClient` methods for `/ims/v3/recipes`.
//! - `types` — request/response shapes.

pub mod http_client;
pub mod types;

// Canonical names: callers should prefer these over the deeper
// `types::*` paths so the internal layout can evolve without rippling
// through every command. (`Link` is the IMS-recipe artifact link, which
// is distinct from `ims::image::Link`; keep the per-submodule namespace.)
pub use types::{Link, RecipeGetResponse};
