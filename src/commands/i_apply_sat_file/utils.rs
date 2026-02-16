use std::collections::{BTreeMap, HashMap};

use crate::{
  bos::{
    self,
    session::http_client::v2::types::{BosSession, Operation},
    template::http_client::v2::types::{BootSet, BosSessionTemplate, Cfs},
  },
  cfs::{
    self,
    configuration::http_client::v2::types::{
      cfs_configuration_request::CfsConfigurationRequest,
      cfs_configuration_response::CfsConfigurationResponse,
    },
    session::http_client::v2::types::CfsSessionPostRequest,
  },
  commands::i_apply_sat_file::utils::{image::Filter, sessiontemplate::Arch},
  common,
  error::Error,
  hsm,
  ims::{self, image::http_client::types::Link},
  node::utils::validate_target_hsm_members,
};
use image::Image;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_yaml::Value;
use uuid::Uuid;

use self::sessiontemplate::SessionTemplate;

#[derive(Deserialize, Serialize, Debug)]
pub struct SatFile {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub configurations: Option<Vec<configuration::Configuration>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub images: Option<Vec<image::Image>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub session_templates: Option<Vec<sessiontemplate::SessionTemplate>>,
}

impl SatFile {
  /// Filter either images or session_templates section according to user request
  pub fn filter(
    &mut self,
    image_only: bool,
    session_template_only: bool,
  ) -> Result<(), Error> {
    // Clean SAT template file if user only wan'ts to process the 'images' section. In this case,
    // we will remove 'session_templates' section from SAT fiel and also the entries in
    // 'configurations' section not used
    if image_only {
      let image_vec_opt: Option<&Vec<Image>> = self.images.as_ref();

      let configuration_name_image_vec: Vec<String> = match image_vec_opt {
        Some(image_vec) => image_vec
          .iter()
          .filter_map(|sat_template_image| {
            sat_template_image.configuration.clone()
          })
          .collect(),
        None => {
          return Err(Error::Message(
            "ERROR - 'images' section missing in SAT file".to_string(),
          ));
        }
      };

      // Remove configurations not used by any image
      self.configurations.as_mut().map(|configuration_vec| {
        configuration_vec.retain(|configuration| {
          configuration_name_image_vec.contains(&configuration.name)
        })
      });

      // Remove section "session_templates"
      self.session_templates = None;
    }

    // Clean SAT template file if user only wan'ts to process the 'session_template' section. In this case,
    // we will remove 'images' section from SAT fiel and also the entries in
    // 'configurations' section not used
    if session_template_only {
      let sessiontemplate_vec_opt: Option<&Vec<SessionTemplate>> =
        self.session_templates.as_ref();

      let configuration_name_sessiontemplate_vec: Vec<String> =
        match sessiontemplate_vec_opt {
          Some(sessiontemplate_vec) => sessiontemplate_vec
            .iter()
            .map(|sat_sessiontemplate| {
              sat_sessiontemplate.configuration.clone()
            })
            .collect(),
          None => {
            return Err(Error::Message(
              "ERROR - 'session_templates' section not defined in SAT file"
                .to_string(),
            ));
          }
        };

      // Remove configurations not used by any sessiontemplate
      if let Some(&[_]) = self.configurations.as_deref() {
        self
          .configurations
          .as_mut()
          .map(|configuration_vec| {
            configuration_vec.retain(|configuration| {
              configuration_name_sessiontemplate_vec
                .contains(&configuration.name)
            })
          })
          .unwrap_or_default()
      } else {
        self.configurations = None;
      }

      let image_name_sessiontemplate_vec: Vec<String> = self
        .session_templates
        .as_ref()
        .map(|session_template_vec| {
          {
            session_template_vec.iter().filter_map(|sessiontemplate| {
              match &sessiontemplate.image {
                sessiontemplate::Image::ImageRef(name) => Some(name),
                sessiontemplate::Image::Ims { ims } => match ims {
                  sessiontemplate::ImsDetails::Name { name } => Some(name),
                  sessiontemplate::ImsDetails::Id { .. } => None,
                },
              }
            })
          }
          .cloned()
          .collect()
        })
        .unwrap_or_default();

      // Remove images not used by any sessiontemplate
      self.images.as_mut().map(|image_vec| {
        image_vec.retain(|sat_image| {
          image_name_sessiontemplate_vec.contains(&sat_image.name)
        })
      });

      if self.images.as_ref().is_some_and(|images| images.is_empty()) {
        self.images = None;
      }
    }

    Ok(())
  }
}

/// struct to represent the `session_templates` section in SAT file
pub mod sessiontemplate {
  use std::collections::HashMap;
  use strum_macros::Display;

  use serde::{Deserialize, Serialize};

  #[derive(Deserialize, Serialize, Debug)]
  pub struct SessionTemplate {
    pub name: String,
    pub image: Image,
    pub configuration: String,
    pub bos_parameters: BosParamters,
  }

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum ImsDetails {
    Name { name: String },
    Id { id: String },
  }

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Image {
    Ims { ims: ImsDetails },
    ImageRef(String),
  }

  #[derive(Deserialize, Serialize, Debug)]
  pub struct BosParamters {
    pub boot_sets: HashMap<String, BootSet>,
  }

  #[derive(Deserialize, Serialize, Debug)]
  pub struct BootSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<Arch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_parameters: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_roles_group: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_groups: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rootfs_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rootfs_provider_passthrough: Option<String>,
  }

  #[derive(Deserialize, Serialize, Debug, Display)]
  pub enum Arch {
    X86,
    ARM,
    Other,
    Unknown,
  }
}

/// Convert from `sessiontemplate` in SAT file to mesa BosSessionTemplate
/// example from https://doc.rust-lang.org/rust-by-example/conversion/try_from_try_into.html
impl TryFrom<SessionTemplate> for BosSessionTemplate {
  type Error = ();

  fn try_from(
    value: SessionTemplate,
  ) -> Result<BosSessionTemplate, Self::Error> {
    let b_st_cfs = Cfs {
      configuration: Some(value.configuration),
    };

    let mut boot_set_map: HashMap<String, BootSet> = HashMap::new();

    for (property, boot_set) in value.bos_parameters.boot_sets {
      let boot_set = BootSet {
        name: Some(format!(
          "Boot set property '{}' created by manta from SAT file",
          property
        )),
        path: None,
        r#type: None,
        etag: None,
        kernel_parameters: None,
        node_list: boot_set.node_list,
        node_roles_groups: boot_set.node_roles_group,
        node_groups: boot_set.node_groups,
        rootfs_provider: boot_set.rootfs_provider,
        rootfs_provider_passthrough: boot_set.rootfs_provider_passthrough,
        cfs: Some(b_st_cfs.clone()),
        arch: boot_set.arch.as_ref().map(Arch::to_string),
      };

      boot_set_map.insert(property, boot_set);
    }

    let b_st = BosSessionTemplate {
      name: Some(value.name),
      description: Some(format!(
        "BOS sessiontemplate created by manta from SAT file"
      )),
      enable_cfs: Some(true),
      cfs: Some(b_st_cfs),
      boot_sets: Some(boot_set_map),
      links: None,
      tenant: None,
    };

    Ok(b_st)
  }
}

/// struct to represent the `images` section in SAT file
pub mod image {
  use serde::{Deserialize, Serialize};
  use strum_macros::AsRefStr;

