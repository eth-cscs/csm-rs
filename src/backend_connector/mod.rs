//! Backend-dispatcher integration layer.
//!
//! Implements the trait surface defined by
//! [`manta_backend_dispatcher`](https://crates.io/crates/manta-backend-dispatcher)
//! so that csm-rs can be plugged into Manta (or any other dispatcher
//! consumer) as a concrete CSM backend.
//!
//! Each submodule below wires one trait family to the corresponding
//! csm-rs API surface — see the inline annotations for which trait each
//! file implements. The [`Csm`] type carries the connection metadata
//! (base URL, root cert, optional SOCKS5 proxy) those impls need;
//! per-request bearer tokens are passed in by the dispatcher.
//!
//! Consumers that talk to CSM directly should reach for
//! [`crate::ShastaClient`] instead — this module exists specifically to
//! satisfy the dispatcher contract.

pub mod authentication;
pub mod bos; // ApplySessionTrait, ClusterSessionTrait, ClusterTemplateTrait
pub mod bss; // BootParametersTrait
pub mod cfs; // CfsTrait
pub mod cleanup; // DeleteConfigurationsAndDataRelatedTrait
// `ConsoleTrait` attaches to the in-cluster `cray-console-node` pod
// via `kube`, so it requires the same `k8s-console` feature as the
// underlying console helpers.
#[cfg(feature = "k8s-console")]
pub mod console; // ConsoleTrait
pub mod group; // GroupTrait
pub mod hsm; // HardwareInventory, ComponentTrait, ComponentEthernetInterfaceTrait, RedfishEndpointTrait
pub mod ims; // ImsTrait, GetImagesAndDetailsTrait
// `MigrateRestoreTrait`/`MigrateBackupTrait` and `SatTrait` are
// implemented in terms of the CLI-shaped admin workflows under
// `commands::{migrate_*, i_apply_sat_file}`, so they are gated behind
// the same `commands-admin` feature.
#[cfg(feature = "commands-admin")]
pub mod migrate; // MigrateRestoreTrait, MigrateBackupTrait
pub mod pcs; // PCSTrait
#[cfg(feature = "commands-admin")]
pub mod sat; // SatTrait, ApplyHwClusterPin

/// Connection metadata for one Shasta installation, used by the
/// [`manta_backend_dispatcher`] trait implementations in this module.
///
/// Holds the base URL, PEM root cert, optional SOCKS5 proxy, and a
/// pre-built [`crate::ShastaClient`] (constructed once at `Csm::new`
/// time and shared across every dispatcher call). Bearer tokens are
/// passed in per request by the dispatcher and are **not** stored.
#[derive(Debug, Clone)]
pub struct Csm {
  pub(crate) base_url: String,
  pub(crate) root_cert: Vec<u8>,
  pub(crate) socks5_proxy: Option<String>,
  pub(crate) client: crate::ShastaClient,
}

impl Csm {
  /// Construct a `Csm` from a base URL, PEM-encoded root cert, and an
  /// optional SOCKS5 proxy URL.
  ///
  /// Builds the underlying `reqwest::Client` (cert parse, connection
  /// pool, DNS resolver, TLS context) once and caches it on `self.
  /// client`; trait-method implementations reuse it across all calls.
  ///
  /// # Errors
  ///
  /// Returns an error if [`crate::ShastaClient::new`] fails — typically
  /// because the proxy URL is malformed.
  pub fn new(
    base_url: &str,
    root_cert: &[u8],
    socks5_proxy: Option<&str>,
  ) -> Result<Self, manta_backend_dispatcher::error::Error> {
    let client = crate::ShastaClient::new(
      base_url,
      root_cert.to_vec(),
      socks5_proxy.map(str::to_owned),
    )
    .map_err(manta_backend_dispatcher::error::Error::from)?;
    Ok(Self {
      base_url: base_url.to_string(),
      root_cert: root_cert.to_vec(),
      socks5_proxy: socks5_proxy.map(str::to_owned),
      client,
    })
  }

  /// Borrow the cached [`crate::ShastaClient`].
  ///
  /// One client serves every dispatcher trait call — the underlying
  /// `reqwest::Client` (with its connection pool) is reused across all
  /// of them.
  pub(crate) fn shasta_client(&self) -> &crate::ShastaClient {
    &self.client
  }
}
