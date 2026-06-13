//! Bidirectional `From` impls between csm-rs's HSM-wide shared
//! response types and the dispatcher's mirror. Gated behind the
//! `manta-dispatcher` Cargo feature.

use manta_backend_dispatcher::types::HsmActionResponse as FrontEndHsmActionResponse;

use super::types::HsmActionResponse;

impl From<FrontEndHsmActionResponse> for HsmActionResponse {
  fn from(value: FrontEndHsmActionResponse) -> Self {
    // Dispatcher mirror keeps `code` as String; csm-rs uses i64 (the
    // wire form). Parse on the way in, default to 0 if the dispatcher
    // ever holds a non-numeric value.
    HsmActionResponse {
      code: value.code.parse().unwrap_or(0),
      message: value.message,
    }
  }
}

impl From<HsmActionResponse> for FrontEndHsmActionResponse {
  fn from(val: HsmActionResponse) -> Self {
    FrontEndHsmActionResponse {
      code: val.code.to_string(),
      message: val.message,
    }
  }
}
