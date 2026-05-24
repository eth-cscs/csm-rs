use std::collections::{BTreeMap, HashMap};

use serde_json::Map;
use serde_yaml::Value;
use uuid::Uuid;

use crate::{
  cfs::{
    self,
    configuration::http_client::v2::types::cfs_configuration_response::CfsConfigurationResponse,
    session::http_client::v2::types::CfsSessionPostRequest,
  },
  error::Error,
  hsm,
  ims::{self},
};

use super::{
  configuration,
  image::{self, Filter},
  session_templates::get_base_image_id_from_sat_file_image_yaml,
};

/// Analyze a list of images in SAT file and returns the image to process next.
/// Input values:
///  - image_yaml_vec: the list of images in the SAT file, each element is a serde_yaml::Value
///  - ref_name_processed_vec: he list of images (ref_name) already processed
/// Note:
/// image.base.image_ref value in SAT file points to the image it depends on (image.ref_name)
/// NOTE 2: we assume that there may be a mix of images in SAT file with and without "ref_name"
/// value, we will use the function "get_ref_name" which will fall back to "name" field if
/// "ref_name" is missing in the image
/// An image is ready to be processed if:
///  - It does not depends on another image (image.base.image_ref is missing)
///  - The image it depends to is already processed (image.base.image_ref included in
///  ref_name_processed)
///  - It has not been already processed
#[deprecated(
  since = "0.100.0",
  note = "Please use 'get_next_image_in_sat_file_to_process_struct' istead"
)]
pub fn get_next_image_in_sat_file_to_process(
  image_yaml_vec: &[serde_yaml::Value],
  ref_name_processed_vec: &[String],
) -> Option<serde_yaml::Value> {
  image_yaml_vec
    .iter()
    .find(|image_yaml| {
      #[allow(deprecated)]
      let ref_name: &str = &get_image_name_or_ref_name_to_process(image_yaml); // Again, because we assume images in
      // SAT file may or may not have ref_name value, we will use "get_ref_name" function to
      // get the id of the image

      let image_base_image_ref_opt: Option<&str> = image_yaml
        .get("base")
        .and_then(|image_base_yaml| image_base_yaml.get("image_ref"))
        .and_then(Value::as_str);

      !ref_name_processed_vec.contains(&ref_name.to_string())
        && (image_base_image_ref_opt.is_none()
          || image_base_image_ref_opt.is_some_and(|image_base_image_ref| {
            ref_name_processed_vec.contains(&image_base_image_ref.to_string())
          }))
    })
    .cloned()
}

/// Analyze a list of images in SAT file and returns the image to process next.
/// Input values:
///  - image_yaml_vec: the list of images in the SAT file, each element is a serde_yaml::Value
///  - ref_name_processed_vec: he list of images (ref_name) already processed
/// Note:
/// image.base.image_ref value in SAT file points to the image it depends on (image.ref_name)
/// NOTE 2: we assume that there may be a mix of images in SAT file with and without "ref_name"
/// value, we will use the function "get_ref_name" which will fall back to "name" field if
/// "ref_name" is missing in the image
/// An image is ready to be processed if:
///  - It does not depends on another image (image.base.image_ref is missing)
///  - The image it depends to is already processed (image.base.image_ref included in
///  ref_name_processed)
///  - It has not been already processed
pub fn get_next_image_in_sat_file_to_process_struct(
  image_yaml_vec: &[image::Image],
  ref_name_processed_vec: &[String],
) -> Option<image::Image> {
  image_yaml_vec
    .iter()
    .find(|image_yaml| {
      let ref_name: &str =
        &get_image_name_or_ref_name_to_process_struct(image_yaml); // Again, because we assume images in
      // SAT file may or may not have ref_name value, we will use "get_ref_name" function to
      // get the id of the image

      /* let image_base_image_ref_opt: Option<&str> = image_yaml
      .get("base")
      .and_then(|image_base_yaml| image_base_yaml.get("image_ref"))
      .and_then(Value::as_str); */

      let image_base_image_ref_opt = if let image::BaseOrIms::Base {
        base: image::Base::ImageRef { image_ref },
      } = &image_yaml.base_or_ims
      {
        Some(image_ref)
      } else {
        None
      };

      !ref_name_processed_vec.contains(&ref_name.to_string())
        && (image_base_image_ref_opt.is_none()
          || image_base_image_ref_opt.is_some_and(|image_base_image_ref| {
            ref_name_processed_vec.contains(&image_base_image_ref.to_string())
          }))
    })
    .cloned()
}

