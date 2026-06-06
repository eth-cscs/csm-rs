//! Internal HTTP helpers shared by all `*::http_client` modules.
//!
//! Centralizes the three patterns that were duplicated across ~30 files:
//!   1. Building a `reqwest::Client` with the CSM root cert and optional SOCKS5 proxy.
//!   2. Issuing a bearer-authenticated request.
//!   3. Branching on response status: deserialize success body as `T`, or map
//!      a non-success status to `Error::CsmError(Value)`.
//!
//! This module is `pub(crate)` — it intentionally does not change csm-rs's
//! public API. Existing `pub async fn x(shasta_token, shasta_base_url, ...)`
//! free functions delegate here, but their signatures stay the same.

use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::Error;

/// TCP connect deadline for `reqwest::Client`s built by csm-rs. A
/// long-running Manta server inheriting reqwest's default (no
/// connect_timeout) would stall indefinitely on a hung upstream.
pub(crate) const HTTP_CONNECT_TIMEOUT: Duration =
  Duration::from_secs(45 * 60);

/// Per-request deadline (response must arrive within this). Without
/// it, a CSM endpoint that accepts the connection and then hangs
/// mid-response would block the caller indefinitely. Sized to be
/// liberal enough for slow CSM dumps (`hsm_*_get_all`, full-cluster
/// inventory queries) but short enough to surface a hung peer.
pub(crate) const HTTP_REQUEST_TIMEOUT: Duration =
  Duration::from_secs(15 * 60);

