//! Built-in fallback list of HSM node roles for offline use.

/// Return the canonical list of HSM node roles as a `Vec<String>`,
/// used when the live `service/values/role` endpoint is unreachable.
#[must_use]
pub fn get() -> Vec<String> {
  vec![
    "Storage".to_string(),
    "Management".to_string(),
    "Compute".to_string(),
    "Service".to_string(),
    "System".to_string(),
    "Application".to_string(),
  ]
}