/// Get the "ref_name" from an image, because we need to be aware of which images in SAT file have
/// been processed in order to find the next image to process. We assume not all images in the yaml
/// will have an "image_ref" value, therefore we will use "ref_name" or "name" field if the former
/// is missing
#[deprecated(
  since = "0.100.0",
  note = "Please use 'get_image_name_or_ref_name_to_process_struct' istead"
)]
pub fn get_image_name_or_ref_name_to_process(
  image_yaml: &serde_yaml::Value,
) -> String {
  // Prefer ref_name, fall back to name. If neither is a string, return empty
  // (this fn is deprecated; the struct version is preferred).
  image_yaml
    .get("ref_name")
    .and_then(Value::as_str)
    .or_else(|| image_yaml.get("name").and_then(Value::as_str))
    .map(str::to_string)
    .unwrap_or_default()
}

/// Get the "ref_name" from an image, because we need to be aware of which images in SAT file have
/// been processed in order to find the next image to process. We assume not all images in the yaml
/// will have an "image_ref" value, therefore we will use "ref_name" or "name" field if the former
/// is missing
pub fn get_image_name_or_ref_name_to_process_struct(
  image_yaml: &image::Image,
) -> String {
  if let Some(ref_name) = image_yaml.ref_name.as_ref() {
    ref_name.clone()
  } else {
    // If the image processed is missing the field "ref_name", then use the field "name"
    // instead, this is needed to flag this image as processed and filtered when
    // calculating the next image to process (get_next_image_to_process)
    image_yaml.name.clone()
  }
}

#[deprecated(
  since = "0.86.2",
  note = "this function prints cfs session logs to stdout"
)]
pub async fn i_import_images_section_in_sat_file(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  vault_base_url: &str,
  site_name: &str,
  k8s_api_url: &str,
  ref_name_processed_hashmap: &mut HashMap<String, String>,
  // image_yaml_vec: &[serde_yaml::Value],
  image_yaml_vec: &[image::Image],
  cray_product_catalog: &BTreeMap<String, String>,
  ansible_verbosity_opt: Option<u8>,
  ansible_passthrough_opt: Option<&str>,
  debug_on_failure: bool, // tag: &str,
  dry_run: bool,
  watch_logs: bool,
  timestamps: bool,
) -> Result<HashMap<String, image::Image>, Error> {
  if image_yaml_vec.is_empty() {
    log::warn!("No images found in SAT file. Nothing to process.");
    return Ok(HashMap::new());
  }

  // Get an image to process (the image either has no dependency or it's image dependency has
  // already ben processed)
  let mut next_image_to_process_opt: Option<image::Image> =
    get_next_image_in_sat_file_to_process_struct(
      image_yaml_vec,
      &ref_name_processed_hashmap
        .keys()
        .cloned()
        .collect::<Vec<String>>(),
    );

  // Process images
  log::info!("Processing image '{:?}'", next_image_to_process_opt);
  let mut image_processed_hashmap: HashMap<String, image::Image> =
    HashMap::new();

  while let Some(image_yaml) = &next_image_to_process_opt {
    #[allow(deprecated)]
    let image_id = i_create_image_from_sat_file_serde_yaml(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      vault_base_url,
      site_name,
      k8s_api_url,
      image_yaml,
      cray_product_catalog,
      ansible_verbosity_opt,
      ansible_passthrough_opt,
      ref_name_processed_hashmap,
      debug_on_failure,
      dry_run,
      watch_logs,
      timestamps,
    )
    .await?;

    image_processed_hashmap.insert(image_id.clone(), image_yaml.clone());

    ref_name_processed_hashmap.insert(
      get_image_name_or_ref_name_to_process_struct(image_yaml),
      image_id.clone(),
    );

    next_image_to_process_opt = get_next_image_in_sat_file_to_process_struct(
      image_yaml_vec,
      &ref_name_processed_hashmap
        .keys()
        .cloned()
        .collect::<Vec<String>>(),
    );
  }

  Ok(image_processed_hashmap)
}

