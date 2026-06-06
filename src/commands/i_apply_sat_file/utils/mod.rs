//! Section-level helpers for the SAT-file workflow: SAT-file shape,
//! conversions from SAT sections to BOS/CFS/IMS shapes, and per-section
//! orchestration submodules.

use std::collections::HashMap;

use crate::{
  bos::{BootSet, BosSessionTemplate, Cfs},
  commands::i_apply_sat_file::utils::sessiontemplate::Arch,
  error::Error,
};
use image::Image;
use serde::{Deserialize, Serialize};

use self::sessiontemplate::SessionTemplate;

/// One entry in the SAT file's `hardware` section: either a component
/// pattern (`pattern`, applied via `apply_hw_cluster_pin`) or an
/// explicit node-membership update (`nodespattern`). Exactly one of
/// the two should be set; the workflow checks `pattern` first.
#[derive(Deserialize, Serialize, Debug)]
pub struct HardwarePattern {
  /// HSM group to pin / membership-update against.
  pub target: String,
  /// HSM group from which to draw members.
  pub parent: String,
  /// Component pattern to evaluate against `parent` and apply to
  /// `target`. When present, takes precedence over `nodespattern`.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pattern: Option<String>,
  /// Comma-separated explicit xname list to add to `target` (members
  /// of `target` are preserved; only the diff is applied).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub nodespattern: Option<String>,
}

/// Deserialised representation of a SAT (System Admin Toolkit) YAML
/// file: up to four top-level sections describing HSM group hardware
/// patterns, CFS configurations to create, IMS images to build, and
/// BOS session templates to apply.
#[derive(Deserialize, Serialize, Debug)]
pub struct SatFile {
  /// HSM group hardware patterns (mirrors the SAT `hardware`
  /// section).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hardware: Option<Vec<HardwarePattern>>,
  /// CFS configurations to create (mirrors the SAT `configurations`
  /// section).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub configurations: Option<Vec<configuration::Configuration>>,
  /// IMS images to build (mirrors the SAT `images` section).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub images: Option<Vec<image::Image>>,
  /// BOS session templates to apply (mirrors the SAT
  /// `session_templates` section).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub session_templates: Option<Vec<sessiontemplate::SessionTemplate>>,
}

impl SatFile {
  /// Filter either images or `session_templates` section according to user request
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
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
      if let Some(configuration_vec) = self.configurations.as_mut() {
        configuration_vec.retain(|configuration| {
          configuration_name_image_vec.contains(&configuration.name)
        });
      }

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
            });
          })
          .unwrap_or_default();
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
                sessiontemplate::Image::ImageRef { image_ref: name } => {
                  Some(name)
                }
                sessiontemplate::Image::Ims { ims } => match ims {
                  sessiontemplate::ImsDetails::Name { name } => Some(name),
                  sessiontemplate::ImsDetails::Id { .. } => None,
                },
                sessiontemplate::Image::ImageName(name) => Some(name),
              }
            })
          }
          .cloned()
          .collect()
        })
        .unwrap_or_default();

      // Remove images not used by any sessiontemplate
      if let Some(image_vec) = self.images.as_mut() {
        image_vec.retain(|sat_image| {
          image_name_sessiontemplate_vec.contains(&sat_image.name)
        });
      }

      if self.images.as_ref().is_some_and(std::vec::Vec::is_empty) {
        self.images = None;
      }
    }

    Ok(())
  }
}

/// struct to represent the `session_templates` section in SAT file
pub mod sessiontemplate;

/// Convert from `sessiontemplate` in SAT file to mesa `BosSessionTemplate`.
///
/// Example from <https://doc.rust-lang.org/rust-by-example/conversion/try_from_try_into.html>.
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
          "Boot set property '{property}' created by manta from SAT file"
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
      description: Some(
        "BOS sessiontemplate created by manta from SAT file".to_string(),
      ),
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
pub mod image;

/// struct to represent the `configurations` section in SAT file
pub mod configuration;


/// CFS configuration creation helpers driven by a SAT file's
/// `configurations` section.
pub(crate) mod configurations;
/// IMS image build helpers driven by a SAT file's `images` section.
pub(crate) mod images;
/// BOS session template creation helpers driven by a SAT file's
/// `session_templates` section.
pub(crate) mod session_templates;

// Re-export the orchestration helpers actually called through
// `utils::name` at the original paths. Restricted to `pub(crate)` —
// these are stages of the SAT-file apply workflow, not building blocks
// an external consumer should depend on. Public entry points are
// `SatFile`, `HardwarePattern`, and `command::exec` (plus
// `Csm::apply_sat_file`). Helpers that are reached through their
// submodule path (e.g. `utils::images::i_create_image_*` from
// `backend_connector/sat.rs`) or only used inside the leaf submodule
// are intentionally not re-exported here.
pub(crate) use configurations::{
  create_cfs_configuration_from_sat_file,
  validate_sat_file_configurations_section,
};

pub(crate) use images::{
  i_import_images_section_in_sat_file, validate_sat_file_images_section,
};
pub(crate) use session_templates::{
  process_session_template_section_in_sat_file,
  validate_sat_file_session_template_section,
};

