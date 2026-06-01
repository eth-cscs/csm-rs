//! Wiremock smoke tests for `ShastaClient::cfs_*` methods.

mod common;
use common::{TEST_TOKEN, make_client};

use serde_json::json;
use wiremock::matchers::{bearer_token, body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------- cfs/component v2 ----------

#[tokio::test]
async fn cfs_component_v2_get_all_hits_v2_components() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/cfs/v2/components"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let components = client
    .cfs_component_v2_get_all(TEST_TOKEN)
    .await
    .expect("ok");
  assert!(components.is_empty());
}

#[tokio::test]
async fn cfs_component_v2_get_single_component_hits_singular_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/cfs/v2/components/xname-1"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(
      ResponseTemplate::new(200).set_body_json(json!({"id": "xname-1"})),
    )
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let c = client
    .cfs_component_v2_get_single_component(TEST_TOKEN, "xname-1")
    .await
    .unwrap();
  assert_eq!(c.id.as_deref(), Some("xname-1"));
}

// ---------- cfs/component v3 ----------

#[tokio::test]
async fn cfs_component_v3_get_options_hits_v3_options() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/cfs/v3/options"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "default_batcher_retry_policy": 3
    })))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let opts = client
    .cfs_component_v3_get_options(TEST_TOKEN)
    .await
    .unwrap();
  assert_eq!(opts["default_batcher_retry_policy"], 3);
}

#[tokio::test]
async fn cfs_component_v3_get_returns_components_from_wrapped_payload() {
  // v3 wraps the array under a top-level "components" key (vs v2's bare array).
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/cfs/v3/components"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(
      ResponseTemplate::new(200)
        .set_body_json(json!({"components": [{"id": "xname-1"}]})),
    )
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let components = client
    .cfs_component_v3_get(TEST_TOKEN, None, None)
    .await
    .unwrap();
  assert_eq!(components.len(), 1);
  assert_eq!(components[0].id.as_deref(), Some("xname-1"));
}

// ---------- cfs/configuration v2 ----------

#[tokio::test]
async fn cfs_configuration_v2_get_all_hits_v2_configurations() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/cfs/v2/configurations"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .cfs_configuration_v2_get_all(TEST_TOKEN)
    .await
    .expect("ok");
}

#[tokio::test]
async fn cfs_configuration_v2_get_by_name_hits_singular_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/cfs/v2/configurations/zinal-config"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "name": "zinal-config",
      "lastUpdated": "2024-01-01T00:00:00Z",
      "layers": []
    })))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let configs = client
    .cfs_configuration_v2_get(TEST_TOKEN, Some("zinal-config"))
    .await
    .unwrap();
  assert_eq!(configs.len(), 1);
}

#[tokio::test]
async fn cfs_configuration_v2_delete_hits_singular_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("DELETE"))
    .and(path("/cfs/v2/configurations/zinal-config"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(204))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .cfs_configuration_v2_delete(TEST_TOKEN, "zinal-config")
    .await
    .expect("ok");
}

// ---------- cfs/session v2 ----------

#[tokio::test]
async fn cfs_session_v2_get_all_hits_v2_sessions() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/cfs/v2/sessions"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  client.cfs_session_v2_get_all(TEST_TOKEN).await.expect("ok");
}

#[tokio::test]
async fn cfs_session_v2_delete_hits_singular_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("DELETE"))
    .and(path("/cfs/v2/sessions/sess-1"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(204))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .cfs_session_v2_delete(TEST_TOKEN, "sess-1")
    .await
    .expect("ok");
}

// ---------- cfs/session v3 ----------

#[tokio::test]
async fn cfs_session_v3_get_returns_sessions_from_wrapped_payload() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/cfs/v3/sessions"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "sessions": [{"name": "sess-1", "debug_on_failure": false}],
      "next": null,
    })))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let sessions = client
    .cfs_session_v3_get(
      TEST_TOKEN, None, None, None, None, None, None, None, None, None,
    )
    .await
    .unwrap();
  assert_eq!(sessions.len(), 1);
  assert_eq!(sessions[0].name, "sess-1");
}

// ---------- cfs/common (health_check) ----------

#[tokio::test]
async fn cfs_health_check_hits_healthz() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/cfs/healthz"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(
      ResponseTemplate::new(200).set_body_json(json!({"status": "ok"})),
    )
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let result = client.cfs_health_check(TEST_TOKEN).await.unwrap();
  assert_eq!(result["status"], "ok");
}

// ---------- cfs/configuration v2: put body shape ----------

#[tokio::test]
async fn cfs_configuration_v2_put_sends_layers_body_to_singular_endpoint() {
  use csm_rs::cfs::v2::CfsConfigurationRequest;
  let server = MockServer::start().await;
  Mock::given(method("PUT"))
    .and(path("/cfs/v2/configurations/cfg-1"))
    .and(bearer_token(TEST_TOKEN))
    // CFS v2 PUT body is `{ "layers": ... }` only -- the name is the
    // URL parameter, not a body field.
    .and(body_json(json!({"layers": []})))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "name": "cfg-1",
      "lastUpdated": "2025-01-01T00:00:00Z",
      "layers": [],
    })))
    .mount(&server)
    .await;

  let client = make_client(&server.uri());
  let request = CfsConfigurationRequest::new();
  let response = client
    .cfs_configuration_v2_put(TEST_TOKEN, &request, "cfg-1")
    .await
    .expect("ok");
  assert_eq!(response.name, "cfg-1");
}
