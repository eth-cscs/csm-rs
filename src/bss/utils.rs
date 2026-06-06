//! Helpers built on top of [`crate::ShastaClient`]`::bss_*` methods.

use super::types::BootParameters;

/// Extract the IMS image ID from a boot-images S3 path.
///
/// Recognises the path shapes:
/// - `s3://boot-images/<image-id>/kernel`
/// - `craycps-s3:s3://boot-images/<image-id>/rootfs:...`
/// - `url=s3://boot-images/<image-id>/rootfs,etag=...`
#[must_use]
pub fn get_image_id_from_s3_path(s3_path: &str) -> Option<&str> {
  s3_path.split('/').nth(3)
}

/// Find the [`BootParameters`] entry whose `hosts` list contains the
/// requested node, returning a clone or `None`.
#[must_use]
pub fn find_boot_params_related_to_node(
  node_boot_params_list: &[BootParameters],
  node: &str,
) -> Option<BootParameters> {
  node_boot_params_list
    .iter()
    .find(|node_boot_param| {
      node_boot_param.hosts.iter().any(|host| host == node)
    })
    .cloned()
}

#[cfg(test)]
mod tests {
  use super::*;

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
