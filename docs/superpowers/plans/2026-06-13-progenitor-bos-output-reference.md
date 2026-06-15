# Progenitor output reference — `src/bos/csm_api_docs.yaml`

Generated against `progenitor` 0.8 (build via `progenitor::Generator::default()`
in a `build.rs` mirroring the existing HSM/CFS/BSS ones). The smoke crate that
produced this reference lived in `/tmp/progenitor-bos-smoke` and is not
committed.

The spec fed to progenitor is the committed `src/bos/csm_api_docs.yaml`
(OpenAPI 3.0.3 — no Swagger 2.0 conversion was required, the upstream BOS spec
is already OpenAPI 3.x). Two small patches were applied to the YAML in the
same commit as this reference doc; see Section F. After the patches, `cargo
build` and `cargo clippy` succeed with zero warnings.

All generated types live in `mod types`. The client struct is `Client` at the
crate root.

## A. Generated type names (by YAML schema name)

The full inventory of generated types is in the generated file (144 structs +
41 enums = 185 types in `mod types`, plus the standard `ConversionError`
newtype in `mod types::error`). Many of those are constraint-validated newtype
wrappers around `String`/`i64` (e.g. `SessionTemplateName`, `V2SessionName`,
`V2ComponentLastUpdated`) or per-operation merged error-body types (see end of
this section).

### A.0. The 11 schemas the plan explicitly listed

| YAML schema name | Kind | Generated Rust type (`types::…`) |
|---|---|---|
| `V2SessionTemplate` | object | `types::V2SessionTemplate` |
| `V2Session` | object | `types::V2Session` |
| `V2SessionStatus` | object | `types::V2SessionStatus` |
| `V2SessionCreateResponse` | *not in spec* — closest is `V2SessionCreate` (request body) | `types::V2SessionCreate` |
| `V2SessionTemplateValidationStatus` | *not in spec* — closest is `V2SessionTemplateValidation` | `types::V2SessionTemplateValidation` (newtype `pub struct V2SessionTemplateValidation(pub String);`) |
| `V2Component` | object | `types::V2Component` |
| `V2ComponentArray` | array of `V2Component` | `types::V2ComponentArray` (newtype `pub struct V2ComponentArray(pub Vec<V2Component>);`) |
| `V2ApplyStaged` | *not in spec* — split into `V2ApplyStagedComponents` (req) and `V2ApplyStagedStatus` (resp) | `types::V2ApplyStagedComponents`, `types::V2ApplyStagedStatus` |
| `V2Options` | object | `types::V2Options` |
| `HealthCheckResponse` | *not in spec* — schema is `Healthz` | `types::Healthz` |
| `Version` | object | `types::Version` |

Note: 4 of the 11 plan-listed names use slightly different identifiers in the
actual upstream YAML — they map to the entries on the right side of the table.

### A.0.1. Other key schemas csm-rs's wrapper will touch

| YAML schema | Generated Rust type |
|---|---|
| `V2SessionArray` | `types::V2SessionArray` (newtype `Vec<V2Session>`) |
| `V2SessionTemplateArray` | `types::V2SessionTemplateArray` (newtype `Vec<V2SessionTemplate>`) |
| `V2SessionUpdate` | `types::V2SessionUpdate` |
| `V2SessionTemplatePatch` | `types::V2SessionTemplatePatch` |
| `V2ComponentArrayWithIds` | `types::V2ComponentArrayWithIds` (newtype `Vec<V2ComponentWithId>`) |
| `V2ComponentWithId` | `types::V2ComponentWithId` |
| `V2ComponentUpdate` | `types::V2ComponentUpdate` |
| `V2ComponentArrayUpdate` | `types::V2ComponentArrayUpdate` |
| `V2SessionExtendedStatus` | `types::V2SessionExtendedStatus` |
| `V2SessionStatusLabel` | `types::V2SessionStatusLabel` (enum) |
| `V2SessionOperation` | `types::V2SessionOperation` (enum) |
| `Healthz` | `types::Healthz` |
| `Version` | `types::Version` |
| `ProblemDetails` | `types::ProblemDetails` |
| `SessionTemplateName` | `types::SessionTemplateName` (constrained-string newtype) |
| `V2SessionName` | `types::V2SessionName` (constrained-string newtype) |
| `V2ComponentId` | `types::V2ComponentId` (constrained-string newtype) |
| `AgeString` | `types::AgeString` (constrained-string newtype) |
| `TenantName` | `types::TenantName` (constrained-string newtype — still emitted in `mod types`, but no longer used in any header parameter signature after the F.2 patch) |

