use futures_channel::mpsc::Sender;
use kube::api::{AttachedProcess, TerminalSize};
use manta_backend_dispatcher::{
  error::Error,
  interfaces::console::ConsoleTrait,
  types::{K8sAuth, K8sDetails},
};
use tokio::io::{AsyncRead, AsyncWrite};

use super::Csm;
use crate::{
  common::vault::http_client::fetch_shasta_k8s_secrets_from_vault,
  node::console,
};

impl ConsoleTrait for Csm {
  type T = Box<dyn AsyncWrite + Unpin>;
  type U = Box<dyn AsyncRead + Unpin>;

  async fn attach_to_node_console(
    &self,
    shasta_token: &str,
    site_name: &str,
    xname: &str,
    term_width: u16,
    term_height: u16,
    k8s: &K8sDetails,
  ) -> Result<(Box<dyn AsyncWrite + Unpin>, Box<dyn AsyncRead + Unpin>), Error>
  {
    let shasta_k8s_secrets = match &k8s.authentication {
      K8sAuth::Native {
        certificate_authority_data,
        client_certificate_data,
        client_key_data,
      } => {
        serde_json::json!({ "certificate-authority-data": certificate_authority_data, "client-certificate-data": client_certificate_data, "client-key-data": client_key_data })
      }
      K8sAuth::Vault {
        base_url,
        // secret_path: _secret_path,
      } => {
        fetch_shasta_k8s_secrets_from_vault(&base_url, shasta_token, &site_name)
          .await
          .map_err(|e| Error::Message(e.to_string()))?
      }
    };

    let mut attached: AttachedProcess =
      console::get_container_attachment_to_conman(
        &xname.to_string(),
        &k8s.api_url,
        shasta_k8s_secrets,
      )
      .await
      .map_err(|e| Error::Message(e.to_string()))?;

    let mut terminal_size_writer: Sender<TerminalSize> =
      attached.terminal_size().unwrap();
    terminal_size_writer
      .try_send(TerminalSize {
        width: term_width,
        height: term_height,
      })
      .map_err(|e| Error::Message(e.to_string()))?;

    println!("Connected to {}!", xname,);
    println!("Use &. key combination to exit the console.",);

    Ok((
      Box::new(attached.stdin().unwrap()),
      Box::new(attached.stdout().unwrap()),
    ))
  }

  async fn attach_to_session_console(
    &self,
    shasta_token: &str,
    site_name: &str,
    session_name: &str,
    term_width: u16,
    term_height: u16,
    k8s: &K8sDetails,
  ) -> Result<(Box<dyn AsyncWrite + Unpin>, Box<dyn AsyncRead + Unpin>), Error>
  {
    let shasta_k8s_secrets = match &k8s.authentication {
      K8sAuth::Native {
        certificate_authority_data,
        client_certificate_data,
        client_key_data,
      } => {
        serde_json::json!({ "certificate-authority-data": certificate_authority_data, "client-certificate-data": client_certificate_data, "client-key-data": client_key_data })
      }
      K8sAuth::Vault {
        base_url,
        // secret_path: _secret_path,
      } => {
        fetch_shasta_k8s_secrets_from_vault(&base_url, shasta_token, &site_name)
          .await
          .map_err(|e| Error::Message(e.to_string()))?
      }
    };

    let mut attached: AttachedProcess =
      console::get_container_attachment_to_cfs_session_image_target(
        &session_name.to_string(),
        &k8s.api_url,
        shasta_k8s_secrets,
      )
      .await
      .map_err(|e| Error::Message(e.to_string()))?;

    let mut terminal_size_writer: Sender<TerminalSize> =
      attached.terminal_size().unwrap();
    terminal_size_writer
      .try_send(TerminalSize {
        width: term_width,
        height: term_height,
      })
      .map_err(|e| Error::Message(e.to_string()))?;

    println!(
      "Connected to session target container for session name: {}!",
      session_name
    );
    println!("Use &. key combination to exit the console.",);

    Ok((
      Box::new(attached.stdin().unwrap()),
      Box::new(attached.stdout().unwrap()),
    ))
  }
}
