//! Small helpers for navigating user-supplied YAML/JSON Value trees with
//! proper error returns instead of panics.
//!
//! Callers (mostly SAT-file processing) work with `serde_yaml::Value` /
//! `serde_json::Value` shapes whose layout is dictated by user input.
//! The historical pattern was `.get("k").and_then(Value::as_str).unwrap()`
//! which panics on any unexpected shape. These helpers convert that to
//! `Error::Message` with field context so missing or wrong-typed fields
//! surface as actionable errors.

use crate::error::Error;

/// Get `value[key]` as `&str`, or return `Error::Message` with field context.
pub(crate) fn yaml_str<'a>(
  value: &'a serde_yaml::Value,
  key: &str,
) -> Result<&'a str, Error> {
  value
    .get(key)
    .and_then(serde_yaml::Value::as_str)
    .ok_or_else(|| {
      Error::Message(format!(
        "SAT file: missing or non-string field '{}'",
        key
      ))
    })
}

/// Get `value[key]` as `&Vec<serde_yaml::Value>` (a sequence), or
/// `Error::Message` with field context.
pub(crate) fn yaml_seq<'a>(
  value: &'a serde_yaml::Value,
  key: &str,
) -> Result<&'a Vec<serde_yaml::Value>, Error> {
  value
    .get(key)
    .and_then(serde_yaml::Value::as_sequence)
    .ok_or_else(|| {
      Error::Message(format!(
        "SAT file: missing or non-sequence field '{}'",
        key
      ))
    })
}

/// Convert a `serde_yaml::Value` to `&str`, or `Error::Message`. Used when a
/// known-existing `Value` is expected to be a string (e.g. inside a chain
/// where the caller already extracted the value but wants to fail cleanly
/// if it's the wrong type).
pub(crate) fn as_yaml_str(value: &serde_yaml::Value) -> Result<&str, Error> {
  value.as_str().ok_or_else(|| {
    Error::Message("SAT file: value is not a string".to_string())
  })
}

/// Get `value[key]` as `&str` from a `serde_json::Value`. Same semantics as
/// `yaml_str` but for JSON.
pub(crate) fn json_str<'a>(
  value: &'a serde_json::Value,
  key: &str,
) -> Result<&'a str, Error> {
  value
    .get(key)
    .and_then(serde_json::Value::as_str)
    .ok_or_else(|| {
      Error::Message(format!(
        "Missing or non-string field '{}' in JSON response",
        key
      ))
    })
}
