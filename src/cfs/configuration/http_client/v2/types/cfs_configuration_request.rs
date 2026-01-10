use manta_backend_dispatcher::types::cfs::cfs_configuration_request::{
  CfsConfigurationRequest as FrontEndCfsConfigurationRequest,
  Layer as FrontEndLayer, SpecialParameter as FrontEndSpecialParameter,
};

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::{common::gitea, error::Error};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Layer {
  pub name: String,
  #[serde(rename = "cloneUrl")]
  pub clone_url: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  // Either commit or branch is passed
  pub commit: Option<String>,
  playbook: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  // Either commit or branch is passed
  pub branch: Option<String>,
  // #[serde(skip_serializing_if = "Option::is_none")]
  // pub tag: Option<String>,
  #[serde(rename = "specialParameters")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub special_parameters: Option<Vec<SpecialParameter>>,
}

impl From<FrontEndLayer> for Layer {
  fn from(front_end_layer: FrontEndLayer) -> Self {
    Self {
      name: front_end_layer.name.unwrap_or_default(),
      clone_url: front_end_layer.clone_url.unwrap_or_default(),
      playbook: front_end_layer.playbook,
      commit: front_end_layer.commit,
      branch: front_end_layer.branch,
      // tag: front_end_layer.tag,
      special_parameters: front_end_layer.special_parameters.map(
        |special_parameters| {
          special_parameters
            .into_iter()
            .map(|special_parameter| special_parameter.into())
            .collect()
        },
      ),
    }
  }
}

