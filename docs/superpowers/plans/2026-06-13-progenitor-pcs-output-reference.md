# Progenitor output reference — `src/pcs/csm_api_docs.yaml`

Generated against `progenitor` 0.8 (build via `progenitor::Generator::default()`
in a `build.rs` mirroring the existing HSM/CFS/BSS/BOS ones). The smoke crate
that produced this reference lived in `/tmp/progenitor-pcs-smoke` and is not
committed.

The spec fed to progenitor is the committed `src/pcs/csm_api_docs.yaml`
(OpenAPI 3.0.0 — no Swagger 2.0 conversion was required, the upstream PCS
spec is already OpenAPI 3.x). Three patches were applied to the YAML in the
same commit as this reference doc; see Section F. After the patches,
`cargo build` and `cargo clippy` succeed with zero warnings.

All generated types live in `mod types`. The client struct is `Client` at the
crate root.

## A. Generated type names (by YAML schema name)

The full inventory in `mod types` is 30 structs + 9 enums = 39 types, plus
the standard `ConversionError` newtype in `mod types::error`. (The 30
includes `ConversionError` and the `NonEmptyStringList` orphan newtype that
is still emitted post-F.3 but unreferenced — see Section F.3.)

### A.0. The 12 schemas the plan explicitly listed

| Plan name | Kind | Generated Rust type (`types::…`) |
|---|---|---|
| `Transition` | *not in spec* — closest is `transition_task_data` | `types::TransitionTaskData` |
| `TransitionRequest` | *not in spec* — closest is `transition_create` | `types::TransitionCreate` |
| `TransitionStatus` | enum | `types::TransitionStatus` |
| `PowerStatus` | object | `types::PowerStatus` |
| `PowerStatusResponse` | *not in spec* — closest is `power_status_all` | `types::PowerStatusAll` |
| `PowerCap` | *not in spec* — closest is `power_cap_patch` | `types::PowerCapPatch` |
| `PowerCapTask` | *not in spec* — closest is `power_cap_task_info` | `types::PowerCapTaskInfo` |
| `PowerCapTaskSnapshot` | *not in spec* — closest is `power_cap_snapshot_req` | `types::PowerCapSnapshotReq` |
| `HealthCheck` | *not in spec* — closest is `health_rsp` | `types::HealthRsp` |
| `Liveness` | *no schema in spec* — `GET /liveness` returns 204 No Content (no body) | n/a (`get_liveness` returns `ResponseValue<()>`) |
| `Readiness` | *no schema in spec* — `GET /readiness` returns 204 No Content (no body) | n/a (`get_readiness` returns `ResponseValue<()>`) |
| `Error` | *not in spec* — every error response uses `Problem7807` (RFC 7807) | `types::Problem7807` |

Note: 9 of the 12 plan-listed names use slightly different identifiers in the
upstream YAML — they map to the entries on the right side of the table.
`Liveness` and `Readiness` have no schema at all (those endpoints return empty
bodies).

### A.0.1. The other 23 schemas (verbatim mapping)

PCS schema names are `snake_case` upstream; progenitor converts each to
PascalCase.

