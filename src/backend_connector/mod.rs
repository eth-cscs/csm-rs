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
#[derive(Clone)]
pub struct Csm {
  pub(crate) base_url: String,
  pub(crate) root_cert: Vec<u8>,
}

impl Csm {
  pub fn new(base_url: &str, root_cert: &[u8]) -> Self {
    Self {
      base_url: base_url.to_string(),
      root_cert: root_cert.to_vec(),
    }
  }
}