### A.0.2. Per-operation merged error response types

Because every 4xx/5xx response is declared as
`allOf: [ProblemDetails, ProblemBadRequest|ProblemAlreadyExists|...]` rather
than a plain `$ref: '#/components/schemas/ProblemDetails'`, progenitor
synthesises a distinct merged struct per operation. Examples (one per
operation):

| Generated type | Used by |
|---|---|
| `types::GetV2HealthzResponse` | `get_v2_healthz` (4xx body) |
| `types::GetV2SessiontemplateResponse` | `get_v2_sessiontemplate` (4xx body) |
| `types::PutV2SessiontemplateResponse` | `put_v2_sessiontemplate` (4xx body) |
| `types::DeleteV2SessiontemplateResponse` | `delete_v2_sessiontemplate` (4xx body) |
| `types::PostV2SessionResponse` | `post_v2_session` (4xx body) |
| `types::DeleteV2SessionResponse` | `delete_v2_session` (4xx body) |
| …                                  | (one per operation that returns 4xx/5xx) |

Each one carries the same five `Option<…>` fields as `ProblemDetails` plus an
operation-scoped `status: Option<…ResponseStatus>` (a constrained-int newtype
that enforces the declared status codes for that operation) and `title:
Option<…ResponseTitle>` (an enum of the declared error titles). The wrapper
does not need to introspect these — it only ever needs the `detail`, `status`,
and `title` strings, which it can render via `Display` / `Debug` for logs.

## A.1. Schemas referenced by the plan but NOT generated

None of the 11 plan-listed names are missing — 4 are simply named slightly
differently in the upstream spec (see the "*not in spec*" rows in A.0). All 79
component schemas listed in `components.schemas` produced Rust types.

## B. Generated method names (by operationId)

All 29 operationIds in the spec are emitted verbatim by progenitor as
`Client::<operationId>` (snake_case, no mangling).

### B.1. The 8 v2 operations csm-rs's BOS public API uses today

| YAML operationId | Generated method | HTTP verb + path | Key args | Success response | Error variant |
|---|---|---|---|---|---|
| `get_v2_sessiontemplates` | `Client::get_v2_sessiontemplates` | GET `/v2/sessiontemplates` | `cray_tenant_name: Option<&str>` | `ResponseValue<types::V2SessionTemplateArray>` | `Error<()>` |
| `get_v2_sessiontemplate` | `Client::get_v2_sessiontemplate` | GET `/v2/sessiontemplates/{session_template_id}` | `session_template_id: &SessionTemplateName`, `cray_tenant_name: Option<&str>` | `ResponseValue<types::V2SessionTemplate>` | `Error<types::GetV2SessiontemplateResponse>` |
| `put_v2_sessiontemplate` | `Client::put_v2_sessiontemplate` | PUT `/v2/sessiontemplates/{session_template_id}` | `session_template_id: &SessionTemplateName`, `cray_tenant_name: Option<&str>`, `body: &V2SessionTemplate` | `ResponseValue<types::V2SessionTemplate>` | `Error<types::PutV2SessiontemplateResponse>` |
| `delete_v2_sessiontemplate` | `Client::delete_v2_sessiontemplate` | DELETE `/v2/sessiontemplates/{session_template_id}` | `session_template_id: &SessionTemplateName`, `cray_tenant_name: Option<&str>` | `ResponseValue<()>` (204) | `Error<types::DeleteV2SessiontemplateResponse>` |
| `get_v2_sessions` | `Client::get_v2_sessions` | GET `/v2/sessions` | `max_age: Option<&AgeString>`, `min_age: Option<&AgeString>`, `status: Option<V2SessionStatusLabel>`, `cray_tenant_name: Option<&str>` | `ResponseValue<types::V2SessionArray>` | `Error<()>` |
| `post_v2_session` | `Client::post_v2_session` | POST `/v2/sessions` | `cray_tenant_name: Option<&str>`, `body: &V2SessionCreate` | `ResponseValue<types::V2Session>` (201) | `Error<types::PostV2SessionResponse>` |
| `delete_v2_session` | `Client::delete_v2_session` | DELETE `/v2/sessions/{session_id}` | `session_id: &V2SessionName`, `cray_tenant_name: Option<&str>` | `ResponseValue<()>` (204) | `Error<types::DeleteV2SessionResponse>` |
| `get_v2_healthz` | `Client::get_v2_healthz` | GET `/v2/healthz` | (none) | `ResponseValue<types::Healthz>` | `Error<types::GetV2HealthzResponse>` |