| YAML schema | Generated Rust type |
|---|---|
| `power_status` | `types::PowerStatus` |
| `power_status_all` | `types::PowerStatusAll` |
| `power_status_get` | `types::PowerStatusGet` |
| `transitions_getID` | `types::TransitionsGetId` |
| `transitions_getAll` | `types::TransitionsGetAll` |
| `transitions_get` | `types::TransitionsGet` |
| `transition_start_output` | `types::TransitionStartOutput` |
| `transitions_abort` | `types::TransitionsAbort` |
| `transition_task_data` | `types::TransitionTaskData` (contains nested `TransitionTaskDataTaskStatus` enum) |
| `reserved_location` | `types::ReservedLocation` |
| `transition_create` | `types::TransitionCreate` (contains nested `TransitionCreateOperation` enum) |
| `task_counts` | `types::TaskCounts` |
| `Problem7807` | `types::Problem7807` |
| `power_state` | `types::PowerState` (enum) |
| `power_operation` | `types::PowerOperation` (enum) |
| `transition_status` | `types::TransitionStatus` (enum) |
| `management_state` | `types::ManagementState` (enum) |
| `non_empty_string_list` | `types::NonEmptyStringList` (newtype `pub struct NonEmptyStringList(pub Vec<String>);`) — emitted but no longer referenced after the F.3 patch (see Section F) |
| `xname` | `types::Xname` (constrained-string newtype with one massive `regress::Regex` pattern; the only `regress::` use in the file) |
| `task_id` | `types::TaskId` (newtype `pub struct TaskId(pub uuid::Uuid);`) |
| `power_cap_patch` | `types::PowerCapPatch` |
| `power_cap_patch_component` | `types::PowerCapPatchComponent` |
| `power_cap_patch_component_control` | `types::PowerCapPatchComponentControl` |
| `op_task_start_response` | `types::OpTaskStartResponse` |
| `power_caps_retdata` | `types::PowerCapsRetdata` (contains nested `PowerCapsRetdataType` enum) |
| `power_cap_task_list` | `types::PowerCapTaskList` |
| `power_cap_task_info` | `types::PowerCapTaskInfo` (contains nested `PowerCapTaskInfoType` enum) |
| `power_cap_snapshot_req` | `types::PowerCapSnapshotReq` |
| `rsp_power_cap_components` | `types::RspPowerCapComponents` |
| `rsp_power_cap_components_control` | `types::RspPowerCapComponentsControl` (contains nested `RspPowerCapComponentsControlName` enum) |
| `capabilities_limits` | `types::CapabilitiesLimits` |
| `health_rsp` | `types::HealthRsp` |

### A.0.2. Per-operation merged error response types

Unlike BOS, PCS uses a single shared error schema (`Problem7807`) and every
4xx/5xx response is declared as a plain `$ref:
'#/components/schemas/Problem7807'`. Progenitor emits a single
`types::Problem7807` and every method that can fail returns
`Error<types::Problem7807>`. There are no per-operation merged error types,
and the wrapper can share one error-conversion helper across operations.

## A.1. Schemas referenced by the plan but NOT generated

None of the 12 plan-listed names are missing — 9 are simply named differently
in the upstream spec (see the "*not in spec*" rows in A.0), and 2 (`Liveness`,
`Readiness`) have no schema because those endpoints return 204 No Content.
All 32 component schemas listed in `components.schemas` produced Rust types.

## B. Generated method names (by operationId)

PCS upstream ships with **no `operationId:` declarations** — all 13
operations had to be patched with explicit operationIds before progenitor
would accept the spec (see Section F.1). Once patched, progenitor emits one
`Client::<operationId>` per operation, snake_case, verbatim.

### B.1. The 10 operations csm-rs's PCS public API uses today

| YAML operationId | Generated method | HTTP verb + path | Key args | Success response | Error variant |
|---|---|---|---|---|---|
| `post_transition` | `Client::post_transition` | POST `/transitions` | `body: &TransitionCreate` | `ResponseValue<types::TransitionStartOutput>` | `Error<types::Problem7807>` |
| `get_transitions` | `Client::get_transitions` | GET `/transitions` | (none) | `ResponseValue<types::TransitionsGetAll>` | `Error<types::Problem7807>` |
| `get_transition` | `Client::get_transition` | GET `/transitions/{transitionID}` | `transition_id: &uuid::Uuid` | `ResponseValue<types::TransitionsGetId>` | `Error<types::Problem7807>` |
| `abort_transition` | `Client::abort_transition` | DELETE `/transitions/{transitionID}` | `transition_id: &uuid::Uuid` | `ResponseValue<types::TransitionsAbort>` (202) | `Error<types::Problem7807>` |
| `get_power_status` | `Client::get_power_status` | GET `/power-status` | `management_state_filter: Option<types::ManagementState>`, `power_state_filter: Option<types::PowerState>`, `xname: Option<&str>` | `ResponseValue<types::PowerStatusAll>` | `Error<types::Problem7807>` |
| `post_power_status` | `Client::post_power_status` | POST `/power-status` | `body: &PowerStatusGet` | `ResponseValue<types::PowerStatusAll>` | `Error<types::Problem7807>` |
| `post_power_cap_snapshot` | `Client::post_power_cap_snapshot` | POST `/power-cap/snapshot` | `body: &PowerCapSnapshotReq` | `ResponseValue<types::OpTaskStartResponse>` | `Error<types::Problem7807>` |
| `patch_power_cap` | `Client::patch_power_cap` | PATCH `/power-cap` | `body: &PowerCapPatch` | `ResponseValue<types::OpTaskStartResponse>` | `Error<types::Problem7807>` |
| `get_power_cap_tasks` | `Client::get_power_cap_tasks` | GET `/power-cap` | (none) | `ResponseValue<types::PowerCapTaskList>` | `Error<types::Problem7807>` |
| `get_power_cap_task` | `Client::get_power_cap_task` | GET `/power-cap/{taskID}` | `task_id: &TaskId` | `ResponseValue<types::PowerCapsRetdata>` | `Error<types::Problem7807>` |

