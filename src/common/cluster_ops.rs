//! Generic cluster-scoped helpers used by the commands layer.

use crate::error::Error;

/// Members of one HSM group matched by [`get_details`].
#[derive(Debug)]
#[allow(missing_docs)]
pub struct ClusterDetails {
  pub members: Vec<String>,
}

/// List the member xnames for every HSM group whose label contains
/// `hsm_group_name` (substring match).
pub async fn get_details(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  hsm_group_name: &str,
) -> Result<Vec<ClusterDetails>, Error> {
  let hsm_group_value_vec = crate::ShastaClient::new(
    shasta_base_url,
    shasta_root_cert.to_vec(),
    socks5_proxy.map(str::to_owned),
  )?
  .hsm_group_get_hsm_group_vec(shasta_token, Some(&hsm_group_name.to_string()))
  .await?;

  Ok(
    hsm_group_value_vec
      .into_iter()
      .map(|hsm_group| ClusterDetails {
        members: crate::hsm::group::utils::get_member_vec_from_hsm_group(
          &hsm_group,
        ),
      })
      .collect(),
  )
}
