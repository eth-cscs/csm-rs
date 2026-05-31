//! Bidirectional `From` impls between csm-rs's BSS types and the
//! dispatcher's mirrors. Gated behind the `manta-dispatcher` Cargo
//! feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::bss::BootParameters as FrontEndBootParameters;

use super::types::BootParameters;

impl From<FrontEndBootParameters> for BootParameters {
  fn from(value: FrontEndBootParameters) -> Self {
    BootParameters {
      hosts: value.hosts,
      macs: value.macs,
      nids: value.nids,
      params: value.params,
      kernel: value.kernel,
      initrd: value.initrd,
      cloud_init: value.cloud_init,
    }
  }
}

impl From<BootParameters> for FrontEndBootParameters {
  fn from(val: BootParameters) -> Self {
    FrontEndBootParameters {
      hosts: val.hosts,
      macs: val.macs,
      nids: val.nids,
      params: val.params,
      kernel: val.kernel,
      initrd: val.initrd,
      cloud_init: val.cloud_init,
    }
  }
}