/// Build a `reqwest::Client` configured with the CSM root certificate and an
/// optional SOCKS5 proxy. This is the per-request setup that used to be
/// inlined at every call site.
pub(crate) fn build_client(
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<reqwest::Client, Error> {
  let builder = reqwest::Client::builder()
    .connect_timeout(HTTP_CONNECT_TIMEOUT)
    .timeout(HTTP_REQUEST_TIMEOUT)
    .add_root_certificate(reqwest::Certificate::from_pem(shasta_root_cert)?);

  let client = match socks5_proxy {
    Some(proxy) => builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => builder.build()?,
  };

  Ok(client)
}

/// On a 2xx response, deserialize the body as `T`. On any other status,
/// deserialize the body as `serde_json::Value` and return `Error::CsmError`
/// stamped with `method` and the response URL for log-correlation.
pub(crate) async fn handle_json_response<T: DeserializeOwned>(
  response: reqwest::Response,
  method: &str,
) -> Result<T, Error> {
  if response.status().is_success() {
    response.json::<T>().await.map_err(Error::NetError)
  } else {
    let status = response.status().as_u16();
    let url = response.url().to_string();
    let payload = response.json::<Value>().await.map_err(Error::NetError)?;
    Err(Error::csm_from_response(method, &url, status, payload))
  }
}

/// On a 2xx response, deserialize the body as `T`. On any other status,
/// read the body as text and return `Error::Message`. Used by endpoints
/// (mostly CFS v3 and BSS) whose error payloads are plain text, not JSON.
pub(crate) async fn handle_json_or_text_response<T: DeserializeOwned>(
  response: reqwest::Response,
) -> Result<T, Error> {
  if response.status().is_success() {
    response.json::<T>().await.map_err(Error::NetError)
  } else {
    let text = response.text().await.map_err(Error::NetError)?;
    Err(Error::Message(text))
  }
}

/// GET `url` with bearer auth, deserialize success body as `T`.
pub(crate) async fn get_json<T: DeserializeOwned>(
  client: &reqwest::Client,
  url: &str,
  shasta_token: &str,
) -> Result<T, Error> {
  let response = client
    .get(url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  handle_json_response(response, "GET").await
}

/// POST JSON `body` to `url` with bearer auth, deserialize success body as `T`.
pub(crate) async fn post_json<B, T>(
  client: &reqwest::Client,
  url: &str,
  shasta_token: &str,
  body: &B,
) -> Result<T, Error>
where
  B: Serialize + ?Sized,
  T: DeserializeOwned,
{
  let response = client
    .post(url)
    .json(body)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  handle_json_response(response, "POST").await
}

/// PUT JSON `body` to `url` with bearer auth, deserialize success body as `T`.
pub(crate) async fn put_json<B, T>(
  client: &reqwest::Client,
  url: &str,
  shasta_token: &str,
  body: &B,
) -> Result<T, Error>
where
  B: Serialize + ?Sized,
  T: DeserializeOwned,
{
  let response = client
    .put(url)
    .json(body)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  handle_json_response(response, "PUT").await
}

/// GET `url` with bearer auth and a query string, deserialize success body as `T`.
/// `query` is anything `serde_urlencoded` can serialize, e.g. `&[("limit", 100000)]`.
pub(crate) async fn get_json_with_query<Q, T>(
  client: &reqwest::Client,
  url: &str,
  shasta_token: &str,
  query: &Q,
) -> Result<T, Error>
where
  Q: Serialize + ?Sized,
  T: DeserializeOwned,
{
  let response = client
    .get(url)
    .query(query)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  handle_json_response(response, "GET").await
}

/// On a 2xx response, deserialize the body as `T`. On `UNAUTHORIZED`, return
/// `Error::RequestError { response, payload: text }`. On any other status,
/// deserialize the body as JSON and return `Error::CsmError`. This is the
/// pattern used by HSM endpoints that distinguish auth failures from other
/// API errors.
pub(crate) async fn handle_json_or_request_error<T: DeserializeOwned>(
  response: reqwest::Response,
  method: &str,
) -> Result<T, Error> {
  if let Err(e) = response.error_for_status_ref() {
    match response.status() {
      reqwest::StatusCode::UNAUTHORIZED => {
        let url = response.url().to_string();
        let payload = response.text().await.map_err(Error::NetError)?;
        return Err(Error::RequestError {
          response: e,
          url,
          payload,
        });
      }
      _ => {
        let status = response.status().as_u16();
        let url = response.url().to_string();
        let payload =
          response.json::<Value>().await.map_err(Error::NetError)?;
        return Err(Error::csm_from_response(
          method, &url, status, payload,
        ));
      }
    }
  }

  response.json().await.map_err(Error::NetError)
}

/// Run `f` across `items.chunks(chunk_size)` with at most
/// `max_in_flight` tasks active at once and flatten the per-batch
/// results into a single `Vec<U>`.
///
/// Each chunk is owned (`Vec<T>`) so the closure can be moved into a
/// `tokio::spawn`'d task without borrowing from the caller. The
/// closure is `Clone` so the helper can hand a fresh copy to each
/// spawned task.
///
/// Errors short-circuit: the first failing batch returns its error
/// (other in-flight batches are dropped when the `JoinSet` is dropped).
pub(crate) async fn parallel_batch<T, U, F, Fut>(
  items: &[T],
  chunk_size: usize,
  max_in_flight: usize,
  f: F,
) -> Result<Vec<U>, Error>
where
  T: Clone + Send + 'static,
  U: Send + 'static,
  F: Fn(Vec<T>) -> Fut + Clone + Send + 'static,
  Fut: std::future::Future<Output = Result<Vec<U>, Error>> + Send + 'static,
{
  use std::sync::Arc;
  use tokio::sync::Semaphore;

  let sem = Arc::new(Semaphore::new(max_in_flight));
  let mut tasks = tokio::task::JoinSet::new();

  for chunk in items.chunks(chunk_size) {
    let chunk = chunk.to_vec();
    let f = f.clone();
    let permit = sem
      .clone()
      .acquire_owned()
      .await
      .expect("semaphore should not be closed");

    tasks.spawn(async move {
      let _permit = permit;
      f(chunk).await
    });
  }

  let mut out = Vec::new();
  while let Some(message) = tasks.join_next().await {
    out.append(&mut message??);
  }
  Ok(out)
}

/// DELETE `url` with bearer auth. Returns unit on 2xx; otherwise
/// `Error::CsmError(json)`.
pub(crate) async fn delete(
  client: &reqwest::Client,
  url: &str,
  shasta_token: &str,
) -> Result<(), Error> {
  let response = client
    .delete(url)
    .bearer_auth(shasta_token)
    .send()
    .await
    .map_err(Error::NetError)?;

  if response.status().is_success() {
    Ok(())
  } else {
    let status = response.status().as_u16();
    let payload = response.json::<Value>().await.map_err(Error::NetError)?;
    Err(Error::csm_from_response("DELETE", url, status, payload))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde::Deserialize;
  use serde_json::json;
  use wiremock::matchers::{
    bearer_token, body_json, header, method, path, query_param,
  };
  use wiremock::{Mock, MockServer, ResponseTemplate};

  // ---------- build_client ----------

  // A minimal self-signed cert, used to verify that build_client accepts
  // well-formed PEM. We don't actually make any requests against it.
  const TEST_PEM: &str = "-----BEGIN CERTIFICATE-----\n\
MIIBhTCCASugAwIBAgIQIRi6zePL6mKjOipn+dNuaTAKBggqhkjOPQQDAjASMRAw\n\
DgYDVQQKEwdBY21lIENvMB4XDTE3MTAyMDE5NDMwNloXDTE4MTAyMDE5NDMwNlow\n\
EjEQMA4GA1UEChMHQWNtZSBDbzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABD0d\n\
7VNhbWvZLWPuj/RtHFjvtJBEwOkhbN/BnnE8rnZR8+sbwnc/KhCk3FhnpHZnQz7B\n\
5aETbbIgmuvewdjvSBSjYzBhMA4GA1UdDwEB/wQEAwICpDATBgNVHSUEDDAKBggr\n\
BgEFBQcDATAPBgNVHRMBAf8EBTADAQH/MCkGA1UdEQQiMCCCDmxvY2FsaG9zdDo1\n\
NDUzgg4xMjcuMC4wLjE6NTQ1MzAKBggqhkjOPQQDAgNIADBFAiEA2zpJEPQyz6/l\n\
Wf86aX6PepsntZv2GYlA5UpabfT2EZICICpJ5h/iI+i341gBmLiAFQOyTDT+/wQc\n\
6MF9+Yw1Yy0t\n\
-----END CERTIFICATE-----\n";

  #[test]
  fn build_client_with_valid_pem_succeeds() {
    let client = build_client(TEST_PEM.as_bytes(), None);
    assert!(client.is_ok());
  }

  // NOTE: there is no `build_client_with_invalid_pem_fails` test because
  // `reqwest::Certificate::from_pem` is lenient: it tolerates input without
  // PEM blocks and returns Ok with an empty cert chain. So malformed input
  // is not actually surfaced as an error by build_client.

  #[test]
  fn build_client_with_socks5_proxy_succeeds() {
    let client =
      build_client(TEST_PEM.as_bytes(), Some("socks5://localhost:9050"));
    assert!(client.is_ok());
  }

  #[test]
  fn build_client_with_invalid_proxy_url_fails() {
    let client = build_client(TEST_PEM.as_bytes(), Some(":::not a url:::"));
    assert!(client.is_err());
  }

  // ---------- request helpers (use wiremock, plain HTTP) ----------

  #[derive(Deserialize, Debug, PartialEq)]
  struct Widget {
    id: u32,
    name: String,
  }

  #[tokio::test]
  async fn get_json_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/widgets/1"))
      .and(bearer_token("tok"))
      .respond_with(
        ResponseTemplate::new(200).set_body_json(json!({"id": 1, "name": "a"})),
      )
      .mount(&server)
      .await;

    let client = reqwest::Client::new();
    let widget: Widget =
      get_json(&client, &format!("{}/widgets/1", server.uri()), "tok")
        .await
        .expect("should succeed");
    assert_eq!(
      widget,
      Widget {
        id: 1,
        name: "a".into()
      }
    );
  }

  #[tokio::test]
  async fn get_json_non_success_returns_csm_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/widgets/missing"))
      .respond_with(
        ResponseTemplate::new(404)
          .set_body_json(json!({"detail": "not found"})),
      )
      .mount(&server)
      .await;

    let client = reqwest::Client::new();
    let result: Result<Widget, _> =
      get_json(&client, &format!("{}/widgets/missing", server.uri()), "tok")
        .await;
    match result {
      Err(Error::CsmError { detail, .. }) => {
        assert_eq!(detail, "not found")
      }
      other => panic!("expected CsmError, got {:?}", other),
    }
  }

  #[tokio::test]
  async fn get_json_with_query_sends_params() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/widgets"))
      .and(query_param("limit", "100000"))
      .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
      .mount(&server)
      .await;

    let client = reqwest::Client::new();
    let result: Vec<Widget> = get_json_with_query(
      &client,
      &format!("{}/widgets", server.uri()),
      "tok",
      &[("limit", 100000)],
    )
    .await
    .expect("should succeed");
    assert!(result.is_empty());
  }

  #[tokio::test]
  async fn post_json_sends_body_and_deserializes_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
      .and(path("/widgets"))
      .and(bearer_token("tok"))
      .and(body_json(json!({"name": "new"})))
      .respond_with(
        ResponseTemplate::new(201)
          .set_body_json(json!({"id": 42, "name": "new"})),
      )
      .mount(&server)
      .await;

    let client = reqwest::Client::new();
    let widget: Widget = post_json(
      &client,
      &format!("{}/widgets", server.uri()),
      "tok",
      &json!({"name": "new"}),
    )
    .await
    .expect("should succeed");
    assert_eq!(widget.id, 42);
  }

  #[tokio::test]
  async fn put_json_works() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
      .and(path("/widgets/1"))
      .respond_with(
        ResponseTemplate::new(200)
          .set_body_json(json!({"id": 1, "name": "updated"})),
      )
      .mount(&server)
      .await;

    let client = reqwest::Client::new();
    let widget: Widget = put_json(
      &client,
      &format!("{}/widgets/1", server.uri()),
      "tok",
      &json!({"name": "updated"}),
    )
    .await
    .expect("should succeed");
    assert_eq!(widget.name, "updated");
  }

  #[tokio::test]
  async fn delete_success_returns_unit() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
      .and(path("/widgets/1"))
      .and(bearer_token("tok"))
      .respond_with(ResponseTemplate::new(204))
      .mount(&server)
      .await;

    let client = reqwest::Client::new();
    let result =
      delete(&client, &format!("{}/widgets/1", server.uri()), "tok").await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn delete_non_success_returns_csm_error() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
      .and(path("/widgets/locked"))
      .respond_with(
        ResponseTemplate::new(409).set_body_json(json!({"detail": "in use"})),
      )
      .mount(&server)
      .await;

    let client = reqwest::Client::new();
    let result =
      delete(&client, &format!("{}/widgets/locked", server.uri()), "tok").await;
    match result {
      Err(Error::CsmError { detail, .. }) => {
        assert_eq!(detail, "in use")
      }
      other => panic!("expected CsmError, got {:?}", other),
    }
  }

  // ---------- response handlers ----------

  #[tokio::test]
  async fn handle_json_or_text_response_text_fallback() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/cfs"))
      .respond_with(ResponseTemplate::new(500).set_body_string("server boom"))
      .mount(&server)
      .await;

    let client = reqwest::Client::new();
    let response = client
      .get(format!("{}/cfs", server.uri()))
      .send()
      .await
      .unwrap();
    let result: Result<Widget, _> =
      handle_json_or_text_response(response).await;
    match result {
      Err(Error::Message(m)) => assert_eq!(m, "server boom"),
      other => panic!("expected Message('server boom'), got {:?}", other),
    }
  }

  #[tokio::test]
  async fn handle_json_or_request_error_unauthorized() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/hsm"))
      .respond_with(
        ResponseTemplate::new(401).set_body_string("auth header missing"),
      )
      .mount(&server)
      .await;

    let client = reqwest::Client::new();
    let response = client
      .get(format!("{}/hsm", server.uri()))
      .send()
      .await
      .unwrap();
    let result: Result<Widget, _> =
      handle_json_or_request_error(response, "GET").await;
    match result {
      Err(Error::RequestError { payload, .. }) => {
        assert_eq!(payload, "auth header missing")
      }
      other => panic!("expected RequestError, got {:?}", other),
    }
  }

  #[tokio::test]
  async fn handle_json_or_request_error_other_status_returns_csm_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/hsm/x"))
      .respond_with(
        ResponseTemplate::new(500)
          .set_body_json(json!({"detail": "db unavailable"})),
      )
      .mount(&server)
      .await;

    let client = reqwest::Client::new();
    let response = client
      .get(format!("{}/hsm/x", server.uri()))
      .send()
      .await
      .unwrap();
    let result: Result<Widget, _> =
      handle_json_or_request_error(response, "GET").await;
    match result {
      Err(Error::CsmError { detail, .. }) => {
        assert_eq!(detail, "db unavailable")
      }
      other => panic!("expected CsmError, got {:?}", other),
    }
  }

  // ---------- parallel_batch ----------

  #[tokio::test]
  async fn parallel_batch_flattens_results() {
    let items: Vec<i32> = (0..10).collect();
    let out = parallel_batch(&items, 3, 4, |chunk: Vec<i32>| async move {
      Ok::<_, Error>(chunk.into_iter().map(|x| x * 2).collect::<Vec<_>>())
    })
    .await
    .expect("should succeed");
    let mut sorted = out;
    sorted.sort();
    assert_eq!(sorted, (0..10).map(|x| x * 2).collect::<Vec<_>>());
  }

  #[tokio::test]
  async fn parallel_batch_propagates_error() {
    let items: Vec<i32> = (0..5).collect();
    let result: Result<Vec<i32>, _> =
      parallel_batch(&items, 2, 2, |_chunk: Vec<i32>| async move {
        Err(Error::Message("boom".into()))
      })
      .await;
    match result {
      Err(Error::Message(m)) => assert_eq!(m, "boom"),
      other => panic!("expected Message('boom'), got {:?}", other),
    }
  }

  #[tokio::test]
  async fn parallel_batch_empty_input_returns_empty() {
    let items: Vec<i32> = vec![];
    let out: Vec<i32> =
      parallel_batch(&items, 3, 4, |_chunk: Vec<i32>| async move {
        unreachable!("closure should not be called on empty input")
      })
      .await
      .expect("should succeed");
    assert!(out.is_empty());
  }

  // Bearer auth verification — make sure every helper actually sends the token
  #[tokio::test]
  async fn bearer_token_is_sent_with_get_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
      .and(path("/auth"))
      .and(header("authorization", "Bearer test-token"))
      .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": 1})))
      .mount(&server)
      .await;

    let client = reqwest::Client::new();
    let _: serde_json::Value =
      get_json(&client, &format!("{}/auth", server.uri()), "test-token")
        .await
        .expect("should succeed");
  }
}