#[deprecated(
  since = "0.86.2",
  note = "this function prints cfs session logs to stdout"
)]
pub async fn i_create_image_from_sat_file_serde_yaml(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  vault_base_url: &str,
  site_name: &str,
  k8s_api_url: &str,
  // image_yaml: &serde_yaml::Value, // NOTE: image may be an IMS job or a CFS session
  image_yaml: &image::Image,
  cray_product_catalog: &BTreeMap<String, String>,
  ansible_verbosity_opt: Option<u8>,
  ansible_passthrough_opt: Option<&str>,
  ref_name_image_id_hashmap: &HashMap<String, String>,
  _debug_on_failure: bool,
  dry_run: bool,
  watch_logs: bool,
  timestamps: bool,
) -> Result<String, Error> {
  // Get CFS session from SAT file image yaml
  let cfs_session = get_session_from_image_yaml(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    socks5_proxy,
    image_yaml,
    ref_name_image_id_hashmap,
    cray_product_catalog,
    ansible_verbosity_opt,
    ansible_passthrough_opt,
    dry_run,
  )
  .await?;

  let image_name = &image_yaml.name;

  // Create CFS session to build image
  if !dry_run {
    #[allow(deprecated)]
    let cfs_session_rslt = cfs::session::i_post_sync(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      vault_base_url,
      site_name,
      k8s_api_url,
      socks5_proxy,
      &cfs_session,
      watch_logs,
      timestamps,
    )
    .await;

    let cfs_session = match cfs_session_rslt {
      Ok(cfs_session) => cfs_session,
      Err(e) => {
        return Err(Error::Message(format!(
          "Could not create Image. Reason:\n{}",
          e
        )));
      }
    };

    if !cfs_session.is_success() {
      return Err(Error::Message(format!(
        "CFS session '{}' failed. Exit",
        cfs_session.name
      )));
    }

    let image_id = cfs_session.first_result_id().unwrap_or_default();
    log::info!("Image '{}' ({}) created", image_name, image_id);

    Ok(image_id.to_string())
  } else {
    log::info!(
      "Dry run mode: Create CFS session:\n{}",
      serde_json::to_string_pretty(&cfs_session)?
    );

    let image_id = Uuid::new_v4().to_string();

    log::info!(
      "Dry run mode: Image '{}' ({}) created",
      image_name, image_id
    );

    Ok(image_id)
  }
}

async fn get_session_from_image_yaml(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  // image_yaml: Value,
  image_yaml: &image::Image,
  ref_name_image_id_hashmap: &HashMap<String, String>,
  cray_product_catalog: &BTreeMap<String, String>,
  ansible_verbosity_opt: Option<u8>,
  ansible_passthrough_opt: Option<&str>,
  dry_run: bool,
) -> Result<CfsSessionPostRequest, Error> {
  // Collect CFS session details from SAT file
  // Get CFS image name from SAT file
  let image_name = image_yaml.name.clone();

  log::info!(
    "Creating CFS session related to build image '{}'",
    image_name
  );

  // Get CFS configuration related to CFS session in SAT file
  let configuration_name =
    image_yaml.configuration.as_ref().ok_or_else(|| {
      Error::Message(format!(
        "SAT file: image '{}' is missing 'configuration' field",
        image_yaml.name
      ))
    })?;

  // Get HSM groups related to CFS session in SAT file
  let groups_name: Vec<&str> = image_yaml
    .configuration_group_names
    .as_ref()
    .map(|group_name_vec| group_name_vec.iter().map(String::as_str).collect())
    .unwrap_or_default();

  // VALIDATION: make sure grups in SAT.images "CFS session" are valid
  // NOTE: this is temporary until we get rid off "group" names as ansible folder names
  let invalid_groups: Vec<String> =
    hsm::group::hacks::validate_groups_auth_token(&groups_name, shasta_token)?;

  if !invalid_groups.is_empty() {
    log::debug!("CFS session group validation - failed");

    return Err(Error::Message(format!(
      "Please fix 'images' section in SAT file.\nInvalid groups: {:?}",
      invalid_groups
    )));
  } else {
    log::debug!("CFS session group validation - passed");
  }

  let base_image_id = get_base_image_id_from_sat_file_image_yaml(
    shasta_token,
    shasta_base_url,
    shasta_root_cert,
    socks5_proxy,
    image_yaml,
    ref_name_image_id_hashmap,
    cray_product_catalog,
    &image_name,
    dry_run,
  )
  .await?;

  // Create a CFS session
  log::info!("Creating CFS session");

  // Create CFS session
  let session_name = image_name.clone();

  let cfs_session = CfsSessionPostRequest::new(
    session_name,
    configuration_name,
    None,
    ansible_verbosity_opt,
    ansible_passthrough_opt,
    true,
    Some(&groups_name),
    Some(&base_image_id),
  );

  Ok(cfs_session)
}

