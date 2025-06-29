#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ERROR - CFS Configuration already exists: {0}")]
    ConfigurationAlreadyExistsError(String),
    #[error("ERROR - CFS Configuration network: {0}")]
    NetError(#[from] reqwest::Error),
    #[error("ERROR - CSM Configuration: {0}")]
    JsonError(serde_json::Value),
    #[error("ERROR - CSM Configuration: {0}")]
    TextError(String),
    #[error("ERROR - CFS Configuration message: {0}")]
    MessageError(String),
}
