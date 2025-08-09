use std::io;

use serde_json::Value;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("CSM-RS: {0}")]
  Message(String),
  #[error("CSM-RS > IO: {0}")]
  IoError(#[from] io::Error),
  #[error("CSM-RS > Serde: {0}")]
  SerdeError(#[from] serde_json::Error),
  #[error("CSM-RS > Net: {0}")]
  NetError(#[from] reqwest::Error),
  #[error("http request:\nresponse: {response}\npayload: {payload}")]
  RequestError {
    response: reqwest::Error,
    payload: String, // NOTE: CSM/OCHAMI Apis either returns plain text or a json therefore, we
                     // will just return a String
  },
  #[error("CSM-RS > CSM: {0}")]
  CsmError(Value),
  #[error("CSM-RS > Console: {0}")]
  ConsoleError(String),
  #[error("CSM-RS > K8s: {0}")]
  K8sError(String),
  #[error("CSM-RS > Image '{0}' not found")]
  ImageNotFound(String),
  #[error("CSM-RS > Group '{0}' not found")]
  GroupNotFound(String),
  #[error("ERROR - No derivatives found for CFS Configuration: {0}")]
  ConfigurationDerivativesNotFound(String),
  #[error("ERROR - CFS Configuration already exists: {0}")]
  ConfigurationAlreadyExists(String),
  #[error("ERROR - CFS Configuration used as a runtime configuration for a cluster and/or used to build an image used to boot node(s)")]
  ConfigurationUsedAsRuntimeConfigurationOrUsedToBuildBootImageUsed,
}
