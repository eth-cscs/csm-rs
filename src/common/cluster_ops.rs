use crate::cfs::{
    configuration::csm_rs::r#struct::cfs_configuration_response::v2::CfsConfigurationResponse,
    session::csm_rs::r#struct::v2::CfsSessionGetResponse,
};

#[derive(Debug)]
pub struct ClusterDetails {
    pub hsm_group_label: String,
    pub most_recent_cfs_configuration_name_created: CfsConfigurationResponse,
    pub most_recent_cfs_session_name_created: CfsSessionGetResponse,
    pub members: Vec<String>,
}

pub async fn get_details(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name: &str,
) -> Vec<ClusterDetails> {
    let mut clusters_details = vec![];

    // Get HSM groups matching cluster name
    let hsm_group_value_vec =
        crate::hsm::group::http_client::get_hsm_group_without_system_wide_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(&hsm_group_name.to_string()),
        )
        .await
        .unwrap();

    for hsm_group in hsm_group_value_vec {
        let hsm_group_name = hsm_group.label.as_str();

        let hsm_group_members: String =
            crate::hsm::group::utils::get_member_vec_from_hsm_group(&hsm_group).join(",");

        // Get all CFS sessions
        let mut cfs_session_vec = crate::cfs::session::csm_rs::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
            None,
            None,
            None,
            Some(true),
        )
        .await
        .unwrap();

        crate::cfs::session::csm_rs::utils::filter_by_hsm(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &mut cfs_session_vec,
            &[hsm_group_name.to_string()],
            None,
            true,
        )
        .await;

        let most_recent_cfs_session;
        let cfs_configuration;

        for cfs_session_value in cfs_session_vec {
            // println!("cfs_session_value:\n{:#?}", cfs_session_value);
            let target_groups = cfs_session_value
                .target
                .as_ref()
                .unwrap()
                .groups
                .as_ref()
                .unwrap();
            let ansible_limit = cfs_session_value
                .ansible
                .as_ref()
                .unwrap()
                .limit
                .as_ref()
                .unwrap();

            // Check CFS session is linkged to HSM GROUP name or any of its members
            if target_groups
                .iter()
                .map(|target_group| target_group.name.as_ref())
                .collect::<Vec<&str>>()
                .contains(&hsm_group_name)
                || ansible_limit.contains(&hsm_group_members)
            {
                most_recent_cfs_session = cfs_session_value;

                // Get CFS configuration linked to CFS session related to HSM GROUP or any of its
                // members
                let cfs_configuration_vec = crate::cfs::configuration::csm_rs::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    Some(
                        &most_recent_cfs_session
                            .configuration
                            .as_ref()
                            .unwrap()
                            .name
                            .clone()
                            .unwrap(),
                    ),
                )
                .await
                .unwrap();

                cfs_configuration = cfs_configuration_vec.first().unwrap();

                let cluster_details = ClusterDetails {
                    hsm_group_label: hsm_group_name.to_string(),
                    most_recent_cfs_configuration_name_created: cfs_configuration.clone(),
                    most_recent_cfs_session_name_created: most_recent_cfs_session,
                    members: hsm_group
                        .members
                        .as_ref()
                        .unwrap()
                        .ids
                        .as_ref()
                        .unwrap()
                        .clone(),
                };

                clusters_details.push(cluster_details);

                break;
            }
        }
    }

    clusters_details
}
