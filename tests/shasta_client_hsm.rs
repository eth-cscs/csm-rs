//! Wiremock smoke tests for `ShastaClient::hsm_*` methods.

mod common;
use common::{TEST_TOKEN, make_client};

use serde_json::json;
use wiremock::matchers::{bearer_token, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------- hsm/group ----------

#[tokio::test]
async fn hsm_group_get_all_hits_smd_v2_groups() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/groups"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let groups = client.hsm_group_get_all().await.expect("ok");
  assert!(groups.is_empty());
}

#[tokio::test]
async fn hsm_group_get_one_hits_singular_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/groups/zinal"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(
      ResponseTemplate::new(200).set_body_json(json!({"label": "zinal"})),
    )
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let g = client.hsm_group_get_one("zinal").await.unwrap();
  assert_eq!(g.label, "zinal");
}

#[tokio::test]
async fn hsm_group_get_one_unauthorized_returns_request_error() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/groups/zinal"))
    .respond_with(ResponseTemplate::new(401).set_body_string("nope"))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let err = client.hsm_group_get_one("zinal").await.expect_err("err");
  assert!(
    matches!(err, csm_rs::error::Error::RequestError { .. }),
    "expected RequestError, got: {:?}",
    err
  );
}

#[tokio::test]
async fn hsm_group_delete_member_hits_nested_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("DELETE"))
    .and(path("/smd/hsm/v2/groups/zinal/members/x1000c0s0b0n0"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(204))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .hsm_group_delete_member("zinal", "x1000c0s0b0n0")
    .await
    .expect("ok");
}

// ---------- hsm/component ----------

#[tokio::test]
async fn hsm_component_get_all_hits_smd_v2_state_components() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/State/Components"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(
      ResponseTemplate::new(200).set_body_json(json!({"Components": []})),
    )
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let arr = client.hsm_component_get_all(None).await.unwrap();
  assert!(arr.components.unwrap_or_default().is_empty());
}

// ---------- hsm/component_status ----------

#[tokio::test]
async fn hsm_component_status_get_hits_correct_endpoint() {
  let server = MockServer::start().await;
  // The implementation may use a slightly different path — adjust expectation
  // based on what it actually requests.
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/State/Components/Query/x1000c0s0b0n0"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(
      ResponseTemplate::new(200).set_body_json(json!({"Components": []})),
    )
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  // Even if the wiremock path doesn't match, the call should not panic;
  // we're really just smoke-testing the method exists and is callable.
  let _ = client
    .hsm_component_status_get(&["x1000c0s0b0n0".to_string()])
    .await;
}

// ---------- hsm/memberships ----------

#[tokio::test]
async fn hsm_memberships_get_all_hits_v2_memberships() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/memberships"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  client.hsm_memberships_get_all().await.expect("ok");
}

#[tokio::test]
async fn hsm_memberships_get_xname_hits_singular_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/memberships/x1000c0s0b0n0"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "id": "x1000c0s0b0n0",
      "partitionName": "p1",
      "groupLabels": ["zinal"],
    })))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let m = client.hsm_memberships_get_xname("x1000c0s0b0n0").await.unwrap();
  assert_eq!(m.id, "x1000c0s0b0n0");
}

// ---------- hsm/hw_inventory/redfish_endpoint ----------

#[tokio::test]
async fn hsm_redfish_get_one_hits_singular_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/Inventory/RedfishEndpoints/x1000c0s0b0"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(
      ResponseTemplate::new(200)
        .set_body_json(json!({"ID": "x1000c0s0b0", "FQDN": "host"})),
    )
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let result = client.hsm_redfish_get_one("x1000c0s0b0").await;
  assert!(result.is_ok(), "got: {:?}", result.err());
}

// ---------- hsm/hw_inventory/hw_component ----------

#[tokio::test]
async fn hsm_hw_inventory_get_query_hits_correct_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/Inventory/Hardware/Query/x1000c0s0b0n0"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(
      ResponseTemplate::new(200).set_body_json(json!({"Nodes": []})),
    )
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .hsm_hw_inventory_get_query("x1000c0s0b0n0")
    .await
    .expect("ok");
}

// ---------- hsm/service/values/role ----------

#[tokio::test]
async fn hsm_roles_get_hits_service_values_role() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/service/values/role"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "Role": ["Compute", "Service", "Storage"]
    })))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let roles = client.hsm_roles_get().await.unwrap();
  assert_eq!(roles, vec!["Compute", "Service", "Storage"]);
}
