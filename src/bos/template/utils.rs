//! Helpers built on top of [`crate::ShastaClient`]`::bos_template_*` methods.

use crate::{
  bos::template::http_client::v2::types::BosSessionTemplate, error::Error,
};
use globset::Glob;

/// Filter a vector of BOS session templates in place by configuration
/// glob, target HSM groups, target xnames, and an optional row limit.
///
/// # Errors
///
/// Returns an [`Error`] variant on CSM, transport, or
/// deserialization failure; see the crate-level `Error` enum
/// for the full set.
pub fn filter(
  bos_sessiontemplate_vec: &mut Vec<BosSessionTemplate>,
  configuration_name_pattern_opt: Option<&str>,
  target_hsm_group_name_vec: &[String],
  xname_vec: &[String],
  // cfs_configuration_name_opt: Option<&str>,
  limit_number_opt: Option<&u8>,
) -> Result<Vec<BosSessionTemplate>, Error> {
  log::debug!("Filter BOS sessiontemplates");

  if let Some(configuration_name_pattern) = configuration_name_pattern_opt {
    let glob = Glob::new(configuration_name_pattern)?.compile_matcher();

    bos_sessiontemplate_vec.retain(|sessiontemplate| {
      sessiontemplate
        .configuration_name()
        .is_some_and(|configuration_name| glob.is_match(configuration_name))
    });
  };

  // Filter by list of HSM group or xnames as target
  if !target_hsm_group_name_vec.is_empty() || !xname_vec.is_empty() {
    bos_sessiontemplate_vec.retain(|bos_sessiontemplate| {
      let bos_sessiontemplate_target_hsm = bos_sessiontemplate.get_target_hsm();
      let bos_sessiontemplate_target_xname =
        bos_sessiontemplate.get_target_xname();

      !bos_sessiontemplate_target_hsm.is_empty()
        && bos_sessiontemplate_target_hsm
          .iter()
          .all(|bos_st_hsm_group| {
            target_hsm_group_name_vec.iter().any(|target_hsm_group| {
              bos_st_hsm_group.contains(target_hsm_group)
            })
          })
        || !bos_sessiontemplate_target_xname.is_empty()
          && bos_sessiontemplate_target_xname
            .iter()
            .all(|target_xname| xname_vec.contains(target_xname))
    });
  }

  if let Some(limit_number) = limit_number_opt {
    // Limiting the number of results to return to client
    *bos_sessiontemplate_vec = bos_sessiontemplate_vec
      [bos_sessiontemplate_vec
        .len()
        .saturating_sub(*limit_number as usize)..]
      .to_vec();
  }

  Ok(bos_sessiontemplate_vec.to_vec())
}

/// Retain only BOS session templates whose `configuration` equals the
/// supplied CFS configuration name.
pub async fn filter_by_configuration(
  bos_sessiontemplate_vec: &mut Vec<BosSessionTemplate>,
  cfs_configuration_name: &str,
) {
  bos_sessiontemplate_vec.retain(|bos_template| {
    bos_template.get_configuration() == Some(cfs_configuration_name)
  });
}

