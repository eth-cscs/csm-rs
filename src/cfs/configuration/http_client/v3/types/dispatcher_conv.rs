//! Bidirectional `From` impls between csm-rs's CFS v3 configuration types
//! and the dispatcher's mirrors. Gated behind the `manta-dispatcher`
//! Cargo feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::cfs::cfs_configuration_details::LayerDetails as FrontendLayerDetails;
use manta_backend_dispatcher::types::cfs::cfs_configuration_request::{
  AdditionalInventory as FrontEndRequestAdditionalInventory,
  CfsConfigurationRequest as FrontEndCfsConfigurationRequest,
  Layer as FrontEndRequestLayer,
  SpecialParameter as FrontEndSpecialParameter,
};
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::{
  AdditionalInventory as FrontEndResponseAdditionalInventory,
  CfsConfigurationResponse as FrontendCfsConfigurationResponse,
  CfsConfigurationVecResponse as FrontendCfsConfigurationVecResponse,
  Layer as FrontendResponseLayer, Next as FrontendNext,
};

use super::cfs_configuration::LayerDetails;
use super::cfs_configuration_request::{
  AdditionalInventory as RequestAdditionalInventory, CfsConfigurationRequest,
  Layer as RequestLayer, SpecialParameter,
};
use super::cfs_configuration_response::{
  AdditionalInventory as ResponseAdditionalInventory, CfsConfigurationResponse,
  CfsConfigurationVecResponse, Layer as ResponseLayer, Next,
};

impl From<FrontendLayerDetails> for LayerDetails {
  fn from(frontend_layer_details: FrontendLayerDetails) -> Self {
    Self {
      name: frontend_layer_details.name,
      repo_name: frontend_layer_details.repo_name,
      commit_id: frontend_layer_details.commit_id,
      author: frontend_layer_details.author,
      commit_date: frontend_layer_details.commit_date,
      branch: frontend_layer_details.branch,
      tag: frontend_layer_details.tag,
      playbook: frontend_layer_details.playbook,
    }
  }
}

impl From<LayerDetails> for FrontendLayerDetails {
  fn from(val: LayerDetails) -> Self {
    FrontendLayerDetails {
      name: val.name,
      repo_name: val.repo_name,
      commit_id: val.commit_id,
      author: val.author,
      commit_date: val.commit_date,
      branch: val.branch,
      tag: val.tag,
      playbook: val.playbook,
    }
  }
}

