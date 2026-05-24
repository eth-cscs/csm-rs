pub mod authentication;
pub mod bos; // ApplySessionTrait, ClusterSessionTrait, ClusterTemplateTrait
pub mod bss; // BootParametersTrait
pub mod cfs; // CfsTrait
pub mod cleanup; // DeleteConfigurationsAndDataRelatedTrait
pub mod console; // ConsoleTrait
pub mod group; // GroupTrait
pub mod hsm; // HardwareInventory, ComponentTrait, ComponentEthernetInterfaceTrait, RedfishEndpointTrait
pub mod ims; // ImsTrait, GetImagesAndDetailsTrait
pub mod migrate; // MigrateRestoreTrait, MigrateBackupTrait
pub mod pcs; // PCSTrait
pub mod sat; // SatTrait, ApplyHwClusterPin
#[derive(Debug, Clone)]
pub struct Csm {
  pub(crate) base_url: String,
  pub(crate) root_cert: Vec<u8>,
  pub(crate) socks5_proxy: Option<String>,
}

impl Csm {
  pub fn new(base_url: &str, root_cert: &[u8], socks5_proxy: Option<&str>) -> Self {
    Self {
      base_url: base_url.to_string(),
      root_cert: root_cert.to_vec(),
      socks5_proxy: socks5_proxy.map(str::to_owned),
    }
  }

  /// Build a `ShastaClient` for this `Csm` + the supplied per-call token.
  /// Cheap: cert parse + reqwest::Client::build per call (microseconds).
  pub(crate) fn shasta_client(
    &self,
    token: &str,
  ) -> Result<crate::ShastaClient, manta_backend_dispatcher::error::Error> {
    crate::ShastaClient::new(
      &self.base_url,
      token,
      self.root_cert.clone(),
      self.socks5_proxy.clone(),
    )
    .map_err(|e| {
      manta_backend_dispatcher::error::Error::Message(e.to_string())
    })
  }
}
