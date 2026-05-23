pub mod node_power_off {

  use serde_json::Value;

  use crate::{
    capmc::{self, types::PowerStatus, utils::wait_nodes_to_power_off},
    common::http,
    error::Error,
  };

  pub async fn post(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    xname_vec: Vec<String>,
    reason_opt: Option<String>,
    force: bool,
  ) -> Result<Value, Error> {
    log::info!("Power OFF nodes: {:?}", xname_vec);

    let power_off = PowerStatus::new(reason_opt, xname_vec, force, None);

    let client = http::build_client(shasta_root_cert, socks5_proxy)?;
    let api_url = format!("{}/capmc/capmc/v1/xname_off", shasta_base_url);

    Ok(
      client
        .post(api_url)
        .bearer_auth(shasta_token)
        .json(&power_off)
        .send()
        .await?
        .json::<Value>()
        .await?,
    )
  }

  /// Shut down a node
  /// This is  sync call meaning it won't return untill the target is down
  pub async fn post_sync(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    xname_vec: Vec<String>,
    reason_opt: Option<String>,
    force: bool,
  ) -> Result<Value, Error> {
    // Check Nodes are shutdown
    let _ = capmc::http_client::node_power_status::post(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      &xname_vec,
    )
    .await?;

    wait_nodes_to_power_off(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      xname_vec,
      reason_opt,
      force,
    )
    .await
  }
}

pub mod node_power_on {

  use serde_json::Value;

  use crate::{
    capmc::{self, types::PowerStatus, utils::wait_nodes_to_power_on},
    common::http,
    error::Error,
  };

  pub async fn post(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    xname_vec: Vec<String>,
    reason: Option<String>,
  ) -> Result<Value, Error> {
    // log::info!("Power ON nodes: {:?}", xname_vec);

    let power_on = PowerStatus::new(reason, xname_vec, false, None);

    let client = http::build_client(shasta_root_cert, socks5_proxy)?;
    let api_url = format!("{}/capmc/capmc/v1/xname_on", shasta_base_url);

    Ok(
      client
        .post(api_url)
        .bearer_auth(shasta_token)
        .json(&power_on)
        .send()
        .await?
        .json::<Value>()
        .await?,
    )
  }

  /// Power ON a group of nodes
  /// This is  sync call meaning it won't return untill all nodes are ON
  pub async fn post_sync(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    xname_vec: Vec<String>,
    reason: Option<String>,
  ) -> Result<Value, Error> {
    // Check Nodes are shutdown
    let _ = capmc::http_client::node_power_status::post(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      &xname_vec,
    )
    .await?;

    wait_nodes_to_power_on(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      xname_vec,
      reason,
    )
    .await
  }
}

pub mod node_power_reset {

  use serde_json::Value;

  use crate::{
    capmc::{self, types::PowerStatus},
    common::http,
    error::Error,
  };

  pub async fn post(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    xname_vec: Vec<String>,
    reason: Option<String>,
    force: bool,
  ) -> Result<Value, Error> {
    let node_restart = PowerStatus::new(reason, xname_vec, force, None);

    let client = http::build_client(shasta_root_cert, socks5_proxy)?;
    let api_url = format!("{}/capmc/capmc/v1/xname_reinit", shasta_base_url);

    Ok(
      client
        .post(api_url)
        .bearer_auth(shasta_token)
        .json(&node_restart)
        .send()
        .await?
        .json::<Value>()
        .await?,
    )
  }

  pub async fn post_sync(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    xname_vec: Vec<String>,
    reason_opt: Option<String>,
    force: bool,
  ) -> Result<Value, Error> {
    log::info!("Power RESET node: {:?}", xname_vec);

    let _ = capmc::http_client::node_power_off::post_sync(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      xname_vec.clone(),
      reason_opt.clone(),
      force,
    )
    .await?;

    capmc::http_client::node_power_on::post_sync(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      socks5_proxy,
      xname_vec,
      reason_opt,
    )
    .await
  }

  /// Power RESET a group of nodes
  /// This is  sync call meaning it won't return untill all nodes are ON
  pub async fn post_sync_vec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    xnames: Vec<String>,
    reason_opt: Option<String>,
    force: bool,
  ) -> Result<Value, Error> {
    let mut nodes_reseted = Vec::new();

    let mut tasks = tokio::task::JoinSet::new();

    for xname in xnames {
      let shasta_token_string = shasta_token.to_string();
      let shasta_base_url_string = shasta_base_url.to_string();
      let shasta_root_cert_vec = shasta_root_cert.to_vec();
      let socks5_proxy_opt = socks5_proxy.map(str::to_owned);
      let reason_cloned = reason_opt.clone();

      tasks.spawn(async move {
        post_sync(
          &shasta_token_string,
          &shasta_base_url_string,
          &shasta_root_cert_vec,
          socks5_proxy_opt.as_deref(),
          vec![xname],
          reason_cloned,
          force,
        )
        .await
      });
    }

    while let Some(message) = tasks.join_next().await {
      nodes_reseted.push(message??);
    }

    Ok(serde_json::to_value(nodes_reseted)?)
  }
}

pub mod node_power_status {

  use serde_json::Value;

  use crate::{
    capmc::types::NodeStatus,
    common::http,
    error::Error,
  };

  pub async fn post(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    xnames: &Vec<String>,
  ) -> core::result::Result<Value, Error> {
    log::info!("Checking nodes status: {:?}", xnames);

    let node_status_payload =
      NodeStatus::new(None, Some(xnames.clone()), Some("redfish".to_string()));

    let client = http::build_client(shasta_root_cert, socks5_proxy)?;
    let url_api =
      format!("{}/capmc/capmc/v1/get_xname_status", shasta_base_url);

    Ok(
      client
        .post(url_api)
        .bearer_auth(shasta_token)
        .json(&node_status_payload)
        .send()
        .await?
        .json::<Value>()
        .await?,
    )
  }
}