Note on path collisions: there is one `GET /v2/sessions` (list, mapped above)
and a separate `GET /v2/sessions/{session_id}` (`get_v2_session` — single
item). Both are in B.2 the wrapper only re-exports `get_v2_sessions` because
csm-rs's current public API only needs the list endpoint.

The `cray_tenant_name` parameter is `Option<&'a str>` (a plain string slice)
in every signature thanks to the F.2 patch. Without that patch progenitor
would have used `Option<&'a types::TenantName>` and the generated file would
not compile (see Section F).

### B.2. The other 21 operations (not in csm-rs public API)

| operationId | HTTP verb + path |
|---|---|
| `root_get` | GET `/` |
| `get_v2` | GET `/v2/` |
| `validate_v2_sessiontemplate` | GET `/v2/sessiontemplatesvalid/{session_template_id}` |
| `patch_v2_sessiontemplate` | PATCH `/v2/sessiontemplates/{session_template_id}` |
| `get_v2_sessiontemplatetemplate` | GET `/v2/sessiontemplatetemplate` |
| `delete_v2_sessions` | DELETE `/v2/sessions` (bulk delete) |
| `get_v2_session` | GET `/v2/sessions/{session_id}` |
| `patch_v2_session` | PATCH `/v2/sessions/{session_id}` |
| `get_v2_session_status` | GET `/v2/sessions/{session_id}/status` |
| `save_v2_session_status` | POST `/v2/sessions/{session_id}/status` |
| `get_v2_components` | GET `/v2/components` |
| `put_v2_components` | PUT `/v2/components` |
| `patch_v2_components` | PATCH `/v2/components` |
| `get_v2_component` | GET `/v2/components/{component_id}` |
| `put_v2_component` | PUT `/v2/components/{component_id}` |
| `patch_v2_component` | PATCH `/v2/components/{component_id}` |
| `delete_v2_component` | DELETE `/v2/components/{component_id}` |
| `post_v2_apply_staged` | POST `/v2/applystaged` |
| `get_v2_options` | GET `/v2/options` |
| `patch_v2_options` | PATCH `/v2/options` |
| `get_version_v2` | GET `/v2/version` |

These all live on `Client` as plain `pub async fn <operationId>`. Task 1 will
not expose them but may surface a thin pass-through if/when needed.

### B.3. Method-name mangling rule

OperationIds in the spec are already snake_case (e.g. `get_v2_sessiontemplate`,
`post_v2_apply_staged`, `get_version_v2`). progenitor passes them through
unchanged. There is no CamelCase→snake_case conversion to track, and no
operationIds were missing — none were added by the patches in Section F.

## C. The `Error` enum

Defined in `progenitor-client 0.8.0` at
`progenitor_client/src/progenitor_client.rs:236`. It is `pub use`d through the
generated file (line 2 of generated.rs).

```rust
pub enum Error<E = ()> {
    /// The request did not conform to API requirements.
    InvalidRequest(String),

    /// A server error either due to the data, or with the connection.
    CommunicationError(reqwest::Error),

    /// An expected response when upgrading connection.
    InvalidUpgrade(reqwest::Error),

    /// A documented, expected error response.
    ErrorResponse(ResponseValue<E>),

    /// Encountered an error reading the body for an expected response.
    ResponseBodyError(reqwest::Error),

    /// An expected response code whose deserialization failed.
    InvalidResponsePayload(Bytes, serde_json::Error),

    /// A response not listed in the API description. This may represent a
    /// success or failure response; check `status().is_success()`.
    UnexpectedResponse(reqwest::Response),

    /// An error occurred in the processing of a request pre-hook.
    PreHookError(String),
}
```

Distribution of `E` parameters in the generated BOS code:

- `Error<()>` — 4 ops: `get_v2_sessiontemplates`,
  `get_v2_sessiontemplatetemplate`, `get_v2_sessions`, `get_v2_options`. Used
  when the spec declares no typed 4xx/5xx response for that operation.