pub(super) async fn process_sat_file_image_product_type_ims_recipe(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  recipe_id: &str,
  image_name: &str,
  dry_run: bool,
) -> Result<String, Error> {
  let root_ims_key_name = "mgmt root key";

  // Get root public ssh key
  let root_public_ssh_key_value: serde_json::Value =
    ims::public_keys::http_client::v3::get_single(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      root_ims_key_name,
    )
    .await?
    .ok_or_else(|| Error::ImsKeyNotFound(root_ims_key_name.to_string()))?;

  let root_public_ssh_key: &str = root_public_ssh_key_value
    .get("id")
    .and_then(serde_json::Value::as_str)
    .ok_or_else(|| {
      Error::Message(
        "IMS public-key response is missing or has non-string 'id'".to_string(),
      )
    })?;

  // let ims_job = ims::job::types::JobPostRequest {
  let ims_job = ims::job::types::Job {
    job_type: "create".to_string(),
    image_root_archive_name: image_name.to_string(),
    kernel_file_name: Some("vmlinuz".to_string()),
    initrd_file_name: Some("initrd".to_string()),
    kernel_parameters_file_name: Some("kernel-parameters".to_string()),
    artifact_id: recipe_id.to_string(),
    public_key_id: root_public_ssh_key.to_string(),
    ssh_containers: None, // Should this be None ???
    enable_debug: Some(false),
    build_env_size: Some(15),
    require_dkms: None, // FIXME: check SAT file and see if this value needs to be set
    id: None,
    created: None,
    status: None,
    kubernetes_job: None,
    kubernetes_service: None,
    kubernetes_configmap: None,
    resultant_image_id: None,
    kubernetes_namespace: None,
    arch: None,
  };

  let ims_job = if dry_run {
    log::info!(
      "Dry run mode: Create IMS job:\n{}",
      serde_json::to_string_pretty(&ims_job)?
    );
    let mut dry_run_ims_job = ims_job;
    dry_run_ims_job.resultant_image_id = Some(Uuid::new_v4().to_string());
    dry_run_ims_job
  } else {
    ims::job::http_client::post_sync(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      &ims_job,
    )
    .await?
  };

  ims_job.resultant_image_id.ok_or_else(|| {
    Error::Message(format!(
      "IMS job for image '{}' did not produce a resultant_image_id",
      image_name
    ))
  })
}