  #[derive(Deserialize, Serialize, Debug, Clone, AsRefStr)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Arch {
    #[serde(rename(serialize = "aarch64", deserialize = "aarch64"))]
    Aarch64,
    #[serde(rename(serialize = "x86_64", deserialize = "x86_64"))]
    X86_64,
  }

  #[derive(Deserialize, Serialize, Debug, Clone)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum ImageIms {
    NameIsRecipe { name: String, is_recipe: bool },
    IdIsRecipe { id: String, is_recipe: bool },
  }

  #[derive(Deserialize, Serialize, Debug, Clone)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum ImageBaseIms {
    NameType { name: String, r#type: String },
    IdType { id: String, r#type: String },
    BackwardCompatible { is_recipe: Option<bool>, id: String },
  }

  #[derive(Deserialize, Serialize, Debug, Clone)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Filter {
    Prefix { prefix: String },
    Wildcard { wildcard: String },
    Arch { arch: Arch },
  }

  #[derive(Deserialize, Serialize, Debug, Clone)]
  pub struct Product {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub r#type: String,
    pub filter: Option<Filter>,
  }

  #[derive(Deserialize, Serialize, Debug, Clone)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Base {
    Ims { ims: ImageBaseIms },
    Product { product: Product },
    ImageRef { image_ref: String },
  }

  // Used for backguard compatibility
  #[derive(Deserialize, Serialize, Debug, Clone)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum BaseOrIms {
    Base { base: Base },
    Ims { ims: ImageIms },
  }

  #[derive(Deserialize, Serialize, Debug, Clone)]
  pub struct Image {
    pub name: String,
    #[serde(flatten)]
    pub base_or_ims: BaseOrIms,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_group_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
  }
}

/// struct to represent the `configurations` section in SAT file
pub mod configuration {
  use serde::{Deserialize, Serialize};

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Product {
    ProductVersionBranch {
      name: String,
      version: Option<String>,
      branch: String,
    },
    ProductVersionCommit {
      name: String,
      version: Option<String>,
      commit: String,
    },
    ProductVersion {
      name: String,
      version: String,
    },
  }

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Git {
    GitCommit { url: String, commit: String },
    GitBranch { url: String, branch: String },
    GitTag { url: String, tag: String },
  }

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum LayerType {
    Git { git: Git },
    Product { product: Product },
  }

  #[derive(Deserialize, Serialize, Debug)]
  pub struct Layer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default = "default_playbook")]
    pub playbook: String, // This field is optional but with default value. Therefore we won't
    #[serde(flatten)]
    pub layer_type: LayerType,
  }

  fn default_playbook() -> String {
    "site.yml".to_string()
  }

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Inventory {
    InventoryCommit {
      #[serde(skip_serializing_if = "Option::is_none")]
      name: Option<String>,
      url: String,
      commit: String,
    },
    InventoryBranch {
      #[serde(skip_serializing_if = "Option::is_none")]
      name: Option<String>,
      url: String,
      branch: String,
    },
  }

  #[derive(Deserialize, Serialize, Debug)]
  pub struct Configuration {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub layers: Vec<Layer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_inventory: Option<Inventory>,
  }
}

pub mod sat_file_image_old {
  use serde::{Deserialize, Serialize};

  #[derive(Deserialize, Serialize, Debug)]
  pub struct Ims {
    is_recipe: bool,
    id: String,
  }

  #[derive(Deserialize, Serialize, Debug)]
  pub struct Product {
    name: String,
    version: String,
    r#type: String,
  }
}

pub async fn create_cfs_configuration_from_sat_file(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  gitea_base_url: &str,
  gitea_token: &str,
  cray_product_catalog: &BTreeMap<String, String>,
  sat_file_configuration_yaml: &serde_yaml::Value,
  dry_run: bool,
  site_name: &str,
  overwrite: bool,
) -> Result<CfsConfigurationResponse, Error> {
  log::debug!(
    "Convert CFS configuration in SAT file (yaml):\n{:#?}",
    sat_file_configuration_yaml
  );

  let (cfs_configuration_name, cfs_configuration) =
    CfsConfigurationRequest::from_sat_file_serde_yaml(
      shasta_root_cert,
      gitea_base_url,
      gitea_token,
      sat_file_configuration_yaml,
      cray_product_catalog,
      site_name,
    )
    .await?;

  if dry_run {
    println!(
      "Dry run mode: Create CFS configuration:\n{}",
      serde_json::to_string_pretty(&cfs_configuration)?
    );

    // Generate mock CFS configuration
    let cfs_configuration = CfsConfigurationResponse {
      name: cfs_configuration_name,
      last_updated: "".to_string(),
      layers: Vec::new(),
      additional_inventory: None,
    };

    // Return mock CFS configuration
    Ok(cfs_configuration)
  } else {
    cfs::configuration::utils::create_new_configuration(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &cfs_configuration,
      &cfs_configuration_name,
      overwrite,
    )
    .await
  }
}

pub async fn create_cfs_configuration_struct_from_sat_file(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  gitea_base_url: &str,
  gitea_token: &str,
  cray_product_catalog: &BTreeMap<String, String>,
  sat_file_configuration_yaml: &configuration::Configuration,
  dry_run: bool,
  site_name: &str,
  overwrite: bool,
) -> Result<CfsConfigurationResponse, Error> {
  log::debug!(
    "Convert CFS configuration in SAT file (yaml):\n{:#?}",
    sat_file_configuration_yaml
  );

  let (cfs_configuration_name, cfs_configuration) =
    CfsConfigurationRequest::from_sat_file_struct_serde_yaml(
      shasta_root_cert,
      gitea_base_url,
      gitea_token,
      sat_file_configuration_yaml,
      cray_product_catalog,
      site_name,
    )
    .await?;

  if dry_run {
    println!(
      "Dry run mode: Create CFS configuration:\n{}",
      serde_json::to_string_pretty(&cfs_configuration)?
    );

    // Generate mock CFS configuration
    let cfs_configuration = CfsConfigurationResponse {
      name: cfs_configuration_name,
      last_updated: "".to_string(),
      layers: Vec::new(),
      additional_inventory: None,
    };

    // Return mock CFS configuration
    Ok(cfs_configuration)
  } else {
    cfs::configuration::utils::create_new_configuration(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &cfs_configuration,
      &cfs_configuration_name,
      overwrite,
    )
    .await
  }
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

      let image_base_image_ref_opt =
        if let image::BaseOrIms::Base { base } = &image_yaml.base_or_ims {
          if let image::Base::ImageRef { image_ref } = base {
            Some(image_ref)
          } else {
            None
          }
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
  if image_yaml.get("ref_name").is_some() {
    image_yaml
      .get("ref_name")
      .and_then(Value::as_str)
      .map(str::to_string)
      .unwrap()
  } else {
    // If the image processed is missing the field "ref_name", then use the field "name"
    // instead, this is needed to flag this image as processed and filtered when
    // calculating the next image to process (get_next_image_to_process)
    image_yaml
      .get("name")
      .and_then(Value::as_str)
      .map(str::to_string)
      .unwrap()
  }
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
  since = "v0.86.2",
  note = "this function prints cfs session logs to stdout"
)]
pub async fn i_import_images_section_in_sat_file(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
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
      &image_yaml_vec,
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
    let image_id = i_create_image_from_sat_file_serde_yaml(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
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
      &image_yaml_vec,
      &ref_name_processed_hashmap
        .keys()
        .cloned()
        .collect::<Vec<String>>(),
    );
  }

  Ok(image_processed_hashmap)
}

#[deprecated(
  since = "v0.86.2",
  note = "this function prints cfs session logs to stdout"
)]
pub async fn i_create_image_from_sat_file_serde_yaml(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
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
    let cfs_session_rslt = cfs::session::i_post_sync(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      vault_base_url,
      site_name,
      k8s_api_url,
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
    println!("Image '{}' ({}) created", image_name, image_id);

    Ok(image_id.to_string())
  } else {
    println!(
      "Dry run mode: Create CFS session:\n{}",
      serde_json::to_string_pretty(&cfs_session)?
    );

    let image_id = Uuid::new_v4().to_string();

    println!(
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
  let configuration_name = image_yaml.configuration.as_ref().unwrap();

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
    &image_yaml,
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
    &configuration_name,
    None,
    ansible_verbosity_opt,
    ansible_passthrough_opt,
    true,
    Some(&groups_name),
    Some(&base_image_id),
  );

  return Ok(cfs_session);
}

async fn process_sat_file_image_product_type_ims_recipe(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
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
      root_ims_key_name,
    )
    .await?
    .ok_or_else(|| Error::ImsKeyNotFound(root_ims_key_name.to_string()))?;

  let root_public_ssh_key: &str = root_public_ssh_key_value
    .get("id")
    .and_then(serde_json::Value::as_str)
    .unwrap();

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
    println!(
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
      &ims_job,
    )
    .await?
  };

  Ok(ims_job.resultant_image_id.unwrap())
}

