use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SshContainer {
  pub name: String,
  pub jail: bool,
}

/* #[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct JobPostRequest {
    pub job_type: String,
    pub image_root_archive_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initrd_file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_parameters_file_name: Option<String>,
    pub artifact_id: String,
    pub public_key_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_containers: Option<Vec<SshContainer>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_debug: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_env_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_dkms: Option<bool>,
}

impl From<JobPostRequest> for Job {
    fn from(job_post_request: JobPostRequest) -> Self {
        Self {
            job_type: job_post_request.job_type,
            image_root_archive_name: job_post_request.image_root_archive_name,
            kernel_file_name: job_post_request.kernel_file_name,
            initrd_file_name: job_post_request.initrd_file_name,
            kernel_parameters_file_name: job_post_request.kernel_parameters_file_name,
            artifact_id: job_post_request.artifact_id,
            public_key_id: job_post_request.public_key_id,
            ssh_containers: job_post_request.ssh_containers,
            enable_debug: job_post_request.enable_debug,
            build_env_size: job_post_request.build_env_size.map(|v| v as u8),
            require_dkms: job_post_request.require_dkms,
            ..Default::default()
        }
    }
} */

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Job {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub created: Option<String>,
  pub job_type: String,
  pub image_root_archive_name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub kernel_file_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub initrd_file_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub kernel_parameters_file_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  pub artifact_id: String,
  pub public_key_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub kubernetes_job: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub kubernetes_service: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub kubernetes_configmap: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ssh_containers: Option<Vec<SshContainer>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub enable_debug: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub resultant_image_id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub build_env_size: Option<u8>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub kubernetes_namespace: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arch: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub require_dkms: Option<bool>,
}
