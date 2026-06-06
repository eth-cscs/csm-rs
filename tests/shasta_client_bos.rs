//! Wiremock smoke tests for `ShastaClient::bos_*` methods.

mod common;
use common::{TEST_TOKEN, make_client};

use serde_json::json;
use wiremock::matchers::{bearer_token, body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------- bos/session/v2 ----------

#[tokio::test]
async fn bos_session_v2_get_all_hits_v2_sessions() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/bos/v2/sessions"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let sessions = client
    .bos_session_v2_get(TEST_TOKEN, None)
    .await
    .expect("ok");
  assert!(sessions.is_empty());
}

#[tokio::test]
async fn bos_session_v2_get_by_id_hits_singular_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/bos/v2/sessions/sess-1"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "name": "sess-1",
      "template_name": "tmpl-1",
    })))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let sessions = client
    .bos_session_v2_get(TEST_TOKEN, Some("sess-1"))
    .await
    .unwrap();
  assert_eq!(sessions.len(), 1);
  assert_eq!(sessions[0].name.as_deref(), Some("sess-1"));
}

#[tokio::test]
async fn bos_session_v2_delete_hits_singular_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("DELETE"))
    .and(path("/bos/v2/sessions/sess-1"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(204))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .bos_session_v2_delete(TEST_TOKEN, "sess-1")
    .await
    .expect("ok");
}

// ---------- bos/template/v2 ----------

#[tokio::test]
async fn bos_template_v2_get_all_hits_v2_sessiontemplates() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/bos/v2/sessiontemplates"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .bos_template_v2_get_all(TEST_TOKEN)
    .await
    .expect("ok");
}

#[tokio::test]
async fn bos_template_v2_get_by_name_hits_singular_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/bos/v2/sessiontemplates/tmpl-1"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(
      ResponseTemplate::new(200).set_body_json(json!({"name": "tmpl-1"})),
    )
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let templates = client
    .bos_template_v2_get(TEST_TOKEN, Some("tmpl-1"))
    .await
    .unwrap();
  assert_eq!(templates.len(), 1);
  assert_eq!(templates[0].name.as_deref(), Some("tmpl-1"));
}

#[tokio::test]
async fn bos_template_v2_delete_propagates_non_2xx_errors() {
  let server = MockServer::start().await;
  Mock::given(method("DELETE"))
    .and(path("/bos/v2/sessiontemplates/tmpl-1"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(500))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let err = client
    .bos_template_v2_delete(TEST_TOKEN, "tmpl-1")
    .await
    .expect_err("500 should propagate");
  assert!(matches!(err, csm_rs::error::Error::NetError(_)));
}

#[tokio::test]
async fn bos_template_v2_delete_succeeds_on_204() {
  let server = MockServer::start().await;
  Mock::given(method("DELETE"))
    .and(path("/bos/v2/sessiontemplates/tmpl-1"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(204))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .bos_template_v2_delete(TEST_TOKEN, "tmpl-1")
    .await
    .expect("ok");
}

// ---------- bos/template/v2: put body shape ----------

#[tokio::test]
async fn bos_template_v2_put_sends_json_body_to_singular_endpoint() {
  use csm_rs::bos::BosSessionTemplate;
  let server = MockServer::start().await;
  Mock::given(method("PUT"))
    .and(path("/bos/v2/sessiontemplates/tmpl-1"))
    .and(bearer_token(TEST_TOKEN))
    .and(body_json(json!({"name": "tmpl-1"})))
    .respond_with(
      ResponseTemplate::new(200).set_body_json(json!({"name": "tmpl-1"})),
    )
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let template = BosSessionTemplate {
    name: Some("tmpl-1".to_string()),
    tenant: None,
    description: None,
    enable_cfs: None,
    cfs: None,
    boot_sets: None,
    links: None,
  };
  let created = client
    .bos_template_v2_put(TEST_TOKEN, &template, "tmpl-1")
    .await
    .expect("ok");
  assert_eq!(created.name.as_deref(), Some("tmpl-1"));
}

// ---------- bos/health_check ----------

#[tokio::test]
async fn bos_health_check_hits_v2_healthz() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/bos/v2/healthz"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(
      ResponseTemplate::new(200).set_body_json(json!({"status": "ok"})),
    )
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let result = client.bos_health_check(TEST_TOKEN).await.unwrap();
  assert_eq!(result["status"], "ok");
}