Notes on B.1:

- The `transition_id` parameter is a plain `&uuid::Uuid` — the spec declares
  the path parameter as `type: string, format: uuid` *without* a named
  schema, so progenitor uses the native `uuid::Uuid` type directly (no
  `TransitionId` newtype).
- The `xname` query parameter is `Option<&str>` after the F.3 patch (was
  originally `Option<&'a Vec<String>>` from the `non_empty_string_list`
  schema with `style: form, explode: true`, which progenitor 0.8 cannot
  serialise — see Section F).
- All 10 csm-rs operations return the same `Error<types::Problem7807>` —
  one shared error-conversion helper covers every wrapper method (much
  simpler than BOS, where each operation had its own merged error struct).

### B.2. The other 3 operations (not in csm-rs public API)

| operationId | HTTP verb + path | Success response | Error variant |
|---|---|---|---|
| `get_liveness` | GET `/liveness` | `ResponseValue<()>` (204) | `Error<()>` |
| `get_readiness` | GET `/readiness` | `ResponseValue<()>` (204) | `Error<()>` |
| `get_health` | GET `/health` | `ResponseValue<types::HealthRsp>` | `Error<types::Problem7807>` |

These all live on `Client` as plain `pub async fn <operationId>`. Task 1 will
not expose them but may surface a thin pass-through if/when needed.

### B.3. Method-name mangling rule

All operationIds added in F.1 are already snake_case (e.g. `post_transition`,
`get_power_cap_task`, `abort_transition`). progenitor passes them through
unchanged. There is no CamelCase→snake_case conversion to track. The
operationId values were chosen to mirror the historical csm-rs function
names in `src/pcs/*/http_client*.rs` where possible.

## C. The `Error` enum

Defined in `progenitor-client 0.8.0` at
`progenitor_client/src/progenitor_client.rs:236`. It is `pub use`d through the
generated file.

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

Distribution of `E` parameters in the generated PCS code:

- `Error<()>` — 2 ops: `get_liveness`, `get_readiness`. Used when the spec
  declares no typed 4xx/5xx response for that operation.
- `Error<types::Problem7807>` — the other 11 ops. PCS uses a single shared
  RFC 7807 error schema for every typed error response, so the wrapper can
  share one error-conversion helper across all operations (cleaner than
  BOS, which had per-op merged error types).

## D. The constructor signatures

Copied verbatim from the generated file:

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
    pub fn api_version(&self) -> &'static str { "1.1.2" }
}
```

Implication for the PCS wrapper: identical to the HSM/CFS/BSS/BOS migrations —
`Client::new` bakes a 15-second connect+request timeout. The wrapper's
`gen_client()` helper MUST use `Client::new_with_client` so the project's
shared `reqwest::Client` controls the timeout, rustls config, and
insecure-TLS toggle.

`api_version()` returns `"1.1.2"` (matches the upstream `info.version`).

## E. basePath behaviour

The PCS OpenAPI 3.0.0 spec declares:

```yaml
servers:
  - url: https://api-gw-service-nmn.local/apis/power-control/v1
    description: Production API service.  Access from outside the service mesh.
  - url: 'http://cray-power-control/v1'
    description: Access from inside the service mesh.
  - url: https://loki-ncn-m001.us.cray.com/apis/power-control/v1
    description: External API access.
  - url: http://localhost:26970
    description: Localhost access to power-control
