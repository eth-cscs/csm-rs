//! Wiremock smoke tests for `ShastaClient::hsm_*` methods.

mod common;
use common::{TEST_TOKEN, make_client};

use serde_json::json;
use wiremock::matchers::{bearer_token, body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------- hsm/group ----------

#[tokio::test]
async fn hsm_group_get_all_hits_smd_v2_groups() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/groups"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let groups = client.hsm_group_get_all(TEST_TOKEN).await.expect("ok");
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
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let g = client.hsm_group_get_one(TEST_TOKEN, "zinal").await.unwrap();
  assert_eq!(g.label, "zinal");
}

#[tokio::test]
async fn hsm_group_get_one_unauthorized_returns_request_error() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/groups/zinal"))
    .respond_with(ResponseTemplate::new(401).set_body_string("nope"))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let err = client
    .hsm_group_get_one(TEST_TOKEN, "zinal")
    .await
    .expect_err("err");
  assert!(
    matches!(err, csm_rs::Error::RequestError { .. }),
    "expected RequestError, got: {err:?}"
  );
}

#[tokio::test]
async fn hsm_group_delete_member_hits_nested_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("DELETE"))
    .and(path("/smd/hsm/v2/groups/zinal/members/x1000c0s0b0n0"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(204))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .hsm_group_delete_member(TEST_TOKEN, "zinal", "x1000c0s0b0n0")
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
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let arr = client
    .hsm_component_get_all(TEST_TOKEN, None)
    .await
    .unwrap();
  assert!(arr.components.unwrap_or_default().is_empty());
}

// ---------- hsm/component_status ----------

#[tokio::test]
async fn hsm_component_status_get_hits_correct_endpoint() {
  use wiremock::matchers::query_param;
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/State/Components"))
    .and(query_param("id", "x1000c0s0b0n0"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .expect(1)
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .hsm_component_status_get(TEST_TOKEN, &["x1000c0s0b0n0".to_string()])
    .await
    .expect("ok");
}

// ---------- hsm/memberships ----------

#[tokio::test]
async fn hsm_memberships_get_all_hits_v2_memberships() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/smd/hsm/v2/memberships"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .hsm_memberships_get_all(TEST_TOKEN)
    .await
    .expect("ok");
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
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let m = client
    .hsm_memberships_get_xname(TEST_TOKEN, "x1000c0s0b0n0")
    .await
    .unwrap();
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
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let result = client.hsm_redfish_get_one(TEST_TOKEN, "x1000c0s0b0").await;
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
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .hsm_hw_inventory_get_query(TEST_TOKEN, "x1000c0s0b0n0")
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
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let roles = client.hsm_roles_get(TEST_TOKEN).await.unwrap();
  assert_eq!(roles, vec!["Compute", "Service", "Storage"]);
}

// ---------- hsm/group: post_member body shape ----------

#[tokio::test]
async fn hsm_group_post_member_sends_id_body_to_members_endpoint() {
  use csm_rs::hsm::group::types::Member;
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path("/smd/hsm/v2/groups/zinal/members"))
    .and(bearer_token(TEST_TOKEN))
    .and(body_json(json!({"id": "x1000c0s0b0n0"})))
    .respond_with(
      ResponseTemplate::new(200)
        .set_body_json(json!({"code": "0", "message": "ok"})),
    )
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let ack = client
    .hsm_group_post_member(
      TEST_TOKEN,
      "zinal",
      Member {
        id: Some("x1000c0s0b0n0".to_string()),
      },
    )
    .await
    .expect("ok");
  assert_eq!(ack.message, "ok");
}