pub(super) async fn process_sat_file_image_ims_type_recipe(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
  recipe_name: &str,
  image_name: &String,
  dry_run: bool,
) -> Result<String, Error> {
  // Base image needs to be created from a IMS job using an IMS recipe
  // Get all IMS recipes
  let recipe_detail_vec: Vec<ims::recipe::types::RecipeGetResponse> =
    crate::ShastaClient::new(
      shasta_base_url,
      shasta_token,
      shasta_root_cert.to_vec(),
      socks5_proxy.map(str::to_owned),
    )?
    .ims_recipe_get(None)
    .await?;

  // Filter recipes by name
  let recipe_detail_opt = recipe_detail_vec
    .iter()
    .find(|recipe| recipe.name == recipe_name);

  log::info!("IMS recipe details:\n{:#?}", recipe_detail_opt);

  // Check recipe with requested name exists
  let recipe_detail = recipe_detail_opt.ok_or_else(|| {
    Error::Message(format!(
      "IMS recipe with name '{}' - not found. Exit",
      recipe_name
    ))
  })?;
  let recipe_id = recipe_detail.id.as_ref().ok_or_else(|| {
    Error::Message(format!(
      "IMS recipe '{}' has no 'id' field",
      recipe_name
    ))
  })?;

  log::info!("IMS recipe id found '{}'", recipe_id);

  let root_ims_key_name = "mgmt root key";

  // Get root public ssh key
  let root_public_ssh_key_value: serde_json::Value =
    ims::public_keys::http_client::v3::get_single(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      root_ims_key_name,
    )
    .await?
    .ok_or_else(|| Error::ImsKeyNotFound(root_ims_key_name.to_string()))?;

  let root_public_ssh_key = root_public_ssh_key_value
    .get("id")
    .and_then(serde_json::Value::as_str)
    .ok_or_else(|| {
      Error::Message(
        "IMS public-key response is missing or has non-string 'id'".to_string(),
      )
    })?;

  let ims_job = ims::job::types::Job {
    job_type: "create".to_string(),
    image_root_archive_name: image_name.to_string(),
    kernel_file_name: Some("vmlinuz".to_string()),
    initrd_file_name: Some("initrd".to_string()),
    kernel_parameters_file_name: Some("kernel-parameters".to_string()),
    artifact_id: recipe_id.to_string(),
    public_key_id: root_public_ssh_key.to_string(),
    ssh_containers: None, // Should this be None ???
    enable_debug: Some(false),
    build_env_size: Some(15),
    require_dkms: None, // FIXME: check SAT file and see if this value needs to be set
    id: None,
    created: None,
    status: None,
    kubernetes_job: None,
    kubernetes_service: None,
    kubernetes_configmap: None,
    resultant_image_id: None,
    kubernetes_namespace: None,
    arch: None,
  };

  let ims_job = if dry_run {
    log::info!(
      "Dry run mode: Create IMS job:\n{}",
      serde_json::to_string_pretty(&ims_job)?
    );
    ims_job
  } else {
    ims::job::http_client::post_sync(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      &ims_job,
    )
    .await?
  };

  log::info!("IMS job response:\n{:#?}", ims_job);

  ims_job.resultant_image_id.ok_or_else(|| {
    Error::Message(format!(
      "IMS job for image '{}' did not produce a resultant_image_id",
      image_name
    ))
  })
}

pub(super) fn process_sat_file_image_old_version_struct(
  sat_file_image_ims_value_yaml: &image::ImageIms,
) -> Result<String, Error> {
  if let image::ImageIms::IdIsRecipe {
    id,
    is_recipe: false,
  } = &sat_file_image_ims_value_yaml
  {
    Ok(id.to_string())
  } else {
    Err(Error::Message("Functionality not built. Exit".to_string()))
  }
}

