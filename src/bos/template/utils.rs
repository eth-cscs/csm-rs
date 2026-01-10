use crate::{
  bos::template::http_client::v2::types::BosSessionTemplate, error::Error,
};
use globset::Glob;

pub fn filter(
  bos_sessiontemplate_vec: &mut Vec<BosSessionTemplate>,
  configuration_name_pattern_opt: Option<&str>,
  target_hsm_group_name_vec: &[&str],
  xname_vec: &[&str],
  // cfs_configuration_name_opt: Option<&str>,
  limit_number_opt: Option<&u8>,
) -> Result<Vec<BosSessionTemplate>, Error> {
  log::info!("Filter BOS sessiontemplates");

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
            .all(|target_xname| xname_vec.contains(&target_xname.as_str()))
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

pub async fn filter_by_configuration(
  bos_sessiontemplate_vec: &mut Vec<BosSessionTemplate>,
  cfs_configuration_name: &str,
) {
  bos_sessiontemplate_vec.retain(|bos_template| {
    bos_template.get_configuration().as_deref() == Some(cfs_configuration_name)
  });
}

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
      log::warn!("BOS sessiontemplate '{:?}' not valid, check fields 'path' and 'cfs.configuration' have valid values. Path field should have 's3://boot-images/' as prefix and '/manifest.json' as sufix", bos_sessiontemplate.name);
    }
  }

  image_id_cfs_configuration_from_bos_sessiontemplate
}