/* async fn process_sat_file_image_ims_type_recipe(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  sat_file_image_base_ims_value_yaml: &serde_yaml::Value,
  image_name: &String,
  dry_run: bool,
) -> Result<String, Error> {
  // Base image needs to be created from a IMS job using an IMS recipe
  let recipe_name = sat_file_image_base_ims_value_yaml
    .get("name")
    .and_then(Value::as_str)
    .unwrap();

  // Get all IMS recipes
  let recipe_detail_vec: Vec<ims::recipe::types::RecipeGetResponse> =
    ims::recipe::http_client::get(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      None,
    )
    .await?;

  // Filter recipes by name
  let recipe_detail_opt = recipe_detail_vec
    .iter()
    .find(|recipe| recipe.name == recipe_name);

  log::info!("IMS recipe details:\n{:#?}", recipe_detail_opt);

  // Check recipe with requested name exists
  let recipe_id = if let Some(recipe_detail) = recipe_detail_opt {
    recipe_detail.id.as_ref().unwrap()
  } else {
    return Err(Error::Message(format!(
      "IMS recipe with name '{}' - not found. Exit",
      recipe_name
    )));
  };

  log::info!("IMS recipe id found '{}'", recipe_id);

  let root_ims_key_name = "mgmt root key";

  // Get root public ssh key
  let root_public_ssh_key_value: serde_json::Value =
    ims::public_keys::http_client::v3::get_single(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      root_ims_key_name,
    )
    .await?
    .ok_or_else(|| Error::ImsKeyNotFound(root_ims_key_name.to_string()))?;

  let root_public_ssh_key = root_public_ssh_key_value
    .get("id")
    .and_then(serde_json::Value::as_str)
    .unwrap();

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
    println!(
      "Dry run mode: Create IMS job:\n{}",
      serde_json::to_string_pretty(&ims_job)?
    );
    ims_job.into()
  } else {
    ims::job::http_client::post_sync(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &ims_job,
    )
    .await?
  };

  log::info!("IMS job response:\n{:#?}", ims_job);

  Ok(ims_job.resultant_image_id.unwrap())
} */

async fn process_sat_file_image_ims_type_recipe(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  recipe_name: &str,
  image_name: &String,
  dry_run: bool,
) -> Result<String, Error> {
  // Base image needs to be created from a IMS job using an IMS recipe
  // Get all IMS recipes
  let recipe_detail_vec: Vec<ims::recipe::types::RecipeGetResponse> =
    ims::recipe::http_client::get(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      None,
    )
    .await?;

  // Filter recipes by name
  let recipe_detail_opt = recipe_detail_vec
    .iter()
    .find(|recipe| recipe.name == recipe_name);

  log::info!("IMS recipe details:\n{:#?}", recipe_detail_opt);

  // Check recipe with requested name exists
  let recipe_id = if let Some(recipe_detail) = recipe_detail_opt {
    recipe_detail.id.as_ref().unwrap()
  } else {
    return Err(Error::Message(format!(
      "IMS recipe with name '{}' - not found. Exit",
      recipe_name
    )));
  };

  log::info!("IMS recipe id found '{}'", recipe_id);

  let root_ims_key_name = "mgmt root key";

  // Get root public ssh key
  let root_public_ssh_key_value: serde_json::Value =
    ims::public_keys::http_client::v3::get_single(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      root_ims_key_name,
    )
    .await?
    .ok_or_else(|| Error::ImsKeyNotFound(root_ims_key_name.to_string()))?;

  let root_public_ssh_key = root_public_ssh_key_value
    .get("id")
    .and_then(serde_json::Value::as_str)
    .unwrap();

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
    println!(
      "Dry run mode: Create IMS job:\n{}",
      serde_json::to_string_pretty(&ims_job)?
    );
    ims_job.into()
  } else {
    ims::job::http_client::post_sync(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &ims_job,
    )
    .await?
  };

  log::info!("IMS job response:\n{:#?}", ims_job);

  Ok(ims_job.resultant_image_id.unwrap())
}

#[deprecated(
  since = "0.100.0",
  note = "Plesae make use of function 'process_sat_file_image_old_version_struct' instead since migrating from serde_yaml::Value to a struct to make use of Rsut type system"
)]
fn process_sat_file_image_old_version(
  sat_file_image_ims_value_yaml: &serde_yaml::Value,
) -> Result<String, Error> {
  if sat_file_image_ims_value_yaml
    .get("is_recipe")
    .is_some_and(|is_recipe_value| is_recipe_value.as_bool().unwrap() == false)
    && sat_file_image_ims_value_yaml.get("id").is_some()
  {
    // Create final image from CFS session
    Ok(
      sat_file_image_ims_value_yaml
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap(),
    )
  } else {
    Err(Error::Message("Functionality not built. Exit".to_string()))
  }
}

