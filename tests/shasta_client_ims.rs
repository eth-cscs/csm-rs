//! Wiremock smoke tests for `ShastaClient::ims_*` methods.

mod common;
use common::{TEST_TOKEN, make_client};

use serde_json::json;
use wiremock::matchers::{bearer_token, body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------- ims/image ----------

#[tokio::test]
async fn ims_image_get_all_returns_vec_from_v3_images() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/ims/v3/images"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(
      ResponseTemplate::new(200)
        .set_body_json(json!([{"id": "abc", "name": "img-a"}])),
    )
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let images = client
    .ims_image_get_all(TEST_TOKEN)
    .await
    .expect("should succeed");
  assert_eq!(images.len(), 1);
  assert_eq!(images[0].name, "img-a");
}

#[tokio::test]
async fn ims_image_get_with_id_hits_singular_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/ims/v3/images/abc"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(
      ResponseTemplate::new(200)
        .set_body_json(json!({"id": "abc", "name": "img-a"})),
    )
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let images = client.ims_image_get(TEST_TOKEN, Some("abc")).await.unwrap();
  assert_eq!(images.len(), 1);
  assert_eq!(images[0].id.as_deref(), Some("abc"));
}

#[tokio::test]
async fn ims_image_get_404_returns_image_not_found_error() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/ims/v3/images/missing"))
    .respond_with(ResponseTemplate::new(404))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let err = client
    .ims_image_get(TEST_TOKEN, Some("missing"))
    .await
    .expect_err("should error");
  assert!(
    matches!(err, csm_rs::Error::ImageNotFound(ref id) if id == "missing"),
    "expected ImageNotFound, got: {err:?}"
  );
}

#[tokio::test]
async fn ims_image_delete_issues_soft_then_permanent_deletes() {
  let server = MockServer::start().await;
  Mock::given(method("DELETE"))
    .and(path("/ims/v3/images/abc"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(204))
    .expect(1).mount(&server)
    .await;
  Mock::given(method("DELETE"))
    .and(path("/ims/v3/deleted/images/abc"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(204))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  client
    .ims_image_delete(TEST_TOKEN, "abc")
    .await
    .expect("should succeed");
}

// ---------- ims/recipe ----------

#[tokio::test]
async fn ims_recipe_get_all_hits_v2_recipes() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/ims/v2/recipes"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let recipes = client
    .ims_recipe_get(TEST_TOKEN, None)
    .await
    .expect("should succeed");
  assert!(recipes.is_empty());
}

#[tokio::test]
async fn ims_recipe_get_by_id_hits_singular_endpoint() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/ims/v2/recipes/abc"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
      "id": "abc",
      "name": "recipe-a",
      "recipe_type": "kiwi-ng",
      "linux_distribution": "sles15",
    })))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let recipes = client
    .ims_recipe_get(TEST_TOKEN, Some("abc"))
    .await
    .expect("should succeed");
  assert_eq!(recipes.len(), 1);
}

// ---------- ims/job ----------

#[tokio::test]
async fn ims_job_get_all_hits_v3_jobs() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/ims/v3/jobs"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let result = client.ims_job_get(TEST_TOKEN, None).await;
  assert!(result.is_ok(), "got: {:?}", result.err());
}

// ---------- ims/public_keys ----------

#[tokio::test]
async fn ims_public_keys_v3_get_filters_by_username() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/ims/v3/public-keys"))
    .and(bearer_token(TEST_TOKEN))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([
      {"id": "k1", "name": "alice", "public_key": "ssh-rsa AAA..."},
      {"id": "k2", "name": "bob", "public_key": "ssh-rsa BBB..."},
    ])))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let keys = client
    .ims_public_keys_v3_get(TEST_TOKEN, Some("alice"))
    .await
    .expect("should succeed");
  assert_eq!(keys.len(), 1);
  assert_eq!(keys[0].name, "alice");
}

#[tokio::test]
async fn ims_public_keys_v3_get_single_returns_some_on_exactly_one_match() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/ims/v3/public-keys"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!([
      {"id": "k1", "name": "alice", "public_key": "ssh-rsa AAA..."}
    ])))
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let key = client
    .ims_public_keys_v3_get_single(TEST_TOKEN, "alice")
    .await
    .unwrap();
  assert!(key.is_some());
}

// ---------- ims/image: post body shape ----------

#[tokio::test]
async fn ims_image_post_sends_json_body_to_v3_images() {
  use csm_rs::ims::Image;
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path("/ims/v3/images"))
    .and(bearer_token(TEST_TOKEN))
    .and(body_json(json!({"name": "new-img"})))
    .respond_with(
      ResponseTemplate::new(201).set_body_json(json!({"id": "new-id"})),
    )
    .expect(1).mount(&server)
    .await;

  let client = make_client(&server.uri());
  let image = Image {
    id: None,
    created: None,
    name: "new-img".to_string(),
    link: None,
    arch: None,
    metadata: None,
  };
  let result = client.ims_image_post(TEST_TOKEN, &image).await.unwrap();
  assert_eq!(result["id"], "new-id");
}