pub fn filter_product_catalog_images(
  filter: &Filter,
  image_map: Map<String, serde_json::Value>,
  image_name: &str,
) -> Result<String, Error> {
  if let Filter::Arch { arch } = filter {
    // Search image in product catalog and filter by arch
    let image_key_vec = image_map
      .keys()
      .collect::<Vec<_>>()
      .into_iter()
      .filter(|product| product.split(".").last().eq(&Some(arch.as_ref())))
      .collect::<Vec<_>>();

    if image_key_vec.is_empty() {
      Err(Error::Message(format!(
        "Product catalog for image '{}' not found. Exit",
        image_name
      )))
    } else if image_key_vec.len() > 1 {
      Err(Error::Message(format!(
        "Product catalog for image '{}' multiple items found. Exit",
        image_name
      )))
    } else {
      let image_key: &String = &image_key_vec[0];
      image_map
        .get(image_key)
        .and_then(|image_value| image_value.get("id"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
          Error::Message(format!(
            "Product catalog entry '{}' for image '{}' has no 'id' field",
            image_key, image_name
          ))
        })
    }
  // } else if let Some(wildcard) = filter.get("wildcard") {
  } else if let Filter::Wildcard { wildcard } = filter {
    // Search image in product catalog and filter by wildcard
    let image_key_vec = image_map
      .keys()
      .filter(|product| product.contains(wildcard.as_str()))
      .collect::<Vec<_>>();

    if image_key_vec.is_empty() {
      Err(Error::Message(format!(
        "Product catalog for image '{}' not found. Exit",
        image_name
      )))
    } else if image_key_vec.len() > 1 {
      Err(Error::Message(format!(
        "Product catalog for image '{}' multiple items found. Exit",
        image_name
      )))
    } else {
      let image_key = image_key_vec[0];
      image_map
        .get(image_key)
        .and_then(|image_value| image_value.get("id"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
          Error::Message(format!(
            "Product catalog entry '{}' for image '{}' has no 'id' field",
            image_key, image_name
          ))
        })
    }
  // } else if let Some(prefix) = filter.get("prefix") {
  } else if let Filter::Prefix { prefix } = filter {
    // Search image in product catalog and filter by prefix
    let image_key_vec = image_map
      .keys()
      .filter(|product| product.strip_prefix(prefix.as_str()).is_some())
      .collect::<Vec<_>>();

    if image_key_vec.is_empty() {
      Err(Error::Message(format!(
        "Product catalog for image '{}' not found. Exit",
        image_name
      )))
    } else if image_key_vec.len() > 1 {
      Err(Error::Message(format!(
        "Product catalog for image '{}' multiple items found. Exit",
        image_name
      )))
    } else {
      let image_key = image_key_vec[0];
      image_map
        .get(image_key)
        .and_then(|image_value| image_value.get("id"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
          Error::Message(format!(
            "Product catalog entry '{}' for image '{}' has no 'id' field",
            image_key, image_name
          ))
        })
    }
  } else {
    Err(Error::Message(format!(
      "Product catalog for image '{}' not found. Exit",
      image_name
    )))
  }
}