```

Progenitor does NOT auto-prepend `servers[0].url`. Every generated method
computes its URL as `format!("{}/<operation path>", self.baseurl)`, where the
operation path is what's literally written under `paths:` in the spec (e.g.
`/liveness`, `/transitions`, `/power-cap/{taskID}`). The `/power-control/v1`
prefix from the server URL is *not* included in operation paths.

This was verified by two wiremock unit tests in
`/tmp/progenitor-pcs-smoke/src/lib.rs`:

1. With `Client::new_with_client(&server.uri(), …)`, a `client.get_liveness()`
   call hits `/liveness` — mock matches without the `/power-control/v1`
   prefix.
2. With `Client::new_with_client(&format!("{}/power-control/v1",
   server.uri()), …)`, the same call hits `/power-control/v1/liveness` —
   mock matches.

Both passed (`cargo test sanity` in the smoke crate: `test result: ok.
2 passed`).

**Operational consequence:** the PCS wrapper must build its base URL as
`format!("{}/power-control/v1", shasta_client.base_url())` because csm-rs's
`base_url` already ends in `/apis` (e.g. `https://api.cmn.alps.cscs.ch/apis`)
and the historical http_client URLs (`src/pcs/*/http_client*.rs`) all use
that `/power-control/v1` segment between the base URL and the operation
path. The operation paths in the spec do NOT carry the `/power-control/v1`
prefix.

## F. Spec patches applied

Three patches were applied to `src/pcs/csm_api_docs.yaml` before progenitor
would produce a buildable generated.rs. All three are committed in the same
commit as this reference doc.

### F.1. Add `operationId:` to all 13 path/verb combinations

PCS upstream ships with no `operationId:` declarations on any of its 13
operations. Without operationIds progenitor panics at codegen time with
`UnexpectedFormat("path /transitions is missing operation ID")`.

The added operationIds, in spec order, are:

| Path | Verb | operationId |
|---|---|---|
| `/transitions` | POST | `post_transition` |
| `/transitions` | GET | `get_transitions` |
| `/transitions/{transitionID}` | GET | `get_transition` |
| `/transitions/{transitionID}` | DELETE | `abort_transition` |
| `/power-status` | GET | `get_power_status` |
| `/power-status` | POST | `post_power_status` |
| `/power-cap/snapshot` | POST | `post_power_cap_snapshot` |
| `/power-cap` | PATCH | `patch_power_cap` |
| `/power-cap` | GET | `get_power_cap_tasks` |
| `/power-cap/{taskID}` | GET | `get_power_cap_task` |
| `/liveness` | GET | `get_liveness` |
| `/readiness` | GET | `get_readiness` |
| `/health` | GET | `get_health` |

The names were chosen to mirror the existing csm-rs function names in
`src/pcs/transitions/`, `src/pcs/power_status/`, and `src/pcs/power_cap/`
where possible, while keeping the `<verb>_<resource>` pattern consistent
with HSM/CFS/BSS/BOS so the wrapper code is uniform across services.

### F.2. Rewrite `application/error` content entries to `application/json` (19 occurrences) and drop the duplicate `application/problem+json` entry on `/health`'s 405

PCS uses the non-standard `application/error` content-type key for every
4xx/5xx response, plus a single redundant `application/problem+json` entry
on the `/health` 405 response. Progenitor's "extract response" step
considers only the `application/json` media type for the typed return path;
non-json media types fall through to `progenitor_client::ByteStream`. With
the original content-types in place, every method that referenced an error
response got `Result<…, Error<progenitor_client::ByteStream>>`, and because
`ByteStream` does not implement `Debug`, the generated file failed to
compile with 11 `error[E0277]: ByteStream doesn't implement Debug` errors at
every 4xx/5xx match arm.

The patch is a global rewrite (`sed s|application/error:|application/json:|g`)
plus a small edit on the `/health` 405 response to remove the
now-duplicate `application/problem+json` key (both pointed at the same
`Problem7807` schema, so dropping the alias is a no-op semantically).

### F.3. Inline the `xname` query parameter as `type: string` (1 parameter rewritten)

The `/power-status` GET endpoint declares `xname` as a query parameter that
`$ref`'s the `non_empty_string_list` schema (an `array of string` with
`minItems: 1`), with `style: form, explode: true` — i.e. the upstream API
expects `?xname=a&xname=b&xname=c`.

