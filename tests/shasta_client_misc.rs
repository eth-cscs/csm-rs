//! Wiremock smoke tests for `ShastaClient::bss_*` methods.

mod common;
use common::{TEST_TOKEN, make_client};

use serde_json::json;
use wiremock::matchers::{bearer_token, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------- bss/bootparameters ----------

#[tokio::test]
async fn bss_bootparameters_get_all_hits_boot_v1_bootparameters() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/bss/boot/v1/bootparameters"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let params = client
    .bss_bootparameters_get_all(TEST_TOKEN)
    .await
    .expect("ok");
  assert!(params.is_empty());
}

#[tokio::test]
async fn bss_bootparameters_get_passes_xnames_as_name_query_params() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/bss/boot/v1/bootparameters"))
    .and(query_param("name", "x1000c0s0b0n0"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let _ = client
    .bss_bootparameters_get(TEST_TOKEN, &["x1000c0s0b0n0".to_string()])
    .await
    .expect("ok");
}

#[tokio::test]
async fn bss_bootparameters_post_hits_bss_boot_v1_bootparameters() {
  use csm_rs::bss::types::BootParameters;
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path("/bss/boot/v1/bootparameters"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let bp = BootParameters {
    hosts: vec!["x1000c0s0b0n0".to_string()],
    macs: None,
    nids: None,
    params: "console=tty0".to_string(),
    kernel: "k".to_string(),
    initrd: "i".to_string(),
    cloud_init: None,
  };
  client
    .bss_bootparameters_post(TEST_TOKEN, bp)
    .await
    .expect("ok");
}

#[tokio::test]
async fn bss_bootparameters_patch_sends_patch_request() {
  use csm_rs::bss::types::BootParameters;
  let server = MockServer::start().await;
  Mock::given(method("PATCH"))
    .and(path("/bss/boot/v1/bootparameters"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let bp = BootParameters {
    hosts: vec!["x1000c0s0b0n0".to_string()],
    macs: None,
    nids: None,
    params: "console=tty0".to_string(),
    kernel: "k".to_string(),
    initrd: "i".to_string(),
    cloud_init: None,
  };
  client
    .bss_bootparameters_patch(TEST_TOKEN, &bp)
    .await
    .expect("ok");
}