/// For each BOS session template, return a tuple of
/// `(image_id, cfs_configuration_name, target_xnames)` extracted from
/// the template's boot sets.
pub fn get_image_id_cfs_configuration_target_tuple_vec(
  bos_sessiontemplate_value_vec: &Vec<BosSessionTemplate>,
) -> Vec<(String, String, Vec<String>)> {
  let mut image_id_cfs_configuration_from_bos_sessiontemplate: Vec<(
    String,
    String,
    Vec<String>,
  )> = Vec::new();

  for bos_sessiontemplate in bos_sessiontemplate_value_vec {
    let cfs_configuration_opt: Option<String> = bos_sessiontemplate
      .clone()
      .cfs
      .and_then(|cfs| cfs.configuration);

    // FIXME: use BosSessionTemplate.get_image_vec() to get the list of image ids
    let first_image_id_opt: Option<String> = bos_sessiontemplate
      .get_path_vec()
      .first()
      .and_then(|v| v.strip_prefix("s3://boot-images/"))
      .and_then(|v| v.strip_suffix("/manifest.json").map(str::to_string));

    if let (Some(cfs_configuration), Some(first_image_id)) =
      (cfs_configuration_opt, first_image_id_opt)
    {
      let target = [
        bos_sessiontemplate.get_target_hsm(),
        bos_sessiontemplate.get_target_xname(),
      ]
      .concat();

      image_id_cfs_configuration_from_bos_sessiontemplate.push((
        first_image_id.to_string(),
        cfs_configuration.to_string(),
        target,
      ));
    } else {
      log::warn!(
        "BOS sessiontemplate '{:?}' not valid, check fields 'path' and 'cfs.configuration' have valid values. Path field should have 's3://boot-images/' as prefix and '/manifest.json' as sufix",
        bos_sessiontemplate.name
      );
    }
  }

  image_id_cfs_configuration_from_bos_sessiontemplate
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::bos::template::http_client::v2::types::{BootSet, Cfs};
  use std::collections::HashMap;

  fn template(
    name: &str,
    cfs_config: Option<&str>,
    boot_sets: Vec<(&str, BootSet)>,
  ) -> BosSessionTemplate {
    let mut map = HashMap::new();
    for (k, v) in boot_sets {
      map.insert(k.to_string(), v);
    }
    BosSessionTemplate {
      name: Some(name.to_string()),
      description: None,
      enable_cfs: None,
      cfs: cfs_config.map(|c| Cfs {
        configuration: Some(c.to_string()),
      }),
      boot_sets: if map.is_empty() { None } else { Some(map) },
      links: None,
      tenant: None,
    }
  }

  fn boot_set_for_hsm(node_groups: Vec<&str>) -> BootSet {
    BootSet {
      name: None,
      path: None,
      r#type: None,
      etag: None,
      kernel_parameters: None,
      cfs: None,
      node_list: None,
      node_roles_groups: None,
      node_groups: Some(node_groups.iter().map(|s| s.to_string()).collect()),
      rootfs_provider: None,
      rootfs_provider_passthrough: None,
      arch: None,
    }
  }

  fn boot_set_for_xnames(node_list: Vec<&str>) -> BootSet {
    BootSet {
      name: None,
      path: None,
      r#type: None,
      etag: None,
      kernel_parameters: None,
      cfs: None,
      node_list: Some(node_list.iter().map(|s| s.to_string()).collect()),
      node_roles_groups: None,
      node_groups: None,
      rootfs_provider: None,
      rootfs_provider_passthrough: None,
      arch: None,
    }
  }

  // ---------- filter: configuration name pattern ----------

  #[test]
  fn filter_glob_keeps_only_matching_configs() {
    let mut templates = vec![
      template("t1", Some("zinal-1.2.3"), vec![]),
      template("t2", Some("daint-1.2.3"), vec![]),
      template("t3", Some("zinal-2.0.0"), vec![]),
    ];

    let result =
      filter(&mut templates, Some("zinal-*"), &[], &[], None).unwrap();
    let names: Vec<&str> = result
      .iter()
      .map(|t| t.configuration_name().unwrap())
      .collect();
    assert_eq!(names, vec!["zinal-1.2.3", "zinal-2.0.0"]);
  }

  #[test]
  fn filter_glob_drops_templates_without_configuration() {
    let mut templates = vec![
      template("t1", Some("zinal"), vec![]),
      template("t2", None, vec![]),
    ];

    let result =
      filter(&mut templates, Some("zinal*"), &[], &[], None).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].configuration_name(), Some("zinal"));
  }

  // ---------- filter: HSM groups / xnames ----------

  #[test]
  fn filter_by_hsm_keeps_templates_whose_node_groups_all_match() {
    let mut templates = vec![
      template(
        "t1",
        None,
        vec![("compute", boot_set_for_hsm(vec!["zinal"]))],
      ),
      template(
        "t2",
        None,
        vec![("compute", boot_set_for_hsm(vec!["daint"]))],
      ),
    ];

    let hsm = vec!["zinal".to_string()];
    let result = filter(&mut templates, None, &hsm, &[], None).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name.as_deref(), Some("t1"));
  }

  #[test]
  fn filter_by_hsm_uses_contains_not_eq() {
    // The HSM filter uses `.contains()` (substring), so "zinal" matches
    // both "zinal" and "zinal-cn".
    let mut templates = vec![template(
      "t1",
      None,
      vec![("compute", boot_set_for_hsm(vec!["zinal-cn"]))],
    )];

    let hsm = vec!["zinal".to_string()];
    let result = filter(&mut templates, None, &hsm, &[], None).unwrap();
    assert_eq!(result.len(), 1);
  }

  #[test]
  fn filter_by_xname_keeps_templates_whose_node_list_all_match() {
    let mut templates = vec![
      template(
        "t1",
        None,
        vec![("compute", boot_set_for_xnames(vec!["x1000c0s0b0n0"]))],
      ),
      template(
        "t2",
        None,
        vec![("compute", boot_set_for_xnames(vec!["x9999c0s0b0n0"]))],
      ),
    ];

    let xnames = vec!["x1000c0s0b0n0".to_string()];
    let result = filter(&mut templates, None, &[], &xnames, None).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name.as_deref(), Some("t1"));
  }

  #[test]
  fn filter_drops_templates_with_empty_targets_when_filter_active() {
    // A template with no node_groups AND no node_list is dropped when any
    // hsm/xname filter is provided.
    let mut templates = vec![
      template("empty", None, vec![]),
      template(
        "with-target",
        None,
        vec![("compute", boot_set_for_hsm(vec!["zinal"]))],
      ),
    ];

    let hsm = vec!["zinal".to_string()];
    let result = filter(&mut templates, None, &hsm, &[], None).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name.as_deref(), Some("with-target"));
  }

  // ---------- filter: limit ----------

  #[test]
  fn filter_limit_keeps_the_last_n_after_filtering() {
    // Limit slices from the tail (preserves the most recent entries given
    // an externally-sorted input).
    let mut templates = vec![
      template("t1", None, vec![]),
      template("t2", None, vec![]),
      template("t3", None, vec![]),
      template("t4", None, vec![]),
    ];

    let limit: u8 = 2;
    let result = filter(&mut templates, None, &[], &[], Some(&limit)).unwrap();
    assert_eq!(result.len(), 2);
    let names: Vec<&str> =
      result.iter().filter_map(|t| t.name.as_deref()).collect();
    assert_eq!(names, vec!["t3", "t4"]);
  }

  #[test]
  fn filter_limit_larger_than_input_keeps_everything() {
    let mut templates = vec![template("t1", None, vec![])];
    let limit: u8 = 10;
    let result = filter(&mut templates, None, &[], &[], Some(&limit)).unwrap();
    assert_eq!(result.len(), 1);
  }

  // ---------- get_image_id_cfs_configuration_target_tuple_vec ----------

  fn boot_set_with_path_and_groups(
    path: &str,
    node_groups: Vec<&str>,
  ) -> BootSet {
    BootSet {
      name: None,
      path: Some(path.to_string()),
      r#type: None,
      etag: None,
      kernel_parameters: None,
      cfs: None,
      node_list: None,
      node_roles_groups: None,
      node_groups: Some(node_groups.iter().map(|s| s.to_string()).collect()),
      rootfs_provider: None,
      rootfs_provider_passthrough: None,
      arch: None,
    }
  }

  #[test]
  fn tuple_vec_extracts_image_id_config_and_targets() {
    let template = BosSessionTemplate {
      name: Some("t1".into()),
      description: None,
      enable_cfs: None,
      cfs: Some(Cfs {
        configuration: Some("zinal-config".into()),
      }),
      boot_sets: Some({
        let mut m = HashMap::new();
        m.insert(
          "compute".to_string(),
          boot_set_with_path_and_groups(
            "s3://boot-images/img-uuid-1/manifest.json",
            vec!["zinal"],
          ),
        );
        m
      }),
      links: None,
      tenant: None,
    };

    let result =
      get_image_id_cfs_configuration_target_tuple_vec(&vec![template]);
    assert_eq!(result.len(), 1);
    let (image_id, config, targets) = &result[0];
    assert_eq!(image_id, "img-uuid-1");
    assert_eq!(config, "zinal-config");
    assert_eq!(targets, &vec!["zinal".to_string()]);
  }

  #[test]
  fn tuple_vec_skips_templates_with_bad_path_format() {
    // Path doesn't have s3:// prefix and manifest.json suffix — skipped.
    let template = BosSessionTemplate {
      name: Some("bad".into()),
      description: None,
      enable_cfs: None,
      cfs: Some(Cfs {
        configuration: Some("c".into()),
      }),
      boot_sets: Some({
        let mut m = HashMap::new();
        m.insert(
          "compute".to_string(),
          boot_set_with_path_and_groups("http://elsewhere/foo", vec!["zinal"]),
        );
        m
      }),
      links: None,
      tenant: None,
    };

    let result =
      get_image_id_cfs_configuration_target_tuple_vec(&vec![template]);
    assert!(result.is_empty());
  }
}