- `Error<types::<OperationName>Response>` — the other 25 ops. Each operation
  has its own merged `ProblemDetails+ProblemXxx` struct because the upstream
  YAML uses `allOf` composition for error bodies rather than a plain `$ref`
  to the common `ProblemDetails`. This means the wrapper cannot share one
  error-conversion helper across operations — each one needs its own
  `From<…>` (or a single helper generic over a trait the wrapper introduces
  via blanket impl over `Display`/`Debug`).

Wrapper code unwrapping `Error` for the 8 operations in B.1 must handle
`Error<()>` (for `get_v2_sessiontemplates` and `get_v2_sessions`) and 6
distinct per-operation error types (for the remaining 6).

## D. The constructor signatures

Copied verbatim from the generated file (around lines 11431–11478):

```rust
pub struct Client {
    pub(crate) baseurl: String,
    pub(crate) client: reqwest::Client,
}

impl Client {
    /// Create a new client.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new(baseurl: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = {
            let dur = std::time::Duration::from_secs(15);
            reqwest::ClientBuilder::new().connect_timeout(dur).timeout(dur)
        };
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::ClientBuilder::new();
        Self::new_with_client(baseurl, client.build().unwrap())
    }

    /// Construct a new client with an existing `reqwest::Client`,
    /// allowing more control over its configuration.
    pub fn new_with_client(baseurl: &str, client: reqwest::Client) -> Self {
        Self {
            baseurl: baseurl.to_string(),
            client,
        }
    }

    pub fn baseurl(&self) -> &String { &self.baseurl }
    pub fn client(&self) -> &reqwest::Client { &self.client }
    pub fn api_version(&self) -> &'static str { "0.0.0-api" }
}
```

Implication for the BOS wrapper: identical to the HSM/CFS/BSS migrations —
`Client::new` bakes a 15-second connect+request timeout. The wrapper's
`gen_client()` helper MUST use `Client::new_with_client` so the project's
shared `reqwest::Client` controls the timeout, rustls config, and
insecure-TLS toggle.

`api_version()` returns `"0.0.0-api"` because the upstream `info.version` is
`"0.0.0-api"` (literal — that is the upstream value, not a placeholder).

## E. basePath behaviour

The BOS OpenAPI 3.0.3 spec declares:

```yaml
servers:
- url: https://api-gw-service-nmn.local/apis/bos
  description: The production BOS API server through a standard API gateway
- url: https://cray-bos
  description: The service as exposed through Kubernetes DNS service mapping
```

Progenitor does NOT auto-prepend `servers[0].url`. Every generated method
computes its URL as `format!("{}/<operation path>", self.baseurl)`, where the
operation path is what's literally written under `paths:` in the spec (e.g.
`/v2/healthz`, `/v2/sessiontemplates`, `/v2/sessions/{session_id}`).

This was verified by two wiremock unit tests in
`/tmp/progenitor-bos-smoke/src/lib.rs`:

1. With `Client::new_with_client(&server.uri(), …)`, a `client.get_v2_healthz()`
   call hits `/v2/healthz` — mock matches without the `/apis/bos` prefix.
2. With `Client::new_with_client(&format!("{}/apis/bos", server.uri()), …)`,
   the same call hits `/apis/bos/v2/healthz` — mock matches.

Both passed (`cargo test sanity` in the smoke crate: `test result: ok.
2 passed`).

**Operational consequence:** the BOS wrapper must build its base URL as
`format!("{}/bos", shasta_client.base_url())` because csm-rs's `base_url`
already ends in `/apis` (e.g. `https://api.cmn.alps.cscs.ch/apis`). The
operation paths in the spec already carry the `/v2/...` prefix.

## F. Spec patches applied

Two patches were applied to `src/bos/csm_api_docs.yaml` before progenitor
would produce a buildable generated.rs. Both are committed in the same commit
as this reference doc.

### F.1. Rewrite `application/problem+json` content entries to `application/json` (6 occurrences)

Progenitor's "extract response" step considers only the `application/json`
media type for the typed return path; non-json media types fall through to
`progenitor_client::ByteStream`. The BOS spec uses `application/problem+json`
(RFC 7807) for the 6 shared error responses (`AlreadyExists`, `BadRequest`,
`ResourceNotFound`, `UpdateConflict`, `ServiceUnavailable`, `InternalError`),
all in `components.responses`. With the original content-types in place, every
method that referenced one of those error responses got
`Result<…, Error<progenitor_client::ByteStream>>`, and because `ByteStream`
does not implement `Debug`, the generated file failed to compile with dozens
of `error[E0277]: ByteStream doesn't implement Debug` errors at every 4xx/5xx
match arm.

