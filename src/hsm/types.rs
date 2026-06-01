//! HSM-wide shared response types — used across multiple HSM
//! submodules (component, group, hw_inventory, redfish_endpoint).
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

/// Success-path ack returned by most HSM mutating endpoints
/// (POST/DELETE/PATCH). Mirrors the swagger `Response_1.0.0` schema —
/// `code` is a string status (e.g. "0" for success), `message`
/// typically carries a count of new/modified/deleted items. Error
/// responses use `Problem7807` and surface via
/// [`Error::CsmError`](crate::error::Error::CsmError) instead.
///
/// Both `code` and `message` are nominally required by the swagger but
/// HSM servers sometimes omit them when empty, so `#[serde(default)]`
/// makes deserialization tolerant of the absence.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct HsmActionResponse {
  #[serde(default)]
  pub code: String,
  #[serde(default)]
  pub message: String,
}

/// One entry in the array returned by `POST /Inventory/RedfishEndpoints`
/// (and similar create-resource endpoints). Holds the URI of the
/// just-created resource — callers can follow it with a GET to read
/// the canonical state.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceURI {
  #[serde(rename = "URI")]
  pub uri: String,
}
