//! Bidirectional `From` impls between csm-rs's HSM-wide shared
//! response types and the dispatcher's mirror. Gated behind the
//! `manta-dispatcher` Cargo feature.

use manta_backend_dispatcher::types::HsmActionResponse as FrontEndHsmActionResponse;

use super::types::HsmActionResponse;

impl From<FrontEndHsmActionResponse> for HsmActionResponse {
  fn from(value: FrontEndHsmActionResponse) -> Self {
    HsmActionResponse {
      code: value.code,
      message: value.message,
    }
  }
}

impl From<HsmActionResponse> for FrontEndHsmActionResponse {
  fn from(val: HsmActionResponse) -> Self {
    FrontEndHsmActionResponse {
      code: val.code,
      message: val.message,
    }
  }
}