pub fn validate_sat_file_images_section(
  image_yaml_vec: &[image::Image],
  configuration_yaml_vec: &[configuration::Configuration],
  hsm_group_available_vec: &[String],
  cray_product_catalog: &BTreeMap<String, String>,
  image_vec: Vec<ims::image::http_client::types::Image>,
  configuration_vec: Vec<CfsConfigurationResponse>,
  ims_recipe_vec: Vec<ims::recipe::types::RecipeGetResponse>,
) -> Result<(), Error> {
  // Validate 'images' section in SAT file

  for image_yaml in image_yaml_vec {
    // Validate image
    let image_name = &image_yaml.name;

    log::info!("Validate 'image' '{}'", image_name);

    if let image::BaseOrIms::Ims { ims } = &image_yaml.base_or_ims {
      if let image::ImageIms::IdIsRecipe { id, is_recipe: _ } = ims {
        // Validate base image
        log::info!("Validate 'image' '{}' base image '{}'", image_name, id);

        // Old format
        log::info!(
          "Searching image.ims.id (old format - backward compatibility) '{}' in CSM",
          id,
        );

        let is_image_base_id_in_csm = image_vec.iter().any(
          |image: &ims::image::http_client::types::Image| {
            image.id.as_ref().is_some_and(|image_id| image_id.eq(id))
          },
        );

        if !is_image_base_id_in_csm {
          return Err(Error::Message(format!(
            "Could not find base image id '{}' in image '{}'. Exit",
            id, image_yaml.name
          )));
        }
      }
    } else if let image::BaseOrIms::Base { base } = &image_yaml.base_or_ims {
      if let image::Base::ImageRef { image_ref } = base {
        // New format
        // Validate base image
        log::info!(
          "Validate 'image' '{}' base image '{}'",
          image_name,
          image_ref
        );

        // Check there is another image with 'ref_name' that matches this 'image_ref'
        let image_found = image_yaml_vec.iter().any(|image_yaml| {
          image_yaml
            .ref_name
            .as_ref()
            .is_some_and(|ref_name| ref_name.eq(image_ref))
        });

        if !image_found {
          return Err(Error::Message(format!(
            "Could not find image with ref name '{}' in SAT file. Cancelling image build proccess. Exit",
            image_ref.as_str(),
          )));
        }
      // } else if let Some(image_base_product) = image_yaml["base"].get("product")
      } else if let image::Base::Product { product } = base {
        // Check if the 'Cray/HPE product' in CSM exists

        log::info!("Image '{}' base.base.product", image_name);
        log::info!("SAT file - 'image.base.product' job");

        // Base image created from a cray product

        let product_name = &product.name;

        let product_version = product.version.as_ref().ok_or_else(|| {
          Error::Message(format!(
            "SAT file: image '{}' base.product '{}' is missing 'version'",
            image_name, product_name
          ))
        })?;

        let product_type = &product.r#type;

        let product_catalog_rslt = &serde_yaml::from_str::<serde_json::Value>(
          cray_product_catalog
            .get(product_name)
            .unwrap_or(&"".to_string()),
        );

        let product_catalog = if let Ok(product_catalog) = product_catalog_rslt
        {
          product_catalog
        } else {
          return Err(Error::Message(format!(
            "Product catalog for image '{}' not found. Exit",
            image_name
          )));
        };

        let product_type_opt = product_catalog
          .get(product_version)
          .and_then(|product_version| product_version.get(product_type.clone()))
          .cloned();

        let product_type_opt = if let Some(product_type) = product_type_opt {
          product_type.as_object().cloned()
        } else {
          return Err(Error::Message(format!(
            "Product catalog for image '{}' not found. Exit",
            image_name
          )));
        };

        let image_map: Map<String, serde_json::Value> =
          if let Some(product_type) = &product_type_opt {
            product_type.clone()
          } else {
            return Err(Error::Message(format!(
              "Product catalog for image '{}' not found. Exit",
              image_name
            )));
          };

        log::debug!(
          "CRAY product catalog items related to product name '{}', product version '{}' and product type '{}':\n{:#?}",
          product_name,
          product_version,
          product_type,
          product_type_opt
        );

        if let Some(filter) = &product.filter {
          let image_recipe_id =
            filter_product_catalog_images(filter, image_map, image_name);
          image_recipe_id.is_ok()
        } else {
          // There is no 'image.product.filter' value defined in SAT file. Check Cray
          // product catalog only has 1 image. Othewise fail
          log::info!(
            "No 'image.product.filter' defined in SAT file. Checking Cray product catalog only/must have 1 image"
          );
          image_map
            .values()
            .next()
            .is_some_and(|value| value.get("id").is_some())
        };
      // } else if let Some(image_base_ims_yaml) = image_yaml["base"].get("ims") {
      } else if let image::Base::Ims { ims } = base {
        // Check if the image exists

        log::info!("Image '{}' base.base.ims", image_name);
        if let image::ImageBaseIms::NameType { name, r#type } = ims {
          // if let Some(image_base_ims_name_yaml) = ims.get("name") {
          let image_base_ims_name_to_find = name;

          // Search image in SAT file

          log::info!(
            "Searching base image '{}' related to image '{}' in SAT file",
            image_base_ims_name_to_find,
            image_name
          );

          let mut image_found = image_yaml_vec
            .iter()
            .any(|image_yaml| image_yaml.name.eq(name));

          if !image_found {
            log::warn!(
              "Base image '{}' not found in SAT file, looking in CSM",
              image_base_ims_name_to_find
            );

            let image_base_ims_type = r#type;
            if image_base_ims_type.eq("recipe") {
              // Base IMS type is a recipe
              // Search in CSM (IMS Recipe)

              log::info!(
                "Searching base image recipe '{}' related to image '{}' in CSM",
                image_base_ims_name_to_find,
                image_name
              );

              image_found = ims_recipe_vec
                .iter()
                .any(|recipe| recipe.name.eq(image_base_ims_name_to_find));

              if !image_found {
                return Err(Error::Message(format!(
                  "Could not find IMS recipe '{}' in CSM. Cancelling image build proccess. Exit",
                  image_base_ims_name_to_find,
                )));
              }
            } else {
              // Base IMS type is an image
              // Search in CSM (IMS Image)

              log::info!(
                "Searching base image '{}' related to image '{}' in CSM",
                image_base_ims_name_to_find,
                image_name
              );

              // CFS session sets a custom image name, therefore we can't seach
              // for exact image name but search by substring
              image_found = image_vec
                .iter()
                .any(|image| image.name.contains(image_base_ims_name_to_find));

              if !image_found {
                return Err(Error::Message(format!(
                  "Could not find image base '{}' in image '{}'. Cancelling image build proccess. Exit",
                  image_base_ims_name_to_find, image_name
                )));
              }
            }
          }
        } else {
          log::warn!(
            "Image '{}' is missing the field 'base.ims.name'. Exit",
            image_name
          );
        };
      } else {
        return Err(Error::Message(format!(
          "Image '{}' yaml not recognised. Exit",
          image_name
        )));
      }
    } else {
      return Err(Error::Message(format!(
        "Image '{}' neither have 'ims' nor 'base' value. Exit",
        image_name
      )));
    }

    // Validate CFS configuration exists (image.configuration)
    log::info!("Validate 'image' '{}' configuration", image_name);

    if let Some(configuration_yaml) = image_yaml.configuration.as_ref() {
      let configuration_name_to_find = configuration_yaml;

      log::info!(
        "Searching configuration name '{}' related to image '{}' in SAT file",
        configuration_name_to_find,
        image_name
      );

      let mut configuration_found =
        configuration_yaml_vec.iter().any(|configuration_yaml| {
          configuration_yaml.name.eq(configuration_name_to_find)
        });

      if !configuration_found {
        // CFS configuration in image not found in SAT file, searching in CSM
        log::warn!(
          "Configuration '{}' not found in SAT file, looking in CSM",
          configuration_name_to_find
        );

        log::info!(
          "Searching configuration name '{}' related to image '{}' in CSM",
          configuration_name_to_find,
          image_yaml.name
        );

        configuration_found = configuration_vec.iter().any(|configuration| {
          configuration.name.eq(configuration_name_to_find)
        });

        if !configuration_found {
          return Err(Error::Message(format!(
            "Could not find configuration '{}' in image '{}'. Cancelling image build proccess. Exit",
            configuration_name_to_find, image_name
          )));
        }
      }

      // Validate user has access to HSM groups in 'image' section
      log::info!("Validate 'image' '{}' HSM groups", image_name);

      //TODO: Get rid of this by making sure CSM admins don't create HSM groups for system
      //wide operations instead of using roles
      let configuration_group_names_vec =
        hsm::group::hacks::filter_system_hsm_group_names(
          image_yaml
            .configuration_group_names
            .clone()
            .unwrap_or_default(),
        );

      if configuration_group_names_vec.is_empty() {
        return Err(Error::Message(format!(
          "Image '{}' must have group name values assigned to it. Canceling image build process. Exit",
          image_name
        )));
      } else {
        for hsm_group in
          configuration_group_names_vec.iter().filter(|&hsm_group| {
            !hsm_group.eq_ignore_ascii_case("Compute")
              && !hsm_group.eq_ignore_ascii_case("Application")
              && !hsm_group.eq_ignore_ascii_case("Application_UAN")
          })
        {
          if !hsm_group_available_vec.contains(hsm_group) {
            return Err(Error::Message(format!(
              "HSM group '{}' in image '{}' not allowed, List of HSM groups available:\n{:?}. Exit",
              hsm_group, image_yaml.name, hsm_group_available_vec
            )));
          }
        }
      };
    }
  }

  Ok(())
}