// TODO: rename to "process_sat_file_image_old_version" and delete the original one already
// deprecated
fn process_sat_file_image_old_version_struct(
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

#[deprecated(since = "0.100.0")]
fn process_sat_file_image_ref_name(
  sat_file_image_base_image_ref_value_yaml: &serde_yaml::Value,
  ref_name_image_id_hashmap: &HashMap<String, String>,
) -> Result<String, Error> {
  let image_ref_opt: Option<String> = sat_file_image_base_image_ref_value_yaml
    .as_str()
    .map(str::to_string);

  match image_ref_opt {
    Some(image_ref) => {
      Ok(ref_name_image_id_hashmap.get(&image_ref).cloned().unwrap())
    }
    None => Err(Error::Message(
      "field 'image_ref' in SAT file not found".to_string(), // FIXME: Create an enum in
                                                             // csm-rs::Error for this
    )),
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
      let image_key: &String = image_key_vec.first().unwrap();
      let image_value_opt = image_map.get(image_key);
      Ok(
        image_value_opt
          .and_then(|image_value| image_value.get("id"))
          .and_then(serde_json::Value::as_str)
          .map(str::to_string)
          .unwrap(),
      )
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
      let image_key = image_key_vec.first().cloned().unwrap();
      let image_value_opt = image_map.get(image_key);
      Ok(
        image_value_opt
          .and_then(|image_value| image_value.get("id"))
          .and_then(serde_json::Value::as_str)
          .map(str::to_string)
          .unwrap(),
      )
    }
  // } else if let Some(prefix) = filter.get("prefix") {
  } else if let Filter::Prefix { prefix } = filter {
    // Search image in product catalog and filter by prefix
    let image_key_vec = image_map
      .keys()
      .filter(|product| product.strip_prefix(&prefix.as_str()).is_some())
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
      let image_key = image_key_vec.first().cloned().unwrap();
      let image_value_opt = image_map.get(image_key);
      Ok(
        image_value_opt
          .and_then(|image_value| image_value.get("id"))
          .and_then(serde_json::Value::as_str)
          .map(str::to_string)
          .unwrap(),
      )
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
      if let image::ImageIms::IdIsRecipe { id, is_recipe } = ims {
        // Validate base image
        log::info!("Validate 'image' '{}' base image '{}'", image_name, id);

        // Old format
        log::info!(
          "Searching image.ims.id (old format - backward compatibility) '{}' in CSM",
          id,
        );

        let is_image_base_id_in_csm = image_vec.iter().any(
          |image: &ims::image::http_client::types::Image| {
            let image_id = image.id.as_ref().unwrap();
            image_id.eq(id)
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

        let product_version = product.version.as_ref().unwrap();

        let product_type = &product.r#type;

        let product_catalog_rslt = &serde_yaml::from_str::<serde_json::Value>(
          &cray_product_catalog
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
            filter_product_catalog_images(&filter, image_map, &image_name);
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
          eprintln!(
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
          if !hsm_group_available_vec.contains(&hsm_group) {
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

/* pub fn validate_sat_file_images_section(
  image_yaml_vec: &Vec<Value>,
  configuration_yaml_vec: &Vec<Value>,
  hsm_group_available_vec: &[String],
  cray_product_catalog: &BTreeMap<String, String>,
  image_vec: Vec<ims::image::http_client::types::Image>,
  configuration_vec: Vec<CfsConfigurationResponse>,
  ims_recipe_vec: Vec<ims::recipe::types::RecipeGetResponse>,
) -> Result<(), Error> {
  // Validate 'images' section in SAT file

  for image_yaml in image_yaml_vec {
    // Validate image
    let image_name = image_yaml.get("name").and_then(Value::as_str).unwrap();

    log::info!("Validate 'image' '{}'", image_name);

    if let Some(image_ims_id_to_find) = image_yaml
      .get("ims")
      .and_then(|ims| ims.get("id").and_then(Value::as_str))
    {
      // Validate base image
      log::info!(
        "Validate 'image' '{}' base image '{}'",
        image_name,
        image_ims_id_to_find
      );

      // Old format
      log::info!(
                "Searching image.ims.id (old format - backward compatibility) '{}' in CSM",
                image_ims_id_to_find,
            );

      let is_image_base_id_in_csm = image_vec.iter().any(
        |image: &ims::image::http_client::types::Image| {
          let image_id = image.id.as_ref().unwrap();
          image_id.eq(image_ims_id_to_find)
        },
      );

      if !is_image_base_id_in_csm {
        return Err(Error::Message(format!(
          "Could not find base image id '{}' in image '{}'. Exit",
          image_ims_id_to_find,
          image_yaml.get("name").and_then(Value::as_str).unwrap()
        )));
      }
    } else if image_yaml.get("base").is_some() {
      // New format
      if let Some(image_ref_to_find) = image_yaml
        .get("base")
        .and_then(|base| base.get("image_ref"))
      {
        // Validate base image
        log::info!(
          "Validate 'image' '{}' base image '{}'",
          image_name,
          image_ref_to_find.clone().as_str().unwrap()
        );

        // Check there is another image with 'ref_name' that matches this 'image_ref'
        let image_found = image_yaml_vec.iter().any(|image_yaml| {
          image_yaml
            .get("ref_name")
            .is_some_and(|ref_name| ref_name.eq(image_ref_to_find))
        });

        if !image_found {
          return Err(Error::Message(format!(
                                "Could not find image with ref name '{}' in SAT file. Cancelling image build proccess. Exit",
                                image_ref_to_find.as_str().unwrap(),
                            )));
        }
      } else if let Some(image_base_product) = image_yaml["base"].get("product")
      {
        // Check if the 'Cray/HPE product' in CSM exists

        log::info!("Image '{}' base.base.product", image_name);
        log::info!("SAT file - 'image.base.product' job");

        // Base image created from a cray product

        let product_name = image_base_product
          .get("name")
          .and_then(Value::as_str)
          .unwrap();

        let product_version = image_base_product
          .get("version")
          .and_then(Value::as_str)
          .unwrap();

        let product_type = image_base_product
          .get("type")
          .and_then(Value::as_str)
          .map(|v| v.to_string() + "s")
          .unwrap();

        let product_catalog_rslt = &serde_yaml::from_str::<serde_json::Value>(
          &cray_product_catalog
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

        log::debug!("CRAY product catalog items related to product name '{}', product version '{}' and product type '{}':\n{:#?}", product_name, product_version, product_type, product_type_opt);

        if let Some(filter) = image_base_product.get("filter") {
          let image_recipe_id =
            filter_product_catalog_images(filter, image_map, image_name);
          image_recipe_id.is_ok()
        } else {
          // There is no 'image.product.filter' value defined in SAT file. Check Cray
          // product catalog only has 1 image. Othewise fail
          log::info!("No 'image.product.filter' defined in SAT file. Checking Cray product catalog only/must have 1 image");
          image_map
            .values()
            .next()
            .is_some_and(|value| value.get("id").is_some())
        };
      } else if let Some(image_base_ims_yaml) = image_yaml["base"].get("ims") {
        // Check if the image exists

        log::info!("Image '{}' base.base.ims", image_name);
        if let Some(image_base_ims_name_yaml) = image_base_ims_yaml.get("name")
        {
          let image_base_ims_name_to_find =
            image_base_ims_name_yaml.as_str().unwrap();

          // Search image in SAT file

          log::info!(
            "Searching base image '{}' related to image '{}' in SAT file",
            image_base_ims_name_to_find,
            image_name
          );

          let mut image_found = image_yaml_vec.iter().any(|image_yaml| {
            image_yaml
              .get("name")
              .is_some_and(|name| name.eq(image_base_ims_name_yaml))
          });

          if !image_found {
            log::warn!(
              "Base image '{}' not found in SAT file, looking in CSM",
              image_base_ims_name_to_find
            );

            if let Some(image_base_ims_type_yaml) =
              image_base_ims_yaml.get("type")
            {
              let image_base_ims_type =
                image_base_ims_type_yaml.as_str().unwrap();
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
                image_found = image_vec.iter().any(|image| {
                  image.name.contains(image_base_ims_name_to_find)
                });

                if !image_found {
                  return Err(Error::Message(format!(
                                        "Could not find image base '{}' in image '{}'. Cancelling image build proccess. Exit",
                                        image_base_ims_name_to_find,
                                        image_name
                                    )));
                }
              }
            } else {
              return Err(Error::Message(format!(
                                "Image '{}' is missing the field base.ims.type. Cancelling image build proccess. Exit",
                                image_base_ims_name_to_find,
                            )));
            }
          }
        } else {
          eprintln!(
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

    if let Some(configuration_yaml) = image_yaml.get("configuration") {
      let configuration_name_to_find = configuration_yaml.as_str().unwrap();

      log::info!(
        "Searching configuration name '{}' related to image '{}' in SAT file",
        configuration_name_to_find,
        image_name
      );

      let mut configuration_found =
        configuration_yaml_vec.iter().any(|configuration_yaml| {
          configuration_yaml
            .get("name")
            .and_then(Value::as_str)
            .eq(&Some(configuration_name_to_find))
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
          image_yaml.get("name").and_then(Value::as_str).unwrap()
        );

        configuration_found = configuration_vec.iter().any(|configuration| {
          configuration.name.eq(configuration_name_to_find)
        });

        if !configuration_found {
          return Err(Error::Message(format!(
             "Could not find configuration '{}' in image '{}'. Cancelling image build proccess. Exit",
             configuration_name_to_find,
             image_name
          )));
        }
      }

      // Validate user has access to HSM groups in 'image' section
      log::info!("Validate 'image' '{}' HSM groups", image_name);

      let configuration_group_names_vec: Vec<String> =
        serde_yaml::from_value(image_yaml["configuration_group_names"].clone())
          .unwrap_or(Vec::new());

      //TODO: Get rid of this by making sure CSM admins don't create HSM groups for system
      //wide operations instead of using roles
      let configuration_group_names_vec =
        hsm::group::hacks::filter_system_hsm_group_names(
          configuration_group_names_vec,
        );

      if configuration_group_names_vec.is_empty() {
        return Err(Error::Message(format!("Image '{}' must have group name values assigned to it. Canceling image build process. Exit", image_name)));
      } else {
        for hsm_group in
          configuration_group_names_vec.iter().filter(|&hsm_group| {
            !hsm_group.eq_ignore_ascii_case("Compute")
              && !hsm_group.eq_ignore_ascii_case("Application")
              && !hsm_group.eq_ignore_ascii_case("Application_UAN")
          })
        {
          if !hsm_group_available_vec.contains(&hsm_group) {
            return Err(Error::Message(format!
               (
               "HSM group '{}' in image '{}' not allowed, List of HSM groups available:\n{:?}. Exit",
               hsm_group,
               image_yaml.get("name").and_then(Value::as_str).unwrap(),
               hsm_group_available_vec
            )));
          }
        }
      };
    }
  }

  Ok(())
} */

pub fn validate_sat_file_configurations_section(
  configuration_yaml_vec: &[configuration::Configuration],
  image_yaml_vec_opt: &[image::Image],
  sessiontemplate_yaml_vec_opt: &[sessiontemplate::SessionTemplate],
) -> Result<(), Error> {
  // Validate 'configurations' sections
  if !configuration_yaml_vec.is_empty() {
    if image_yaml_vec_opt.is_empty() && sessiontemplate_yaml_vec_opt.is_empty()
    {
      return Err(Error::Message(
        "Incorrect SAT file. Please define either an 'images' or a 'session_templates' section. Exit"
            .to_string(),
      ));
    }
  }

  Ok(())
}

/* pub fn validate_sat_file_configurations_section(
  configuration_yaml_vec_opt: Option<&Vec<Value>>,
  image_yaml_vec_opt: Option<&Vec<Value>>,
  sessiontemplate_yaml_vec_opt: Option<&Vec<Value>>,
) -> Result<(), Error> {
  // Validate 'configurations' sections
  if configuration_yaml_vec_opt.is_some()
    && !configuration_yaml_vec_opt.unwrap().is_empty()
  {
    if !(image_yaml_vec_opt.is_some()
      && !image_yaml_vec_opt.unwrap().is_empty())
      && !(sessiontemplate_yaml_vec_opt.is_some()
        && !sessiontemplate_yaml_vec_opt.unwrap().is_empty())
    {
      return Err(Error::Message(
        "Incorrect SAT file. Please define either an 'image' or a 'session template'. Exit"
            .to_string(),
      ));
    }
  }

  Ok(())
} */

pub async fn validate_sat_file_session_template_section(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  image_yaml_vec: &[image::Image],
  configuration_yaml_vec: &[configuration::Configuration],
  session_template_yaml_vec: &[sessiontemplate::SessionTemplate],
  hsm_group_available_vec: &[String],
) -> Result<(), Error> {
  // Validate 'session_template' section in SAT file
  log::info!("Validate 'session_template' section in SAT file");
  for session_template_yaml in session_template_yaml_vec {
    // Validate session_template
    log::info!(
      "Validate 'session_template' '{}'",
      session_template_yaml.name
    );

    // Validate user has access to HSM groups in 'session_template' section
    log::info!(
      "Validate 'session_template' '{}' HSM groups",
      session_template_yaml.name
    );

    let bos_session_template_hsm_groups: Vec<String> =
      if let Some(boot_sets_compute) = session_template_yaml
        .bos_parameters
        .boot_sets
        .get("compute")
      {
        boot_sets_compute.node_groups.clone().unwrap_or_default()
      } else if let Some(boot_sets_uan) =
        session_template_yaml.bos_parameters.boot_sets.get("uan")
      {
        boot_sets_uan.node_groups.clone().unwrap_or_default()
      } else {
        return Err(Error::Message(format!(
          "No HSM group found in session_templates section in SAT file"
        )));
      };

    for hsm_group in bos_session_template_hsm_groups {
      if !hsm_group_available_vec.contains(&hsm_group) {
        return Err(Error::Message(format!(
          "HSM group '{}' in session_templates {} not allowed, List of HSM groups available {:?}. Exit",
          hsm_group, session_template_yaml.name, hsm_group_available_vec
        )));
      }
    }

    // Validate boot image (session_template.image)
    log::info!(
      "Validate 'session_template' '{}' boot image",
      session_template_yaml.name
    );

    if let sessiontemplate::Image::ImageRef(ref_name_to_find) =
      &session_template_yaml.image
    {
      // Validate image_ref (session_template.image.image_ref). Search in SAT file for any
      // image with images[].ref_name
      log::info!("Searching ref_name '{}' in SAT file", ref_name_to_find,);

      let image_ref_name_found = image_yaml_vec
        .iter()
        .any(|image| image.ref_name.eq(&Some(ref_name_to_find).cloned()));

      if !image_ref_name_found {
        return Err(Error::Message(format!(
          "Could not find image ref '{}' in SAT file. Exit",
          ref_name_to_find
        )));
      }
    /* } else if let Some(image_name_substr_to_find) = session_template_yaml
    .get("image")
    .and_then(|image| image.get("ims").and_then(|ims| ims.get("name"))) */
    } else if let sessiontemplate::Image::Ims { ims } =
      &session_template_yaml.image
    {
      if let sessiontemplate::ImsDetails::Name {
        name: image_name_substr_to_find,
      } = ims
      {
        // Validate image name (session_template.image.ims.name). Search in SAT file and CSM
        log::info!(
          "Searching image name '{}' related to session template '{}' in SAT file",
          image_name_substr_to_find,
          session_template_yaml.name
        );

        let mut image_found = image_yaml_vec
          .iter()
          .any(|image| image.name.eq(image_name_substr_to_find));

        if !image_found {
          // image not found in SAT file, looking in CSM
          log::warn!(
            "Image name '{}' not found in SAT file, looking in CSM",
            image_name_substr_to_find
          );
          log::info!(
            "Searching image name '{}' related to session template '{}' in CSM",
            image_name_substr_to_find,
            session_template_yaml.name
          );

          image_found = ims::image::utils::get_fuzzy(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group_available_vec,
            Some(image_name_substr_to_find.as_str()),
            Some(&1),
          )
          .await
          .is_ok();
        }

        if !image_found {
          return Err(Error::Message(format!(
            "Could not find image name '{}' in session_template '{}'. Exit",
            image_name_substr_to_find, session_template_yaml.name
          )));
        }
      }
    /* } else if let Some(image_id) = session_template_yaml
      .get("image")
      .and_then(|image| image.get("ims").and_then(|ims| ims.get("id")))
    { */
    } else if let sessiontemplate::Image::Ims { ims } =
      &session_template_yaml.image
    {
      if let sessiontemplate::ImsDetails::Id { id: image_id } = ims {
        // Validate image id (session_template.image.ims.id). Search in SAT file and CSM
        log::info!(
          "Searching image id '{}' related to session template '{}' in CSM",
          image_id,
          session_template_yaml.name
        );

        let image_found = ims::image::http_client::get(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          Some(image_id.as_str()),
        )
        .await
        .is_ok();

        if !image_found {
          return Err(Error::Message(format!(
            "Could not find image id '{}' in session_template '{}'. Exit",
            image_id, session_template_yaml.name
          )));
        }
      }
    }

    // Validate configuration
    log::info!(
      "Validate 'session_template' '{}' configuration",
      session_template_yaml.name
    );

    log::info!(
      "Searching configuration name '{}' related to session template '{}' in CSM in SAT file",
      session_template_yaml.configuration,
      session_template_yaml.name
    );

    let mut configuration_found =
      configuration_yaml_vec.iter().any(|configuration_yaml| {
        configuration_yaml
          .name
          .eq(&session_template_yaml.configuration)
      });

    if !configuration_found {
      // CFS configuration in session_template not found in SAT file, searching in CSM
      log::warn!("Configuration not found in SAT file, looking in CSM");
      log::info!(
        "Searching configuration name '{}' related to session_template '{}' in CSM",
        session_template_yaml.configuration,
        session_template_yaml.name
      );

      configuration_found = cfs::configuration::http_client::v3::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&session_template_yaml.configuration),
      )
      .await
      .is_ok();

      if !configuration_found {
        return Err(Error::Message(format!(
          "Could not find configuration '{}' in session_template '{}'. Exit",
          session_template_yaml.configuration, session_template_yaml.name,
        )));
      }
    }
  }

  Ok(())
}

pub async fn process_session_template_section_in_sat_file(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  ref_name_processed_hashmap: HashMap<String, String>,
  hsm_group_available_vec: &[String],
  sat_file_yaml: Value,
  reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let empty_vec = Vec::new();
  let bos_session_template_list_yaml = sat_file_yaml
    .get("session_templates")
    .and_then(Value::as_sequence)
    .unwrap_or(&empty_vec);

  if bos_session_template_list_yaml.is_empty() {
    log::warn!(
      "No 'session_templates' section found in SAT file. Skipping session template processing"
    );
    return Ok(());
  }

  let mut bos_st_created_vec: Vec<String> = Vec::new();

  for bos_sessiontemplate_yaml in bos_session_template_list_yaml {
    // Get boot image details in BOS sessiontemplate. This is needed to create the BOS
    // sessiontemplate BootSets
    let image_details: ims::image::http_client::types::Image =
      if let Some(bos_sessiontemplate_image) =
        bos_sessiontemplate_yaml.get("image")
      {
        let (image_reference, is_image_id) =
          get_image_reference_from_bos_sessiontemplate_yaml(
            bos_sessiontemplate_image,
            &ref_name_processed_hashmap,
          )?;
        if dry_run {
          let dry_run_mock_image =
            get_image_details_from_bos_sessiontemplate_yaml(
              shasta_token,
              shasta_base_url,
              shasta_root_cert,
              &hsm_group_available_vec,
              &image_reference,
              is_image_id,
            )
            .await
            .unwrap_or_else(|_| {
              // In dry run mode, generate a mock image
              let dry_run_mock_image = if is_image_id {
                // Image reference is an image ID
                ims::image::http_client::types::Image {
                  id: Some(image_reference.to_string()),
                  created: None,
                  name: "dryrun_image".to_string(),
                  link: Some(Link {
                    path: "dryrun_path".to_string(),
                    etag: Some("dryrun_etag".to_string()),
                    r#type: "dryrun_type".to_string(),
                  }),
                  arch: None,
                  metadata: None,
                }
              } else {
                // Image reference is an image name
                ims::image::http_client::types::Image {
                  id: None,
                  created: None,
                  name: image_reference.to_string(),
                  link: Some(Link {
                    path: "dryrun_path".to_string(),
                    etag: Some("dryrun_etag".to_string()),
                    r#type: "dryrun_type".to_string(),
                  }),
                  arch: None,
                  metadata: None,
                }
              };

              dry_run_mock_image
            });

          println!(
            "Dry run mode: Generate mock Image\n{}",
            serde_json::to_string_pretty(&dry_run_mock_image)?
          );

          dry_run_mock_image
        } else {
          get_image_details_from_bos_sessiontemplate_yaml(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &hsm_group_available_vec,
            &image_reference,
            is_image_id,
          )
          .await?
        }
      } else {
        return Err(Error::Message(
          "ERROR: no 'image' section in session_template.\nExit".to_string(),
        ));
      };

    log::info!("Image with name '{}' found", image_details.name);

    // Get CFS configuration to configure the nodes
    let bos_session_template_configuration_name = bos_sessiontemplate_yaml
      .get("configuration")
      .and_then(Value::as_str)
      .map(str::to_string)
      .unwrap();

    // Check CFS configuration exists in CSM
    log::info!(
      "Looking for CFS configuration with name: {}",
      bos_session_template_configuration_name
    );

    if dry_run {
      println!(
        "Dry run mode: CFS configuration '{}' found in CSM.",
        bos_session_template_configuration_name
      );
    } else {
      cfs::configuration::http_client::v3::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&bos_session_template_configuration_name),
      )
      .await?;
    };

    // let ims_image_name = image_details.name.to_string();
    let ims_image_etag: &str = image_details
      .link
      .as_ref()
      .and_then(|link| link.etag.as_ref())
      .unwrap();
    let ims_image_path: &str = image_details
      .link
      .as_ref()
      .map(|link| link.path.as_ref())
      .unwrap();
    let ims_image_type: &str = image_details
      .link
      .as_ref()
      .map(|link| link.r#type.as_ref())
      .unwrap();

    let bos_sessiontemplate_name = bos_sessiontemplate_yaml
      .get("name")
      .and_then(Value::as_str)
      .map(str::to_string)
      .unwrap_or_default();

    let mut boot_set_vec: HashMap<String, BootSet> = HashMap::new();

    for (parameter, boot_set) in bos_sessiontemplate_yaml
      .get("bos_parameters")
      .and_then(|bos_parameters| bos_parameters.get("boot_sets"))
      .and_then(Value::as_mapping)
      .unwrap()
    {
      let kernel_parameters = boot_set
        .get("kernel_parameters")
        .and_then(Value::as_str)
        .unwrap();
      let arch_opt = boot_set
        .get("arch")
        .and_then(Value::as_str)
        .map(str::to_string);

      let node_roles_groups_opt: Option<Vec<String>> = boot_set
        .get("node_roles_groups")
        .and_then(Value::as_sequence)
        .and_then(|node_role_groups| {
          node_role_groups
            .iter()
            .map(|hsm_group_value| hsm_group_value.as_str().map(str::to_string))
            .collect()
        });

      // Validate/check user can create BOS sessiontemplates based on node roles. Users
      // with tenant role are not allowed to create BOS sessiontemplates based on node roles
      // however admin tenants are allowed to create BOS sessiontemplates based on node roles
      if !hsm_group_available_vec.is_empty()
        && node_roles_groups_opt
          .clone()
          .is_some_and(|node_roles_groups| !node_roles_groups.is_empty())
      {
        return Err(Error::Message(
          "User type tenant can't user node roles in BOS sessiontemplate. Exit"
            .to_string(),
        ));
      }

      let node_groups_opt: Option<Vec<String>> = boot_set
        .get("node_groups")
        .and_then(Value::as_sequence)
        .and_then(|node_group| {
          node_group
            .iter()
            .map(|hsm_group_value| hsm_group_value.as_str().map(str::to_string))
            .collect()
        });

      //FIXME: Get rid of this by making sure CSM admins don't create HSM groups for system
      //wide operations instead of using roles
      let node_groups_opt = node_groups_opt.map(|node_groups| {
        hsm::group::hacks::filter_system_hsm_group_names(node_groups)
      });

      // Validate/check HSM groups in YAML file session_templates.bos_parameters.boot_sets.<parameter>.node_groups matches with
      // Check hsm groups in SAT file includes the hsm_group_param
      for node_group in node_groups_opt.clone().unwrap_or_default() {
        if !hsm_group_available_vec.contains(&node_group) {
          return Err(Error::Message(format!(
            "User does not have access to HSM group '{}' in SAT file under session_templates.bos_parameters.boot_sets.compute.node_groups section. Exit",
            node_group
          )));
        }
      }

      // Validate user has access to the xnames in the BOS sessiontemplate
      let node_list_opt: Option<Vec<String>> = boot_set
        .get("node_list")
        .and_then(Value::as_sequence)
        .and_then(|node_list| {
          node_list
            .into_iter()
            .map(|node_value_value| {
              node_value_value.as_str().map(str::to_string)
            })
            .collect()
        });

      // Validate user has access to the list of nodes in BOS sessiontemplate
      if let Some(node_list) = &node_list_opt {
        validate_target_hsm_members(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          &node_list.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
        )
        .await?;
      }

      let cfs = Cfs {
        configuration: Some(bos_session_template_configuration_name.clone()),
      };

      let rootfs_provider = boot_set
        .get("rootfs_provider")
        .and_then(Value::as_str)
        .map(str::to_string);
      let rootfs_provider_passthrough = boot_set
        .get("rootfs_provider_passthrough")
        .and_then(Value::as_str)
        .map(str::to_string);

      let boot_set = BootSet {
        name: None,
        path: Some(ims_image_path.to_string()),
        r#type: Some(ims_image_type.to_string()),
        etag: Some(ims_image_etag.to_string()),
        kernel_parameters: Some(kernel_parameters.to_string()),
        node_list: node_list_opt,
        node_roles_groups: node_roles_groups_opt,
        node_groups: node_groups_opt,
        rootfs_provider,
        rootfs_provider_passthrough,
        cfs: Some(cfs),
        arch: arch_opt,
      };

      boot_set_vec
        .insert(parameter.as_str().map(str::to_string).unwrap(), boot_set);
    }

    let cfs = Cfs {
      configuration: Some(bos_session_template_configuration_name),
    };

    let create_bos_session_template_payload = BosSessionTemplate {
      name: None,
      description: None,
      enable_cfs: Some(true),
      cfs: Some(cfs),
      boot_sets: Some(boot_set_vec),
      links: None,
      tenant: None,
    };

    if dry_run {
      println!(
        "Dry run mode: Create BOS sessiontemplate:\n{}",
        serde_json::to_string_pretty(&create_bos_session_template_payload)?
      );

      // Generate a mock name for the BOS session template
      let dry_run_bos_sessiontemplate_name =
        format!("DRYRUN_{}", Uuid::new_v4().to_string());
      println!(
        "Dry Run Mode: BOS sessiontemplate name '{}' created",
        dry_run_bos_sessiontemplate_name
      );
      bos_st_created_vec.push(dry_run_bos_sessiontemplate_name);
    } else {
      let bos_sessiontemplate = bos::template::http_client::v2::put(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &create_bos_session_template_payload,
        &bos_sessiontemplate_name,
      )
      .await?;

      println!(
        "BOS sessiontemplate name '{}' created",
        bos_sessiontemplate_name
      );

      bos_st_created_vec.push(bos_sessiontemplate.name.unwrap())
    }
  }

  // Create BOS session. Note: reboot operation shuts down the nodes and they may not start
  // up... hence we will split the reboot into 2 operations shutdown and start

  if reboot {
    log::info!("Rebooting");

    for bos_st_name in bos_st_created_vec {
      log::info!(
        "Creating BOS session for BOS sessiontemplate '{}' with action 'reboot'",
        bos_st_name
      );

      // BOS session v2
      let bos_session = BosSession {
        name: None,
        tenant: None,
        operation: Some(Operation::Reboot),
        template_name: bos_st_name.clone(),
        limit: None,
        stage: None,
        include_disabled: None,
        status: None,
        components: None,
      };

      if dry_run {
        println!(
          "Dry run mode: Create BOS session:\n{}",
          serde_json::to_string_pretty(&bos_session)?
        );
      } else {
        bos::session::http_client::v2::post(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_session,
        )
        .await?;
      }
    }
  }

  // Audit
  let user = common::jwt_ops::get_name(shasta_token).unwrap();
  let username = common::jwt_ops::get_preferred_username(shasta_token).unwrap();

  log::info!(target: "app::audit", "User: {} ({}) ; Operation: Apply cluster", user, username);

  Ok(())
}

/// Returns image reference related to a session template in SAT file.
/// An image refenrece can be:
///     - image_name
///     - image_id
/// Image names are supposed to be fetched using 'get_fuzzy' function (so we increase the probablity of finding the image in CSM if it was created using 'sat bootprep --overwrite-images') while image ids can be fetched
/// by just 'get' function
/// This function returns a tuple with the image reference and a boolean indicating whether the image is
/// an image id or not
fn get_image_reference_from_bos_sessiontemplate_yaml(
  bos_sessiontemplate_image: &Value,
  ref_name_processed_hashmap: &HashMap<String, String>,
) -> Result<(String, bool), Error> {
  if let Some(bos_sessiontemplate_image_ims) =
    bos_sessiontemplate_image.get("ims")
  {
    // Get boot image to configure the nodes
    if let Some(bos_session_template_image_ims_name) =
      bos_sessiontemplate_image_ims.get("name")
    {
      // BOS sessiontemplate boot image defined by name
      let image_name = bos_session_template_image_ims_name
        .as_str()
        .map(str::to_string)
        .unwrap();

      Ok((image_name, false))
    } else if let Some(bos_session_template_image_ims_id) =
      bos_sessiontemplate_image_ims.get("id")
    {
      // BOS sessiontemplate boot image defined by id
      let image_id = bos_session_template_image_ims_id
        .as_str()
        .map(str::to_string)
        .unwrap();

      Ok((image_id, true))
    } else {
      return Err(Error::Message("ERROR: neither 'image.ims.name' nor 'image.ims.id' fields defined in session_template.".to_string()));
    }
  } else if let Some(bos_session_template_image_image_ref) =
    bos_sessiontemplate_image.get("image_ref")
  {
    // BOS sessiontemplate boot image defined by image_ref
    let image_ref = bos_session_template_image_image_ref
      .as_str()
      .map(str::to_string)
      .unwrap();

    let image_id = ref_name_processed_hashmap.get(&image_ref).cloned().unwrap();

    Ok((image_id, true))
  } else if let Some(image_name_substring) = bos_sessiontemplate_image.as_str()
  {
    let image_name = image_name_substring;
    // Backward compatibility
    // Get base image details

    Ok((image_name.to_string(), false))
  } else {
    return Err(Error::Message("ERROR: neither 'image.ims' nor 'image.image_ref' nor 'image.<image id>' sections found in session_template.image.\nExit".to_string()));
  }
}

async fn get_image_details_from_bos_sessiontemplate_yaml(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hsm_group_available_vec: &[String],
  image_reference: &str,
  is_image_id: bool,
) -> Result<ims::image::http_client::types::Image, Error> {
  let image = if is_image_id {
    ims::image::http_client::get(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      Some(&image_reference),
    )
    .await
    .map(|image_vec| image_vec.first().cloned().unwrap())
  } else {
    ims::image::utils::get_fuzzy(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_available_vec,
      Some(&image_reference),
      Some(&1),
    )
    .await
    .map(|image_vec| image_vec.first().cloned().unwrap())
  };

  image
}

/* async fn get_base_image_id_from_sat_file_image_yaml(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  image_yaml: &Value,
  ref_name_image_id_hashmap: &HashMap<String, String>,
  cray_product_catalog: &BTreeMap<String, String>,
  image_name: &String,
  dry_run: bool,
) -> Result<String, Error> {
  let base_image_id: String;

  // Get/process base image
  if let Some(sat_file_image_ims_value_yaml) = image_yaml.get("ims") {
    // ----------- BASE IMAGE - BACKWARD COMPATIBILITY WITH PREVIOUS SAT FILE
    log::info!("SAT file - 'image.ims' job ('images' section in SAT file is outdated - switching to backward compatibility)");

    base_image_id =
      process_sat_file_image_old_version(sat_file_image_ims_value_yaml)?;
  } else if let Some(sat_file_image_base_value_yaml) = image_yaml.get("base") {
    if let Some(sat_file_image_base_image_ref_value_yaml) =
      sat_file_image_base_value_yaml.get("image_ref")
    {
      log::info!("SAT file - 'image.base.image_ref' job");

      base_image_id = process_sat_file_image_ref_name(
        sat_file_image_base_image_ref_value_yaml,
        ref_name_image_id_hashmap,
      )?;
    } else if let Some(sat_file_image_base_ims_value_yaml) =
      sat_file_image_base_value_yaml.get("ims")
    {
      log::info!("SAT file - 'image.base.ims' job");
      let ims_job_type = sat_file_image_base_ims_value_yaml
        .get("type")
        .and_then(Value::as_str)
        .unwrap();
      if ims_job_type == "recipe" {
        log::info!("SAT file - 'image.base.ims' job of type 'recipe'");

        base_image_id = process_sat_file_image_ims_type_recipe(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          sat_file_image_base_ims_value_yaml,
          image_name,
          dry_run,
        )
        .await?;
      } else if ims_job_type == "image" {
        log::info!("SAT file - 'image.base.ims' job of type 'image'");

        base_image_id = sat_file_image_base_ims_value_yaml
          .get("id")
          .and_then(Value::as_str)
          .map(str::to_string)
          .unwrap();
      } else {
        return Err(Error::Message(
          "Can't process SAT file 'images.base.ims' is missing. Exit"
            .to_string(),
        ));
      }

    // ----------- BASE IMAGE - CRAY PRODUCT CATALOG
    } else if let Some(sat_file_image_base_product_value_yaml) =
      sat_file_image_base_value_yaml.get("product")
    {
      log::info!("SAT file - 'image.base.product' job");
      // Base image created from a cray product
      let product_name = sat_file_image_base_product_value_yaml
        .get("name")
        .and_then(Value::as_str)
        .unwrap();

      let product_version = sat_file_image_base_product_value_yaml
        .get("version")
        .and_then(Value::as_str)
        .unwrap();

      let product_type = sat_file_image_base_product_value_yaml
        .get("type")
        .and_then(Value::as_str)
        .map(|v| v.to_string() + "s")
        .unwrap();

      // We assume the SAT file has been alredy validated therefore taking some risks in
      // getting the details from the Cray product catalog
      let product_image_map = &serde_yaml::from_str::<serde_json::Value>(
        &cray_product_catalog[product_name],
      )?[product_version][product_type.clone()]
      .as_object()
      .cloned()
      .unwrap();

      let image_id = if let Some(filter) =
        sat_file_image_base_product_value_yaml.get("filter")
      {
        filter_product_catalog_images(
          filter,
          product_image_map.clone(),
          &image_name,
        )?
      } else {
        // There is no 'image.product.filter' value defined in SAT file. Check Cray
        // product catalog only has 1 image. Othewise fail
        log::info!("No 'image.product.filter' defined in SAT file. Checking Cray product catalog only/must have 1 image");
        product_image_map
          .values()
          .next()
          .and_then(|value| value.get("id"))
          .and_then(serde_json::Value::as_str)
          .map(str::to_string)
          .unwrap()
      };

      // ----------- BASE IMAGE - CRAY PRODUCT CATALOG TYPE RECIPE
      base_image_id = if product_type == "recipes" {
        // Create base image from an IMS job (the 'id' field in
        // images[].base.product.id is the id of the IMS recipe used to
        // build the new base image)

        log::info!("SAT file - 'image.base.product' job based on IMS recipes");

        let product_recipe_id = image_id.clone();

        process_sat_file_image_product_type_ims_recipe(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          &product_recipe_id,
          &image_name,
          dry_run,
        )
        .await?

        // ----------- BASE IMAGE - CRAY PRODUCT CATALOG TYPE IMAGE
      } else if product_type == "images" {
        // Base image already created and its id is available in the Cray
        // product catalog

        log::info!("SAT file - 'image.base.product' job based on IMS images");

        log::info!("Getting base image id from Cray product catalog");

        let product_image_id = image_id;

        product_image_id
      } else {
        return Err(Error::Message(
                    "Can't process SAT file, field 'images.base.product.type' must be either 'images' or 'recipes'. Exit".to_string(),
                ));
      }
    } else {
      return Err(Error::Message(
        "Can't process SAT file 'images.base.product' is missing. Exit"
          .to_string(),
      ));
    }
  } else {
    return Err(Error::Message(
      "Can't process SAT file 'images.base' is missing. Exit".to_string(),
    ));
  }

  Ok(base_image_id)
} */

async fn get_base_image_id_from_sat_file_image_yaml(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  // image_yaml: &Value,
  image_yaml: &image::Image,
  ref_name_image_id_hashmap: &HashMap<String, String>,
  cray_product_catalog: &BTreeMap<String, String>,
  image_name: &String,
  dry_run: bool,
) -> Result<String, Error> {
  // Get/process base image
  // if let Some(sat_file_image_ims_value_yaml) = image_yaml.get("ims") {
  let base_image_id: String = if let image::BaseOrIms::Ims { ims } =
    &image_yaml.base_or_ims
  {
    // ----------- BASE IMAGE - BACKWARD COMPATIBILITY WITH PREVIOUS SAT FILE
    log::info!(
      "SAT file - 'image.ims' job ('images' section in SAT file is outdated - switching to backward compatibility)"
    );

    process_sat_file_image_old_version_struct(&ims)?
  // } else if let Some(sat_file_image_base_value_yaml) = image_yaml.get("base") {
  } else if let image::BaseOrIms::Base { base } = &image_yaml.base_or_ims {
    /* if let Some(sat_file_image_base_image_ref_value_yaml) =
      sat_file_image_base_value_yaml.get("image_ref")
    { */
    if let image::Base::ImageRef { image_ref } = base {
      log::info!("SAT file - 'image.base.image_ref' job");

      image_ref.clone()
    /* } else if let Some(sat_file_image_base_ims_value_yaml) =
      sat_file_image_base_value_yaml.get("ims")
    { */
    } else if let image::Base::Ims { ims } = base {
      if let image::ImageBaseIms::NameType { name, r#type } = ims {
        log::info!("SAT file - 'image.base.ims' job");
        if r#type == "recipe" {
          log::info!("SAT file - 'image.base.ims' job of type 'recipe'");

          process_sat_file_image_ims_type_recipe(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            name,
            image_name,
            dry_run,
          )
          .await?
        } else {
          return Err(Error::Message(
            "Can't process SAT file 'images.base.ims' is missing. Exit"
              .to_string(),
          ));
        }
      } else if let image::Base::Ims { ims } = base {
        if let image::ImageBaseIms::IdType { id, r#type } = ims {
          if r#type == "image" {
            log::info!("SAT file - 'image.base.ims' job of type 'image'");

            id.to_string()
          } else {
            return Err(Error::Message(
              "Can't process SAT file 'images.base.ims' is missing. Exit"
                .to_string(),
            ));
          }
        } else {
          return Err(Error::Message(
            "Can't process SAT file 'images.base.ims' is missing. Exit"
              .to_string(),
          ));
        }
      } else {
        return Err(Error::Message(
          "Can't process SAT file 'images.base.ims' is missing. Exit"
            .to_string(),
        ));
      }
    // ----------- BASE IMAGE - CRAY PRODUCT CATALOG
    /* } else if let Some(sat_file_image_base_product_value_yaml) =
      sat_file_image_base_value_yaml.get("product")
    { */
    } else if let image::Base::Product { product } = base {
      log::info!("SAT file - 'image.base.product' job");
      // Base image created from a cray product
      let product_name = &product.name;

      let product_version = product.version.as_ref().unwrap();

      let product_type = &product.r#type;

      // We assume the SAT file has been alredy validated therefore taking some risks in
      // getting the details from the Cray product catalog
      let product_image_map = &serde_yaml::from_str::<serde_json::Value>(
        &cray_product_catalog[product_name],
      )?[product_version][product_type]
        .as_object()
        .cloned()
        .unwrap();

      let image_id = if let Some(filter) = product.filter.as_ref() {
        filter_product_catalog_images(
          filter,
          product_image_map.clone(),
          &image_name,
        )?
      } else {
        // There is no 'image.product.filter' value defined in SAT file. Check Cray
        // product catalog only has 1 image. Othewise fail
        log::info!(
          "No 'image.product.filter' defined in SAT file. Checking Cray product catalog only/must have 1 image"
        );
        product_image_map
          .values()
          .next()
          .and_then(|value| value.get("id"))
          .and_then(serde_json::Value::as_str)
          .map(str::to_string)
          .unwrap()
      };

      // ----------- BASE IMAGE - CRAY PRODUCT CATALOG TYPE RECIPE
      if product_type == "recipes" {
        // Create base image from an IMS job (the 'id' field in
        // images[].base.product.id is the id of the IMS recipe used to
        // build the new base image)

        log::info!("SAT file - 'image.base.product' job based on IMS recipes");

        let product_recipe_id = image_id.clone();

        process_sat_file_image_product_type_ims_recipe(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          &product_recipe_id,
          &image_name,
          dry_run,
        )
        .await?

        // ----------- BASE IMAGE - CRAY PRODUCT CATALOG TYPE IMAGE
      } else if product_type == "images" {
        // Base image already created and its id is available in the Cray
        // product catalog

        log::info!("SAT file - 'image.base.product' job based on IMS images");

        log::info!("Getting base image id from Cray product catalog");

        image_id
      } else {
        return Err(Error::Message(
          "Can't process SAT file, field 'images.base.product.type' must be either 'images' or 'recipes'. Exit".to_string(),
        ));
      }
    } else {
      return Err(Error::Message(
        "Can't process SAT file 'images.base.product' is missing. Exit"
          .to_string(),
      ));
    }
  } else {
    return Err(Error::Message(
      "Can't process SAT file 'images.base' is missing. Exit".to_string(),
    ));
  };

  Ok(base_image_id)
}