impl From<FrontEndRequestLayer> for RequestLayer {
  fn from(front_end_layer: FrontEndRequestLayer) -> Self {
    Self {
      name: front_end_layer.name,
      clone_url: front_end_layer.clone_url,
      source: front_end_layer.source,
      playbook: front_end_layer.playbook,
      commit: front_end_layer.commit,
      branch: front_end_layer.branch,
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

impl From<RequestLayer> for FrontEndRequestLayer {
  fn from(val: RequestLayer) -> Self {
    FrontEndRequestLayer {
      name: val.name,
      clone_url: val.clone_url,
      source: val.source,
      playbook: val.playbook,
      commit: val.commit,
      branch: val.branch,
      special_parameters: val.special_parameters.map(|special_parameters| {
        special_parameters
          .into_iter()
          .map(|special_parameter| special_parameter.into())
          .collect()
      }),
    }
  }
}

impl From<FrontEndSpecialParameter> for SpecialParameter {
  fn from(front_end_special_parameter: FrontEndSpecialParameter) -> Self {
    Self {
      ims_required_dkms: front_end_special_parameter.ims_required_dkms,
    }
  }
}

impl From<SpecialParameter> for FrontEndSpecialParameter {
  fn from(val: SpecialParameter) -> Self {
    FrontEndSpecialParameter {
      ims_required_dkms: val.ims_required_dkms,
    }
  }
}

impl From<FrontEndRequestAdditionalInventory> for RequestAdditionalInventory {
  fn from(
    front_end_additional_inventory: FrontEndRequestAdditionalInventory,
  ) -> Self {
    Self {
      name: front_end_additional_inventory.name,
      clone_url: front_end_additional_inventory.clone_url,
      source: front_end_additional_inventory.source,
      commit: front_end_additional_inventory.commit,
      branch: front_end_additional_inventory.branch,
    }
  }
}

impl From<RequestAdditionalInventory> for FrontEndRequestAdditionalInventory {
  fn from(val: RequestAdditionalInventory) -> Self {
    FrontEndRequestAdditionalInventory {
      name: val.name,
      clone_url: val.clone_url,
      source: val.source,
      commit: val.commit,
      branch: val.branch,
    }
  }
}

impl From<FrontEndCfsConfigurationRequest> for CfsConfigurationRequest {
  fn from(
    front_end_cfs_configuration_request: FrontEndCfsConfigurationRequest,
  ) -> Self {
    Self {
      description: front_end_cfs_configuration_request.description,
      layers: front_end_cfs_configuration_request
        .layers
        .map(|layer_vec| {
          layer_vec.into_iter().map(RequestLayer::from).collect()
        }),
      additional_inventory: front_end_cfs_configuration_request
        .additional_inventory
        .map(|additional_inventory| additional_inventory.into()),
    }
  }
}

impl From<CfsConfigurationRequest> for FrontEndCfsConfigurationRequest {
  fn from(val: CfsConfigurationRequest) -> Self {
    FrontEndCfsConfigurationRequest {
      description: val.description,
      layers: val.layers.map(|layer_vec| {
        layer_vec.into_iter().map(RequestLayer::into).collect()
      }),
      additional_inventory: val
        .additional_inventory
        .map(|additional_inventory| additional_inventory.into()),
    }
  }
}

impl From<FrontendResponseLayer> for ResponseLayer {
  fn from(frontend_layer: FrontendResponseLayer) -> Self {
    Self {
      name: frontend_layer.name,
      clone_url: frontend_layer.clone_url,
      source: frontend_layer.source,
      commit: frontend_layer.commit,
      playbook: frontend_layer.playbook,
      branch: frontend_layer.branch,
    }
  }
}

impl From<ResponseLayer> for FrontendResponseLayer {
  fn from(val: ResponseLayer) -> Self {
    FrontendResponseLayer {
      name: val.name,
      clone_url: val.clone_url,
      source: val.source,
      commit: val.commit,
      playbook: val.playbook,
      branch: val.branch,
    }
  }
}

impl From<FrontEndResponseAdditionalInventory> for ResponseAdditionalInventory {
  fn from(value: FrontEndResponseAdditionalInventory) -> Self {
    Self {
      clone_url: value.clone_url,
      commit: value.commit,
      name: value.name,
      branch: value.branch,
    }
  }
}

impl From<ResponseAdditionalInventory> for FrontEndResponseAdditionalInventory {
  fn from(val: ResponseAdditionalInventory) -> Self {
    FrontEndResponseAdditionalInventory {
      clone_url: val.clone_url,
      commit: val.commit,
      name: val.name,
      branch: val.branch,
    }
  }
}

impl From<FrontendCfsConfigurationResponse> for CfsConfigurationResponse {
  fn from(value: FrontendCfsConfigurationResponse) -> Self {
    CfsConfigurationResponse {
      name: value.name,
      last_updated: value.last_updated,
      layers: value.layers.into_iter().map(ResponseLayer::from).collect(),
      additional_inventory: value
        .additional_inventory
        .map(ResponseAdditionalInventory::from),
    }
  }
}

impl From<CfsConfigurationResponse> for FrontendCfsConfigurationResponse {
  fn from(val: CfsConfigurationResponse) -> Self {
    FrontendCfsConfigurationResponse {
      name: val.name,
      last_updated: val.last_updated,
      layers: val.layers.into_iter().map(Into::into).collect(),
      additional_inventory: val.additional_inventory.map(Into::into),
    }
  }
}

impl From<FrontendCfsConfigurationVecResponse> for CfsConfigurationVecResponse {
  fn from(value: FrontendCfsConfigurationVecResponse) -> Self {
    CfsConfigurationVecResponse {
      configurations: value
        .configurations
        .into_iter()
        .map(CfsConfigurationResponse::from)
        .collect(),
      next: value.next.map(Next::from),
    }
  }
}

impl From<CfsConfigurationVecResponse> for FrontendCfsConfigurationVecResponse {
  fn from(val: CfsConfigurationVecResponse) -> Self {
    FrontendCfsConfigurationVecResponse {
      configurations: val.configurations.into_iter().map(Into::into).collect(),
      next: val.next.map(Into::into),
    }
  }
}

impl From<FrontendNext> for Next {
  fn from(value: FrontendNext) -> Self {
    Next {
      limit: value.limit,
      after_id: value.after_id,
      in_use: value.in_use,
    }
  }
}

impl From<Next> for FrontendNext {
  fn from(val: Next) -> Self {
    FrontendNext {
      limit: val.limit,
      after_id: val.after_id,
      in_use: val.in_use,
    }
  }
}
