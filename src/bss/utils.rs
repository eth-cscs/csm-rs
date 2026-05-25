//! Helpers built on top of [`crate::ShastaClient`]`::bss_*` methods.

use std::collections::HashMap;

use super::types::BootParameters;

// Assumes s3 path looks like:
// - s3://boot-images/59e0180a-3fdd-4936-bba7-14ba914ffd34/kernel
// - craycps-s3:s3://boot-images/59e0180a-3fdd-4936-bba7-14ba914ffd34/rootfs:3dfae8d1fa3bb2bfb18152b4f9940ad0-667:dvs:api-gw-service-nmn.local:300:nmn0,hsn0:0
// - url=s3://boot-images/59e0180a-3fdd-4936-bba7-14ba914ffd34/rootfs,etag=3dfae8d1fa3bb2bfb18152b4f9940ad0-667 bos_update_frequency=4h
pub fn get_image_id_from_s3_path(s3_path: &str) -> Option<&str> {
  s3_path.split("/").nth(3)
}

pub fn convert_kernel_params_to_map(
  kernel_params: &str,
) -> HashMap<String, String> {
  kernel_params
    .split_whitespace()
    .map(|kernel_param| {
      let (key_str, value_str) =
        kernel_param.split_once('=').unwrap_or((kernel_param, ""));

      let key = key_str.to_string();
      let value = value_str.to_string();

      (key, value)
    })
    .collect()
}

pub fn find_boot_params_related_to_node(
  node_boot_params_list: &[BootParameters],
  node: &String,
) -> Option<BootParameters> {
  node_boot_params_list
    .iter()
    .find(|node_boot_param| {
      node_boot_param.hosts.iter().any(|host| host.eq(node))
    })
    .cloned()
}

#[cfg(test)]
mod tests {
  use super::*;

  // ---------- convert_kernel_params_to_map ----------

  #[test]
  fn convert_kernel_params_empty_input_returns_empty_map() {
    assert!(convert_kernel_params_to_map("").is_empty());
  }

  #[test]
  fn convert_kernel_params_whitespace_only_returns_empty_map() {
    assert!(convert_kernel_params_to_map("   \t\n  ").is_empty());
  }

  #[test]
  fn convert_kernel_params_single_kv_pair() {
    let map = convert_kernel_params_to_map("foo=bar");
    assert_eq!(map.get("foo"), Some(&"bar".to_string()));
    assert_eq!(map.len(), 1);
  }

  #[test]
  fn convert_kernel_params_multiple_kv_pairs() {
    let map = convert_kernel_params_to_map("a=1 b=2 c=3");
    assert_eq!(map.get("a"), Some(&"1".to_string()));
    assert_eq!(map.get("b"), Some(&"2".to_string()));
    assert_eq!(map.get("c"), Some(&"3".to_string()));
    assert_eq!(map.len(), 3);
  }

  #[test]
  fn convert_kernel_params_value_with_equals_splits_only_at_first() {
    // `split_once('=')` splits at the FIRST '=' — the rest stays in value
    let map = convert_kernel_params_to_map("path=s3://bucket/key=etag");
    assert_eq!(map.get("path"), Some(&"s3://bucket/key=etag".to_string()));
  }

  #[test]
  fn convert_kernel_params_bare_flag_maps_to_empty_value() {
    let map = convert_kernel_params_to_map("quiet console=tty0");
    assert_eq!(map.get("quiet"), Some(&"".to_string()));
    assert_eq!(map.get("console"), Some(&"tty0".to_string()));
  }

  #[test]
  fn convert_kernel_params_collapses_runs_of_whitespace() {
    let map = convert_kernel_params_to_map("a=1   b=2\tc=3");
    assert_eq!(map.len(), 3);
  }

  // ---------- get_image_id_from_s3_path ----------
  //
  // Note: there are already tests for happy-path s3:// inputs in tests.rs;
  // we add the edge cases here.

  #[test]
  fn get_image_id_from_s3_path_returns_none_for_too_short_input() {
    assert_eq!(get_image_id_from_s3_path("s3://"), None);
    assert_eq!(get_image_id_from_s3_path("s3://only-two-segments"), None);
  }

  #[test]
  fn get_image_id_from_s3_path_returns_segment_3_regardless_of_scheme() {
    // The function just splits on `/` and grabs index 3, no scheme check.
    // For `scheme://host/seg3/seg4`, the splits are
    // ["scheme:", "", "host", "seg3", "seg4"], so index 3 is "seg3".
    assert_eq!(
      get_image_id_from_s3_path("https://example.com/image-id/more"),
      Some("image-id")
    );
  }
}
