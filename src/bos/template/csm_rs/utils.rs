use crate::bos::template::csm_rs::r#struct::v2::BosSessionTemplate;

pub async fn filter(
    bos_sessiontemplate_vec: &mut Vec<BosSessionTemplate>,
    hsm_group_available_vec: &[String],
    xname_available_vec: &[String],
    // cfs_configuration_name_opt: Option<&str>,
    limit_number_opt: Option<&u8>,
) -> Vec<BosSessionTemplate> {
    log::info!("Filter BOS sessiontemplates");
    // Filter by list of HSM group or xnames as target
    if !hsm_group_available_vec.is_empty() || !xname_available_vec.is_empty() {
        bos_sessiontemplate_vec.retain(|bos_sessiontemplate| {
            let bos_sessiontemplate_target_hsm = bos_sessiontemplate.get_target_hsm();
            let bos_sessiontemplate_target_xname = bos_sessiontemplate.get_target_xname();

            !bos_sessiontemplate_target_hsm.is_empty()
                && bos_sessiontemplate_target_hsm.iter().all(|target_hsm| {
                    hsm_group_available_vec
                        .iter()
                        .any(|hsm_group_available| target_hsm.contains(hsm_group_available))
                })
                || !bos_sessiontemplate_target_xname.is_empty()
                    && bos_sessiontemplate_target_xname
                        .iter()
                        .all(|target_xname| xname_available_vec.contains(target_xname))
        });
    }

    if let Some(limit_number) = limit_number_opt {
        // Limiting the number of results to return to client
        *bos_sessiontemplate_vec = bos_sessiontemplate_vec[bos_sessiontemplate_vec
            .len()
            .saturating_sub(*limit_number as usize)..]
            .to_vec();
    }

    bos_sessiontemplate_vec.to_vec()
}

pub async fn filter_by_configuration(
    bos_sessiontemplate_vec: &mut Vec<BosSessionTemplate>,
    cfs_configuration_name: &str,
) {
    bos_sessiontemplate_vec.retain(|bos_template| {
        bos_template.get_confguration().as_deref() == Some(cfs_configuration_name)
    });
}

pub fn get_image_id_cfs_configuration_target_tuple_vec(
    bos_sessiontemplate_value_vec: Vec<BosSessionTemplate>,
) -> Vec<(String, String, Vec<String>)> {
    let mut image_id_cfs_configuration_from_bos_sessiontemplate: Vec<(
        String,
        String,
        Vec<String>,
    )> = Vec::new();

    for bos_sessiontemplate in bos_sessiontemplate_value_vec {
        if bos_sessiontemplate.cfs.is_none() {
            continue;
        }

        let cfs_configuration = bos_sessiontemplate
            .cfs
            .as_ref()
            .unwrap()
            .configuration
            .as_ref()
            .unwrap();

        let path = bos_sessiontemplate
            .get_path_vec()
            .first()
            .unwrap()
            .strip_prefix("s3://boot-images/")
            .unwrap()
            .strip_suffix("/manifest.json")
            .unwrap()
            .to_string();

        let target = [
            bos_sessiontemplate.get_target_hsm(),
            bos_sessiontemplate.get_target_xname(),
        ]
        .concat();

        image_id_cfs_configuration_from_bos_sessiontemplate.push((
            path.to_string(),
            cfs_configuration.to_string(),
            target,
        ));

        /* for boot_set in bos_sessiontemplate.boot_sets.as_ref().unwrap().values() {
            let path = boot_set
                .path
                .as_ref()
                .unwrap()
                .strip_prefix("s3://boot-images/")
                .unwrap()
                .strip_suffix("/manifest.json")
                .unwrap()
                .to_string();

            let target: Vec<String> = if let Some(node_groups) = boot_set.node_groups.as_ref() {
                node_groups
                    .iter()
                    .map(|node_group| node_group.to_string())
                    .collect()
            } else if let Some(node_list) = boot_set.node_list.as_ref() {
                node_list
                    .iter()
                    .map(|target_group| target_group.to_string())
                    .collect()
            } else {
                vec![]
            };

            image_id_cfs_configuration_from_bos_sessiontemplate.push((
                path.to_string(),
                cfs_configuration.to_string(),
                target,
            ));
        } */
    }

    image_id_cfs_configuration_from_bos_sessiontemplate
}

/* pub fn get_cfs_configuration_name(bos_sessiontemplate: &BosSessionTemplate) -> Option<String> {
    bos_sessiontemplate
        .cfs
        .as_ref()
        .unwrap()
        .configuration
        .as_ref()
        .cloned()
} */

pub fn find_bos_sessiontemplate_related_to_image_id(
    bos_sessiontemplate_vec: &[BosSessionTemplate],
    image_id: &str,
) -> Option<BosSessionTemplate> {
    bos_sessiontemplate_vec
        .iter()
        .find(|bos_sessiontemplate| {
            bos_sessiontemplate
                .get_path_vec()
                .first()
                .unwrap()
                .contains(image_id)
            /* bos_sessiontemplate
            .boot_sets
            .as_ref()
            .unwrap()
            .values()
            .next()
            .as_ref()
            .unwrap()
            .path
            .as_ref()
            .unwrap()
            .contains(image_id) */
        })
        .cloned()
}