impl Into<FrontEndLayer> for Layer {
  fn into(self) -> FrontEndLayer {
    FrontEndLayer {
      name: Some(self.name),
      clone_url: Some(self.clone_url),
      playbook: self.playbook,
      commit: self.commit,
      branch: self.branch,
      source: None, // This field is not used in the backend
      special_parameters: self.special_parameters.map(|special_parameters| {
        special_parameters
          .into_iter()
          .map(|special_parameter| special_parameter.into())
          .collect()
      }),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecialParameter {
  #[serde(rename = "imsRequiredDkms")]
  #[serde(skip_serializing_if = "Option::is_none")]
  ims_required_dkms: Option<bool>,
}

impl From<FrontEndSpecialParameter> for SpecialParameter {
  fn from(front_end_special_parameter: FrontEndSpecialParameter) -> Self {
    Self {
      ims_required_dkms: front_end_special_parameter.ims_required_dkms,
    }
  }
}

impl Into<FrontEndSpecialParameter> for SpecialParameter {
  fn into(self) -> FrontEndSpecialParameter {
    FrontEndSpecialParameter {
      ims_required_dkms: self.ims_required_dkms,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CfsConfigurationRequest {
  pub name: String,
  pub layers: Vec<Layer>,
}

impl From<FrontEndCfsConfigurationRequest> for CfsConfigurationRequest {
  fn from(
    front_end_cfs_configuration_request: FrontEndCfsConfigurationRequest,
  ) -> Self {
    Self {
      name: "".to_string(), // FIXME: add 'name' field to the frontend request and change
      // this code to `name: front_end_cfs_configuration_response.name,`
      layers: front_end_cfs_configuration_request
        .layers
        .unwrap_or_default()
        .into_iter()
        .map(Layer::from)
        .collect(),
    }
  }
}

impl Into<FrontEndCfsConfigurationRequest> for CfsConfigurationRequest {
  fn into(self) -> FrontEndCfsConfigurationRequest {
    FrontEndCfsConfigurationRequest {
      description: None,
      layers: Some(self.layers.into_iter().map(Layer::into).collect()),
      additional_inventory: None,
    }
  }
}

impl Layer {
  pub fn new(
    clone_url: String,
    commit: Option<String>,
    name: String,
    playbook: String,
    branch: Option<String>,
    _tag: Option<String>,
    special_parameters: Option<Vec<SpecialParameter>>,
  ) -> Self {
    Self {
      clone_url,
      commit,
      name,
      playbook,
      branch,
      // tag,
      special_parameters,
    }
  }
}

impl Default for CfsConfigurationRequest {
  fn default() -> Self {
    Self::new()
  }
}

impl CfsConfigurationRequest {
  pub fn new() -> Self {
    Self {
      name: String::default(),
      layers: Vec::default(),
    }
  }

  pub fn add_layer(&mut self, layer: Layer) {
    self.layers.push(layer);
  }

  pub async fn from_sat_file_serde_yaml(
    shasta_root_cert: &[u8],
    gitea_base_url: &str,
    gitea_token: &str,
    configuration_yaml: &serde_yaml::Value,
    cray_product_catalog: &BTreeMap<String, String>,
    site_name: &str,
  ) -> Result<(String, Self), Error> {
    let mut cfs_configuration = Self::new();

    let cfs_configuration_name = configuration_yaml
      .get("name")
      .and_then(Value::as_str)
      .map(str::to_string)
      .unwrap();

    cfs_configuration.name = cfs_configuration_name.clone();

    for layer_yaml in configuration_yaml
      .get("layers")
      .and_then(Value::as_sequence)
      .unwrap()
    {
      // println!("\n\n### Layer:\n{:#?}\n", layer_json);

      if layer_yaml.get("git").is_some() {
        // Git layer

        let layer_name = layer_yaml
          .get("name")
          .and_then(Value::as_str)
          .map(str::to_string)
          .unwrap();

        let repo_url = layer_yaml
          .get("git")
          .and_then(|git| git.get("url"))
          .and_then(Value::as_str)
          .map(str::to_string)
          .unwrap();

        let commit_id_value_opt =
          layer_yaml.get("git").and_then(|git| git.get("commit"));
        let tag_value_opt =
          layer_yaml.get("git").and_then(|git| git.get("tag"));
        let branch_value_opt =
          layer_yaml.get("git").and_then(|git| git.get("branch"));

        let commit_id_opt: Option<String> = if commit_id_value_opt.is_some() {
          // Git commit id
          layer_yaml
            .get("git")
            .and_then(|git| git.get("commit"))
            .and_then(Value::as_str)
            .map(str::to_string)
        } else if let Some(git_tag_value) = tag_value_opt {
          // Git tag
          let git_tag = git_tag_value.as_str().unwrap();

          log::info!("git tag: {}", git_tag_value.as_str().unwrap());

          let tag_details_rslt = gitea::http_client::get_tag_details(
            &repo_url,
            git_tag,
            gitea_token,
            shasta_root_cert,
            site_name,
          )
          .await;

          let tag_details = if let Ok(tag_details) = tag_details_rslt {
            log::debug!("tag details:\n{:#?}", tag_details);
            tag_details
          } else {
            return Err(Error::Message(
              format!("ERROR - Could not get details for git tag '{}' in CFS configuration '{}'. Reason:\n{:#?}", git_tag, cfs_configuration_name, tag_details_rslt)
            ));
          };

          // Assumming user sets an existing tag name. It could be an annotated tag
          // (different object than the commit id with its own sha value) or a
          // lightweight tag (pointer to commit id, therefore the tag will have the
          // same sha as the commit id it points to), either way CFS session will
          // do a `git checkout` to the sha we found here, if an annotated tag, then,
          // git is clever enough to take us to the final commit id, if it is a
          // lighweight tag, then there is no problem because the sha is the same
          // as the commit id
          // NOTE: the `id` field is the tag's sha, note we are not taking the commit id
          // the tag points to and we should not use sha because otherwise we won't be
          // able to fetch the annotated tag using a commit sha through the Gitea APIs
          tag_details
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
        } else if branch_value_opt.is_some() {
          // Branch name
          Some(
            gitea::http_client::get_commit_pointed_by_branch(
              gitea_base_url,
              gitea_token,
              shasta_root_cert,
              &repo_url,
              branch_value_opt.and_then(Value::as_str).unwrap(),
            )
            .await?,
          )
        } else {
          // This should be an error but we will let CSM to handle this
          None
        };

        // IMPORTANT: CSM won't allow CFS configuration layers with both commit id and
        // branch name, therefore, we will set branch name to None if we already have a
        // commit id
        let branch_name = if commit_id_opt.is_some() {
          None
        } else {
          branch_value_opt.and_then(Value::as_str).map(str::to_string)
        };

        let layer = Layer::new(
          repo_url,
          commit_id_opt,
          layer_name,
          layer_yaml
            .get("playbook")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_default(),
          branch_name,
          None,
          None,
        );
        cfs_configuration.add_layer(layer);
      } else if layer_yaml.get("product").is_some() {
        // Product layer

        let product_name = layer_yaml
          .get("product")
          .and_then(|product| product.get("name"))
          .and_then(Value::as_str)
          .unwrap();
        let product_version = layer_yaml
          .get("product")
          .and_then(|product| product.get("version"))
          .and_then(Value::as_str)
          .unwrap();
        let product_branch_value_opt = layer_yaml
          .get("product")
          .and_then(|product| product.get("branch"));

        let product = cray_product_catalog.get(product_name);

        if product.is_none() {
          return Err(Error::Message(format!(
            "Product {} not found in cray product catalog",
            product_name
          )));
        }

        let cos_cray_product_catalog =
          serde_yaml::from_str::<Value>(product.unwrap()).unwrap();

        let product_details_opt = cos_cray_product_catalog
          .get(product_version)
          .and_then(|product| product.get("configuration"));

        if product_details_opt.is_none() {
          return Err(Error::Message(format!(
            "Product details for product name '{}', product_version '{}' and 'configuration' not found in cray product catalog", product_name, product_version)
          ));
        }

        let product_details = product_details_opt.cloned().unwrap();

        log::debug!(
          "CRAY product catalog details for product: {}, version: {}:\n{:#?}",
          product_name,
          product_version,
          product_details
        );

        // Manta may run outside the CSM local network therefore we have to change the
        // internal URLs for the external one
        let repo_url = product_details
          .get("clone_url")
          .and_then(Value::as_str)
          .map(str::to_string)
          .map(|url| {
            url.replace(
              format!("vcs.cmn.{}.cscs.ch", site_name).as_str(),
              "api-gw-service-nmn.local",
            )
          })
          .unwrap();

        let commit_id_opt = if product_branch_value_opt.is_some() {
          // If branch is provided, then ignore the commit id in the CRAY products table

          let commit = Some(
            gitea::http_client::get_commit_pointed_by_branch(
              gitea_base_url,
              gitea_token,
              shasta_root_cert,
              &repo_url,
              product_branch_value_opt.and_then(Value::as_str).unwrap(),
            )
            .await?,
          );

          commit
        } else {
          product_details
            .get("commit")
            .and_then(Value::as_str)
            .map(str::to_string)
        };

        // IMPORTANT: CSM won't allow CFS configuration layers with both commit id and
        // branch name, therefore, we will set branch name to None if we already have a
        // commit id
        let branch_name = if commit_id_opt.is_some() {
          None
        } else {
          product_branch_value_opt
            .and_then(Value::as_str)
            .map(str::to_string)
        };

        // Create CFS configuration layer struct
        let layer = Layer::new(
          repo_url,
          commit_id_opt,
          product_name.to_string(),
          layer_yaml
            .get("playbook")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap(),
          branch_name,
          None,
          None,
        );
        cfs_configuration.add_layer(layer);
      } else {
        return Err(Error::Message(
           format!("ERROR - configurations section in SAT file error - CFS configuration layer error")
        ));
      }
    }

    Ok((cfs_configuration_name, cfs_configuration))
  }
}