Progenitor 0.8 cannot emit query-parameter serialisation for arrays; it
unconditionally calls `.to_string()` on each query value, and
`Vec<String>` does not implement `Display`. The generated file failed with
`error[E0599]: the method 'to_string' exists for reference '&&Vec<String>',
but its trait bounds were not satisfied`.

The patch inlines `schema: { type: string }` on the parameter. This downgrades
the parameter to a single string (the caller can either supply one xname per
call, or fall back to the `POST /power-status` form which takes a JSON body
with an `xnames` array). After the patch the wrapper method signature uses
`xname: Option<&'a str>`. The `non_empty_string_list` newtype itself is
still emitted in `mod types` (because the schema is still defined in
`components.schemas`), but nothing in the generated code references it
after the patch.

This is the only callsite that referenced `non_empty_string_list`; no other
operation is affected.

### F.4. Patches NOT required (vs HSM/CFS/BSS/BOS)

None of the following patches that earlier migrations needed were needed for
PCS:

- **No Swagger 2.0 → OpenAPI 3.0 conversion**. PCS ships as OpenAPI 3.0.0
  upstream, unlike BSS (Swagger 2.0).
- **No `$ref` to sub-keys inside another schema's `properties`**. PCS does
  not share property definitions across schemas the way CFS does.
- **No `default:` response panics**. Every operation declares only explicit
  status codes (`200`, `202`, `204`, `400`, `404`, `405`, `500`, `503`) and
  no `default:` arms, so the `response_types.len() <= 1` assertion in
  `progenitor-impl/src/method.rs:1264` never fires.
- **No tenant-header / constrained-string inline patch**. PCS does not have
  a `Cray-Tenant-Name` header parameter (unlike BOS/CFS).
- **No per-operation merged error types**. PCS uses a single shared
  `Problem7807` schema as a plain `$ref` (not `allOf`), so progenitor emits
  exactly one error struct and every fallible method shares
  `Error<types::Problem7807>` (much simpler than BOS).

## G. Generated artefact stats

- Total generated file: 2,779 lines (≈91 KB) — by far the smallest among the
  five Task 0 spikes; smaller than BSS (2,326 lines is a separate count but
  comparable order), much smaller than CFS (13,589), BOS (12,904), and HSM
  (37,574).
- Type count in `mod types`: 39 (30 structs + 9 enums); plus the standard
  `ConversionError` newtype in `mod types::error`. (The 30-struct count
  includes both `ConversionError` and the `NonEmptyStringList` orphan
  newtype that is emitted but unreferenced after the F.3 patch.)
- Method count on `impl Client`: 13 `pub async fn`s — one per operationId.
- One `pub struct Client` at crate root, plus its inherent impls (`new`,
  `new_with_client`, `baseurl`, `client`, `api_version`).
- `cargo build` and `cargo clippy` both succeed with zero warnings.

## H. Runtime crate dependencies

The generated PCS code references the following extern crates:

| Crate | Already in csm-rs Cargo.toml? | Action |
|---|---|---|
| `progenitor_client` | yes (added for HSM) | none |
| `reqwest` | yes | none |
| `serde` | yes | none |
| `serde_json` | yes | none |
| `chrono` (with `serde` feature) | yes (added for HSM) | none |
| `regress` | yes (added for HSM) | none |
| `uuid` (with `serde` feature) | yes (added for HSM) | none |

Crate usage counts in the generated file:

- `chrono::` — 5 references (`chrono::DateTime<chrono::offset::Utc>` for the
  `automatic_expiration_time` and `last_updated` timestamp fields on
  several transition/task structs).
- `regress::` — 1 reference (`regress::Regex::new(...)` in the `Xname`
  constrained-string newtype's `FromStr` impl — the only `pattern`-validated
  string in the spec).
- `uuid::` — 19 references. PCS uses `uuid::Uuid` natively for the
  `transitionID` path parameter (inline `format: uuid`) and the `task_id`
  schema (`pub struct TaskId(pub uuid::Uuid);`).

The `progenitor-client`, `chrono` (with `serde`), `regress`, and `uuid` (with
`serde`) feature additions already committed for HSM, CFS, BSS, and BOS more
than cover PCS. **No new dependencies, and no feature-flag tweaks, are
required to land Task 1.**
