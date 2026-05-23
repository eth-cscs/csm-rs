use std::{env::VarError, io, num::ParseIntError, str::Utf8Error};

use aws_smithy_types::byte_stream;
use globset::Error as GlobsetError;
use manta_backend_dispatcher::error::Error as MantaError;
use serde_json::Value;
use tokio::task::JoinError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("CSM-RS > Generic error: {0}")]
  Message(String),
  #[error("CSM-RS > Environment variable: {0}")]
  EnvVarError(#[from] VarError),
  #[error("CSM-RS > IO: {0}")]
  IoError(#[from] io::Error),
  #[error("CSM-RS > Serde JSON: {0}")]
  SerdeJsonError(#[from] serde_json::Error),
  #[error("CSM-RS > Serde YAML: {0}")]
  SerdeYamlError(#[from] serde_yaml::Error),
  #[error("CSM-RS > Net: {0}")]
  NetError(#[from] reqwest::Error),
  #[error("CSM-RS > Tokio: {0}")]
  TokioError(#[from] JoinError),
  #[error("CSM-RS > Error converting from UTF8 to String: {0}")]
  UtfError(#[from] Utf8Error),
  #[error("CSM-RS > Glob error: {0}")]
  GlobError(#[from] GlobsetError),
  #[error("CSM-RS > Parse int error: {0}")]
  ParseStrIntError(#[from] ParseIntError),
  #[error("CSM-RS > URL parse error: {0}")]
  SmithyDataStreamError(#[from] byte_stream::error::Error),
  #[error("http request:\nresponse: {response}\npayload: {payload}")]
  RequestError {
    response: reqwest::Error,
    payload: String, // NOTE: CSM/OCHAMI Apis either returns plain text or a json therefore, we
                     // will just return a String
  },
  #[error("CSM-RS > CSM: {}", .0.get("detail").and_then(|detail| detail.as_str()).unwrap_or("Unknown error"))]
  CsmError(Value),
  #[error("CSM-RS > Console: {0}")]
  ConsoleError(String),
  #[error("CSM-RS > K8s: {0}")]
  K8sError(String),
  #[error("CSM-RS > K8s: field '{0}' missing in k8s credentials")]
  K8sCredentialMissingError(String),
  #[error("CSM-RS > K8s: '{0}' value not a string")]
  K8sCredentialNotStringError(String),
  #[error("CSM-RS > K8s: {0}")]
  K8sExecError(#[from] kube::Error),
  #[error("CSM-RS > CFS Session")]
  ImageNotFound(String),
  #[error("CSM-RS > Group '{0}' not found")]
  GroupNotFound(String),
  #[error("CSM-RS > No derivatives found for CFS Configuration: {0}")]
  ConfigurationDerivativesNotFound(String),
  #[error("CSM-RS > Configuration '{0}' does not have a name defined")]
  ConfigurationNameNotDefined(String),
  #[error("CSM-RS > CFS Configuration already exists: {0}")]
  ConfigurationAlreadyExists(String),
  #[error(
    "CSM-RS > CFS Configuration used as a runtime configuration for a cluster and/or used to build an image used to boot node(s)"
  )]
  ConfigurationUsedAsRuntimeConfigurationOrUsedToBuildBootImageUsed,
  #[error("CSM-RS > Session '{0}' not found")]
  SessionNotFound(String),
  #[error("CSM-RS > Session '{0}' does not have a name defined")]
  SessionNameNotDefined(String),
  #[error("CSM-RS > Session '{0}' does not have a configuration defined")]
  SessionConfigurationNotDefined(String),
  #[error("CSM-RS > IMS key '{0}' not found")]
  ImsKeyNotFound(String),
  #[error("CSM-RS > HSM component '{0}' not found")]
  HsmComponentNotFound(String),
  #[error("CSM-RS > HSM component '{0}' does not have a ID defined")]
  HsmComponentIdNotDefined(String),
  #[error("CSM-RS > HSM component '{0}' does not have a NID defined")]
  HsmComponentNidNotDefined(String),
  #[error("CSM-RS > HSM component '{0}' does not have a power state defined")]
  HsmComponentPowerStateNotDefined(String),
  #[error("CSM-RS > HSM component '{0}' does not have a '{1}' defined")]
  HsmComponentFieldNotDefined(String, String),
  #[error("CSM-RS > CFS component field '{0}' not defined")]
  CfsComponentFieldNotDefined(String),
  #[error("CSM-RS > CFS component does not have a 'name' defined")]
  CfsComponentNameFieldNotDefined(),
  #[error("CSM-RS > CFS component does not have a 'desired_conf' defined")]
  CfsComponentDesiredConfFieldNotDefined(),
}

// Convert Error to manta_backend_dispatcher::error::Error
impl From<crate::error::Error> for MantaError {
  fn from(val: crate::error::Error) -> Self {
    match val {
      Error::IoError(e) => MantaError::IoError(e),
      Error::SerdeJsonError(e) => MantaError::SerdeError(e),
      Error::NetError(e) => MantaError::NetError(e),
      Error::RequestError { response, payload } => {
        MantaError::RequestError { response, payload }
      }
      Error::CsmError(v) => MantaError::CsmError(v),
      // Variants with direct dispatcher equivalents
      Error::Message(s) => MantaError::Message(s),
      Error::ConsoleError(s) => MantaError::ConsoleError(s),
      Error::ConfigurationAlreadyExists(s) => MantaError::ConfigurationAlreadyExistsError(s),
      Error::SessionNotFound(_) => MantaError::SessionNotFound,
      Error::ConfigurationUsedAsRuntimeConfigurationOrUsedToBuildBootImageUsed => {
        MantaError::Conflict(
          Error::ConfigurationUsedAsRuntimeConfigurationOrUsedToBuildBootImageUsed
            .to_string(),
        )
      }
      // Not-found variants mapped to NotFound
      Error::ImageNotFound(s) => MantaError::NotFound(format!("Image '{s}'")),
      Error::GroupNotFound(s) => MantaError::NotFound(format!("Group '{s}'")),
      Error::HsmComponentNotFound(s) => MantaError::NotFound(format!("HSM component '{s}'")),
      Error::ImsKeyNotFound(s) => MantaError::NotFound(format!("IMS key '{s}'")),
      Error::ConfigurationDerivativesNotFound(s) => {
        MantaError::NotFound(format!("No derivatives for CFS configuration '{s}'"))
      }
      // K8s variants collapsed to K8sError
      Error::K8sError(s) => MantaError::K8sError(s),
      Error::K8sCredentialMissingError(s) => {
        MantaError::K8sError(format!("field '{s}' missing in k8s credentials"))
      }
      Error::K8sCredentialNotStringError(s) => {
        MantaError::K8sError(format!("'{s}' value not a string"))
      }
      Error::K8sExecError(e) => MantaError::K8sError(e.to_string()),
      // Technical/internal errors — preserve message text
      _ => MantaError::Message(val.to_string()),
    }
  }
}
