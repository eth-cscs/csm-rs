use std::collections::BTreeMap;

use crate::{common::kubernetes, error::Error};

/// Fetch the `cray-product-catalog` ConfigMap from the in-cluster
/// `services` namespace and return its `.data` map.
///
/// Thin wrapper over [`crate::common::kubernetes::try_get_configmap`]
/// that fixes the well-known ConfigMap name; SAT-file apply consults
/// this catalog to validate product layers against installed versions.
pub async fn get_products(
  kube_client: kube::Client,
) -> Result<BTreeMap<String, String>, Error> {
  kubernetes::try_get_configmap(kube_client, "cray-product-catalog").await
}
