//! # csm-rs
//!
//! A Rust library for talking to the HPE Cray Shasta CSM (Cray System
//! Management) API.
//!
//! ## Quick start
//!
//! All HTTP calls hang off [`ShastaClient`]. Construct one per Shasta
//! installation and reuse it across calls — it caches a pre-built
//! `reqwest::Client` (with connection pool, TLS context, DNS resolver).
//! The bearer token is supplied per call so a single client can serve
//! many tokens:
//!
//! ```no_run
//! # async fn example() -> Result<(), csm_rs::error::Error> {
//! let client = csm_rs::ShastaClient::new(
//!     "https://api.shasta.example.com",
//!     std::fs::read("/etc/shasta/ca.crt").unwrap(),
//!     None, // or Some("socks5://localhost:9050".to_string())
//! )?;
//!
//! let token = "your-bearer-token";
//! // Methods are namespaced by API module: `<module>_<resource>_<verb>`.
//! // The first arg is always the bearer token.
//! let images = client.ims_image_get_all(token).await?;
//! let groups = client.hsm_group_get_all(token).await?;
//! let configs = client.cfs_configuration_v2_get_all(token).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Migrating from earlier releases
//!
//! - **Releases ≤ 0.106**: exposed each HTTP call as a free function with
//!   a 4-parameter auth quartet (`token`, `base_url`, `root_cert`,
//!   `proxy`). Removed in 0.107.
//! - **0.107.x**: replaced the free functions with methods on
//!   [`ShastaClient`]; the token was stored on the client.
//! - **0.108 (this release)**: removed the token from
//!   [`ShastaClient`] — it is now passed per call as the first method
//!   argument. One client can serve many tokens, and the underlying
//!   `reqwest::Client` (with its connection pool) is reused across all
//!   of them.
//!
//! ```ignore
//! // 0.107.x
//! let client = ShastaClient::new(base_url, token, cert, proxy)?;
//! client.ims_image_get_all().await?;
//!
//! // 0.108+
//! let client = ShastaClient::new(base_url, cert, proxy)?;
//! client.ims_image_get_all(token).await?;
//! ```
//!
//! ## Source layout
//!
//! Every CSM API namespace under `src/` follows the same shape so
//! reviewers don't have to track per-module conventions:
//!
//! ```text
//! <namespace>/                 // bos, bss, capmc, cfs, hsm, ims, pcs
//!   mod.rs                     // module docs + canonical `pub use` aliases
//!   <resource>/                // e.g. cfs/configuration, bos/session
//!     mod.rs                   // resource docs; declares the items below
//!     types.rs                 // wire-format request/response structs
//!     http_client/             // `impl ShastaClient` HTTP method blocks
//!       mod.rs                 // unversioned methods (or `v{N}/mod.rs` for
//!       v2/mod.rs              // version-split CSM APIs — currently
//!       v3/mod.rs              // `cfs/*` and `bos/{session, template}`)
//!     dispatcher_conv.rs       // `From` impls between local and
//!                              // `manta-backend-dispatcher` types
//!                              // (gated by the `manta-dispatcher` feature)
//!     utils.rs                 // helpers built on the raw HTTP methods
//! ```
//!
//! Higher-level composed operations that combine multiple namespaces
//! live in `commands/`, with the most CLI-shaped ones (file I/O, YAML,
//! progress bars) gated behind the `commands-admin` Cargo feature.

#![allow(clippy::doc_lazy_continuation)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]

#[cfg(feature = "manta-dispatcher")]
pub mod backend_connector;
pub mod bos;
pub mod bss;
pub mod capmc;
pub mod cfs;
pub mod client;
pub mod commands;
pub(crate) mod common;
pub mod error;
pub mod hsm;
pub mod ims;
pub mod node;
pub mod pcs;

pub use client::ShastaClient;
pub use error::Error;

// Canonical type re-exports lifted from each namespace's `mod.rs`. Only
// types that are already curated as the namespace-level canonical name
// are re-exported here; deep `*::http_client::*::types::*` paths stay
// internal so a future version bump is a single edit in the namespace.
pub use bos::{BosSession, BosSessionTemplate};
pub use bss::BootParameters;
pub use capmc::{XnameError, XnamePowerActionResponse, XnameStatusResponse};
pub use ims::{Image, Link, PatchImage, PublicKey};
// CFS exposes both v2 and v3 endpoints with structurally different
// wire types, so the canonical surface is the `cfs::v2` and `cfs::v3`
// submodules rather than a single crate-root alias.
