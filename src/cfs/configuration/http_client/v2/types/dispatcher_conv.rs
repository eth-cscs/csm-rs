//! Bidirectional `From` impls between csm-rs's CFS v2 configuration types
//! and the dispatcher's mirrors. Gated behind the `manta-dispatcher`
//! Cargo feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::cfs::cfs_configuration_request::{
  CfsConfigurationRequest as FrontEndCfsConfigurationRequest,
  Layer as FrontEndLayer, SpecialParameter as FrontEndSpecialParameter,
};
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::{
  AdditionalInventory as FrontEndAdditionalInventory,
  CfsConfigurationResponse as FrontendCfsConfigurationResponse,
  CfsConfigurationVecResponse as FrontendCfsConfigurationVecResponse,
  Layer as FrontendLayer, Next as FrontendNext,
};

use super::cfs_configuration_request::{
  CfsConfigurationRequest, Layer as RequestLayer, SpecialParameter,
};
use super::cfs_configuration_response::{
  AdditionalInventory, CfsConfigurationResponse, CfsConfigurationVecResponse,
  Layer as ResponseLayer, Next,
};

impl From<FrontEndLayer> for RequestLayer {
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

impl From<RequestLayer> for FrontEndLayer {
  fn from(val: RequestLayer) -> Self {
    FrontEndLayer {
      name: Some(val.name),
      clone_url: Some(val.clone_url),
      playbook: val.playbook,
      commit: val.commit,
      branch: val.branch,
      source: None, // This field is not used in the backend
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

impl From<FrontEndCfsConfigurationRequest> for CfsConfigurationRequest {
  fn from(
    front_end_cfs_configuration_request: FrontEndCfsConfigurationRequest,
  ) -> Self {
    Self {
      layers: front_end_cfs_configuration_request
        .layers
        .unwrap_or_default()
        .into_iter()
        .map(RequestLayer::from)
        .collect(),
    }
  }
}

impl From<CfsConfigurationRequest> for FrontEndCfsConfigurationRequest {
  fn from(val: CfsConfigurationRequest) -> Self {
    FrontEndCfsConfigurationRequest {
      description: None,
      layers: Some(
        val.layers.into_iter().map(RequestLayer::into).collect(),
      ),
      additional_inventory: None,
    }
  }
}

impl From<FrontendLayer> for ResponseLayer {
  fn from(frontend_layer: FrontendLayer) -> Self {
    Self {
      name: frontend_layer.name,
      clone_url: frontend_layer.clone_url,
      commit: frontend_layer.commit,
      playbook: frontend_layer.playbook,
      branch: frontend_layer.branch,
    }
  }
}

impl From<ResponseLayer> for FrontendLayer {
  fn from(val: ResponseLayer) -> Self {
    FrontendLayer {
      name: val.name,
      clone_url: val.clone_url,
      source: None,
      commit: val.commit,
      playbook: val.playbook,
      branch: val.branch,
    }
  }
}

impl From<FrontEndAdditionalInventory> for AdditionalInventory {
  fn from(value: FrontEndAdditionalInventory) -> Self {
    Self {
      clone_url: value.clone_url,
      commit: value.commit,
      name: value.name,
      branch: value.branch,
    }
  }
}

impl From<AdditionalInventory> for FrontEndAdditionalInventory {
  fn from(val: AdditionalInventory) -> Self {
    FrontEndAdditionalInventory {
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
        .map(AdditionalInventory::from),
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
