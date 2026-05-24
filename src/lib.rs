//! # csm-rs
//!
//! A Rust library for talking to the HPE Cray Shasta CSM (Cray System
//! Management) API.
//!
//! ## Quick start
//!
//! All HTTP calls hang off [`ShastaClient`]. Construct one per Shasta
//! installation and reuse it across calls — it bundles the auth token,
//! base URL, root certificate, and an optional SOCKS5 proxy with a
//! pre-built `reqwest::Client`:
//!
//! ```no_run
//! # async fn example() -> Result<(), csm_rs::error::Error> {
//! let client = csm_rs::ShastaClient::new(
//!     "https://api.shasta.example.com",
//!     "your-bearer-token",
//!     std::fs::read("/etc/shasta/ca.crt").unwrap(),
//!     None, // or Some("socks5://localhost:9050".to_string())
//! )?;
//!
//! // Methods are namespaced by API module: `<module>_<resource>_<verb>`.
//! let images = client.ims_image_get_all().await?;
//! let groups = client.hsm_group_get_all().await?;
//! let configs = client.cfs_configuration_v2_get_all().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Migrating from 0.106 and earlier
//!
//! Releases up to and including 0.106 exposed each HTTP call as a free
//! function with a 4+ parameter auth quartet:
//!
//! ```ignore
//! // 0.106 and earlier — removed in 0.107.
//! ims::image::http_client::get_all(token, base_url, root_cert, proxy).await?;
//! ```
//!
//! In 0.107 these free functions were removed in favor of methods on
//! [`ShastaClient`]. The new equivalent is:
//!
//! ```ignore
//! // 0.107+
//! client.ims_image_get_all().await?;
//! ```
//!
//! The new method names follow `<module>_<resource>_<verb>` and are
//! versioned where the underlying API is (e.g. `cfs_session_v2_get` vs
//! `cfs_session_v3_get`). See each module's docs for the full list.

#![allow(clippy::doc_lazy_continuation)]

pub mod backend_connector;
pub mod bos;
pub mod bss;
pub mod capmc;
pub mod cfs;
pub mod client;
pub mod commands;
pub mod common;
pub mod error;
pub mod hsm;
pub mod ims;
pub mod node;
pub mod pcs;

pub use client::ShastaClient;
