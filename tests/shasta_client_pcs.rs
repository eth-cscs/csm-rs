//! Wiremock smoke tests for `ShastaClient::pcs_*` methods.
//!
//! These tests assert URL formation, HTTP method, and bearer auth — they do
//! not exhaustively re-verify JSON parsing (that's covered by the unit tests
//! in `src/common/http.rs`).

mod common;
use common::{TEST_TOKEN, make_client};

use serde_json::json;
use wiremock::matchers::{bearer_token, body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------- pcs/transitions ----------

#[tokio::test]
async fn pcs_transitions_post_hits_v1_transitions_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path("/power-control/v1/transitions"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "transitionID": "xform-1",
      "operation": "On",
      "createTime": "2024-01-01T00:00:00Z",
    })))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let result = client
    .pcs_transitions_post(TEST_TOKEN, "on", &["x1000c0s0b0n0".to_string()])
    .await;
  assert!(result.is_ok(), "got: {:?}", result.err());
}

#[tokio::test]
async fn pcs_transitions_post_rejects_invalid_operation_before_dispatch() {
  // The operation string is validated client-side; no HTTP request is made.
  let server = MockServer::start().await;
  let client = make_client(&server.uri());
  let result = client
    .pcs_transitions_post(
      TEST_TOKEN,
      "invalid-op",
      &["x1000c0s0b0n0".to_string()],
    )
    .await;
  assert!(result.is_err());
}

#[tokio::test]
async fn pcs_transitions_get_by_id_hits_correct_url() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/power-control/v1/transitions/xform-1"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "transitionID": "xform-1",
      "operation": "On",
      "createTime": "2024-01-01T00:00:00Z",
      "automaticExpirationTime": "2024-01-01T01:00:00Z",
      "transitionStatus": "completed",
      "taskCounts": {
        "total": 1, "new": 0, "in-progress": 0, "failed": 0,
        "succeeded": 1, "un-supported": 0,
      },
      "tasks": [],
    })))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let result = client
    .pcs_transitions_get_by_id(TEST_TOKEN, "xform-1")
    .await;
  assert!(result.is_ok(), "got: {:?}", result.err());
}

// ---------- pcs/power_status ----------

#[tokio::test]
async fn pcs_power_status_post_hits_correct_endpoint_with_filters() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path("/power-control/v1/power-status"))
    .and(bearer_token(TEST_TOKEN))
    .and(body_json(json!({
      "xname": ["x1000c0s0b0n0"],
      "powerStateFilter": "on",
      "managementStateFilter": "available",
    })))
    .respond_with(
      ResponseTemplate::new(200).set_body_json(json!({"status": []})),
    )
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let result = client
    .pcs_power_status_post(
      TEST_TOKEN,
      Some(&["x1000c0s0b0n0"]),
      Some("on"),
      Some("available"),
    )
    .await;
  assert!(result.is_ok(), "got: {:?}", result.err());
}

// ---------- pcs/power_cap ----------

#[tokio::test]
async fn pcs_power_cap_get_hits_correct_url() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/power-control/v1/power-cap"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "taskId": "task-1",
      "type": "snapshot",
      "taskCreateTime": "2024-01-01T00:00:00Z",
      "taskStatus": "completed",
      "taskCounts": {
        "total": 1, "new": 0, "in-progress": 0,
        "failed": 0, "succeeded": 1, "un-supported": 0,
      },
    })))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let result = client.pcs_power_cap_get(TEST_TOKEN).await;
  assert!(result.is_ok(), "got: {:?}", result.err());
}