The 7th occurrence of `application/problem+json` (in the `info.description`
prose explaining the API conventions) is left untouched — it's documentation,
not a content-type key, and does not affect codegen.

### F.2. Inline the `TenantName` schema into the `Cray-Tenant-Name` header parameter as a plain `type: string` (1 parameter rewritten)

The `V2TenantHeaderParam` parameter references
`components.schemas.TenantName`, which is a constrained string (`maxLength:
127`). progenitor emits that as a newtype `pub struct TenantName(String);`
in `mod types` — and then for each of the 22 operations that include the
header, generates `header_map.append("Cray-Tenant-Name", HeaderValue::try_from(v)?)`,
where `v: &TenantName`. `HeaderValue` has no `TryFrom<&TenantName>` impl and
`TenantName` does not auto-deref into anything `HeaderValue` accepts, so the
file failed to compile with 22 `error[E0277]: HeaderValue: From<&TenantName>`
errors.

The patched parameter inlines `schema: { type: string }` (no length
constraint, otherwise progenitor still emits an operation-specific newtype
like `PostV2SessionCrayTenantName`). After the patch every method signature
uses `cray_tenant_name: Option<&'a str>`, which is what the wrapper wants
anyway. This is the same patch as CFS Section F.3 and is identical to the
hack acceptable for the smoke; the real fix is either a custom progenitor
`replace` directive or a build-time spec rewrite.

The `TenantName` newtype itself is still emitted in `mod types` (because the
schema is still defined in `components.schemas`), but nothing in the generated
code references it after the patch.

### F.3. Patches NOT required (vs HSM/CFS/BSS)

None of the following patches that earlier migrations needed were needed for
BOS:

- **No Swagger 2.0 → OpenAPI 3.0 conversion**. BOS ships as OpenAPI 3.0.3
  upstream, unlike BSS (Swagger 2.0) and the original HSM workflow.
- **No missing operationIds**. All 29 paths/verbs have explicit `operationId`
  declarations — unlike BSS, which was missing 14.
- **No `$ref` to sub-keys inside another schema's `properties`**. BOS does not
  share property definitions across schemas the way CFS does.
- **No `default:` response panics**. Every operation declares only explicit
  status codes (`200`, `201`, `204`, `400`, `404`, `409`, `500`, `503`) and no
  `default:` arms, so the `response_types.len() <= 1` assertion in
  `progenitor-impl/src/method.rs:1264` never fires.

## G. Generated artefact stats

- Total generated file: 12,904 lines (≈430 KB) — smaller than CFS (13,589
  lines) and HSM (37,574 lines), larger than BSS (2,326 lines).
- Type count in `mod types`: 185 (144 structs + 41 enums); plus the standard
  `ConversionError` newtype in `mod types::error`.
- Method count on `impl Client`: 29 `pub async fn`s — one per operationId.
- One `pub struct Client` at crate root, plus its inherent impls (`new`,
  `new_with_client`, `baseurl`, `client`, `api_version`).
- `cargo build` and `cargo clippy` both succeed with zero warnings.

## H. Runtime crate dependencies

The generated BOS code references the following extern crates:

| Crate | Already in csm-rs Cargo.toml? | Action |
|---|---|---|
| `progenitor_client` | yes (added for HSM) | none |
| `reqwest` | yes | none |
| `serde` | yes | none |
| `serde_json` | yes | none |
| `chrono` (with `serde` feature) | yes (added for HSM) | none |
| `regress` | yes (added for HSM) | none |

Crate usage counts in the generated file:

- `chrono::` — 10 references (one constrained datetime newtype:
  `V2ComponentLastUpdated(chrono::DateTime<chrono::offset::Utc>)`).
- `regress::` — 11 references (`regress::Regex::new(...)` calls in the
  `pattern`-constrained string newtypes: `AgeString`, `SessionTemplateName`,
  `V2ComponentId`, `V2SessionEndTime`, `V2SessionStartTime`, etc.).
- `uuid::` — 0 references. BOS uses plain strings for identifiers (xnames,
  session names, template names), not UUIDs.

The `progenitor-client`, `chrono` (with `serde`), `regress`, and `uuid` (with
`serde`) feature additions already committed for HSM, CFS, and BSS more than
cover BOS. **No new dependencies, and no feature-flag tweaks, are required to
land Task 1.**
