# Progenitor output reference — `src/bss/csm_api_docs.openapi3.json`

Generated against `progenitor` 0.8 (build via `progenitor::Generator::default()`
in a `build.rs` mirroring the existing HSM/CFS ones). The smoke crate that
produced this reference lived in `/tmp/progenitor-bss-smoke` and is not
committed.

The spec fed to progenitor is the committed
`src/bss/csm_api_docs.openapi3.json` (OpenAPI 3.0.0, machine-converted from
the upstream Swagger 2.0 `src/bss/csm_api_docs.yaml` via `swagger2openapi`).
Two small patches were applied to the converted JSON before progenitor would
accept it; see Section F. Both patches are committed in the same commit as
this reference doc.

All generated types live in `mod types`. The client struct is `Client` at the
crate root.

## A. Generated type names (by JSON schema name)

The 10 schemas listed in the Task 0 description, plus the auxiliary types
progenitor synthesised for inline enum/object response shapes, are
enumerated below. The full inventory of generated types is in the generated
file; grep `pub (struct|enum) ` to enumerate them (19 structs + 9 enums =
28 types in `mod types`, plus one `ConversionError` newtype in
`mod types::error`).

### A.0. The 10 schemas the plan explicitly lists

| JSON schema name | Kind | Generated Rust type (`types::…`) |
|---|---|---|
| `BootParams` | object | `types::BootParams` |
| `CloudInit` | object | `types::CloudInit` |
| `CloudInitMetadata` | object (free-form) | `types::CloudInitMetadata` (newtype `pub struct CloudInitMetadata(pub ::serde_json::Map<String, ::serde_json::Value>);`) |
| `CloudInitUserData` | object (free-form) | `types::CloudInitUserData` (newtype `pub struct CloudInitUserData(pub ::serde_json::Map<String, ::serde_json::Value>);`) |
| `CloudInitPhoneHome` | object | `types::CloudInitPhoneHome` |
| `Component` | object | `types::Component` |
| `StateInfo` | object | `types::StateInfo` |
| `HostInfo` | array of `Component` | `types::HostInfo` (newtype `pub struct HostInfo(pub Vec<Component>);`) |
| `EndpointAccess` | object | `types::EndpointAccess` |
| `Error` | object (RFC 7807-ish problem details) | `types::Error` |

All 10 plan-listed schemas survived codegen with the schema name intact.

### A.0.1. Other types progenitor synthesised

These are not in the original schema list — they are inline objects/enums
extracted from operation parameters or response bodies and named after the
operation:

| Generated Rust type | Origin |
|---|---|
| `types::EndpointAccessEndpoint` | inline enum from `EndpointAccess.endpoint` (string enum: `bootscript`, `user-data`) |
| `types::GetEndpointHistoryEndpoint` | inline enum from `GET /boot/v1/endpoint-history`'s `endpoint` query param (same string enum) |
| `types::GetServiceEtcdResponse` + `GetServiceEtcdResponseBssStatusEtcd` | inline object from `GET /boot/v1/service/etcd` 200 response |
| `types::GetServiceHsmResponse` + `GetServiceHsmResponseBssStatusHsm` | inline object from `GET /boot/v1/service/hsm` 200 response |
| `types::GetServiceStatusAllResponse` + `GetServiceStatusAllResponseBssStatus` + …`Etcd` + …`Hsm` | inline object from `GET /boot/v1/service/status/all` 200 response (3 nested enums) |
| `types::GetServiceStatusResponse` + `GetServiceStatusResponseBssStatus` | inline object from `GET /boot/v1/service/status` 200 response |
| `types::GetServiceVersionResponse` + `GetServiceVersionResponseBssVersion` | inline object from `GET /boot/v1/service/version` 200 response |
| `types::StateInfoComponentsItem` | inline object from `StateInfo.Components` items |
| `types::StateInfoParamsItem` | inline object from `StateInfo.Params` items |

The csm-rs BSS public API does not interact with the `/boot/v1/service/*`
or `/boot/v1/dumpstate` operations (Section B), so the wrapper does not need
to re-export the synthesised `GetService*Response` types.

## A.1. Schemas referenced by the plan but NOT generated

None. All 10 schemas listed in the Task 0 plan exist verbatim as Rust
types in `mod types`.

## B. Generated method names (by operationId)

The BSS Swagger 2.0 spec only declared `operationId` on 4 of the 18
operations: `meta_data_get`, `user_data_get`, `phone_home_post`,
`bootscript_get`. progenitor refuses to codegen any operation without an
operationId, so 14 operationIds were added to the converted OpenAPI 3.0
spec (see Section F). The names chosen follow the snake_case
`<verb>_<noun>` style.

### B.1. The 6 operations the csm-rs BSS wrapper actually uses

These all live on `/boot/v1/bootparameters`.

| Added operationId | Generated method | HTTP verb + path | Request body | Success response |
|---|---|---|---|---|
| `get_boot_parameters` | `Client::get_boot_parameters` | GET `/boot/v1/bootparameters` | `&types::BootParams` (yes, a GET with body — that's how the upstream spec is written) | `ResponseValue<Vec<types::BootParams>>` |
| `post_boot_parameters` | `Client::post_boot_parameters` | POST `/boot/v1/bootparameters` | `&types::BootParams` | `ResponseValue<()>` (201) |
| `put_boot_parameters` | `Client::put_boot_parameters` | PUT `/boot/v1/bootparameters` | `&types::BootParams` | `ResponseValue<()>` (200) |
| `patch_boot_parameters` | `Client::patch_boot_parameters` | PATCH `/boot/v1/bootparameters` | `&types::BootParams` | `ResponseValue<()>` (200) |
| `delete_boot_parameters` | `Client::delete_boot_parameters` | DELETE `/boot/v1/bootparameters` | `&types::BootParams` | `ResponseValue<()>` (200) |

The error variant for all five is `Error<types::Error>`.

Note: `get_boot_parameters` also takes three optional query parameters
(`mac: Option<&str>`, `name: Option<&str>`, `nid: Option<i64>`) in addition
to the body. The wrapper will need to thread those through.

The Task 0 plan asked for "the 6 of interest" but only 5 verbs are defined
on `/boot/v1/bootparameters` in this spec (GET, POST, PUT, PATCH, DELETE).
The 6th of csm-rs's existing BSS hand-written methods presumably maps to
the `/boot/v1/bootscript` lookup; see Section B.2 below.

### B.2. The other 13 generated operations (not in csm-rs public API)

| Added operationId (or original) | Generated method | HTTP verb + path |
|---|---|---|
| `meta_data_get` (original) | `Client::meta_data_get` | GET `/meta-data` |
| `user_data_get` (original) | `Client::user_data_get` | GET `/user-data` |
| `phone_home_post` (original) | `Client::phone_home_post` | POST `/phone-home` |
| `bootscript_get` (original) | `Client::bootscript_get` | GET `/boot/v1/bootscript` |
| `get_hosts` | `Client::get_hosts` | GET `/boot/v1/hosts` |
| `post_hosts` | `Client::post_hosts` | POST `/boot/v1/hosts` |
| `get_dumpstate` | `Client::get_dumpstate` | GET `/boot/v1/dumpstate` |
| `get_endpoint_history` | `Client::get_endpoint_history` | GET `/boot/v1/endpoint-history` |
| `get_service_status` | `Client::get_service_status` | GET `/boot/v1/service/status` |
| `get_service_etcd` | `Client::get_service_etcd` | GET `/boot/v1/service/etcd` |
| `get_service_hsm` | `Client::get_service_hsm` | GET `/boot/v1/service/hsm` |
| `get_service_version` | `Client::get_service_version` | GET `/boot/v1/service/version` |
| `get_service_status_all` | `Client::get_service_status_all` | GET `/boot/v1/service/status/all` |

`bootscript_get` and `user_data_get` return non-JSON content types
(`text/plain`, `text/yaml` respectively), so progenitor types them as
`Result<ResponseValue<ByteStream>, Error<ByteStream>>` — the wrapper would
need to read the body bytes explicitly if it ever called them. They are
listed here only for completeness; csm-rs does not call them.

### B.3. Method-name mangling rule

operationIds in the spec are snake_case. progenitor's typify pass passes
them through unchanged. There is no CamelCase→snake_case conversion to
track. The path-derived names invented in Section F (`get_boot_parameters`
etc.) follow the same convention so the wrapper can call them without any
further renaming.

## C. The `Error` enum

Defined in `progenitor-client 0.8.0` at
`progenitor_client/src/progenitor_client.rs:236`. It is `pub use`d through
the generated file (line 2 of generated.rs).

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

Three different `E` parameters appear in the generated BSS code:

- `Error<types::Error>` — the majority. Used by all 5 `/boot/v1/bootparameters`
  ops (the ones csm-rs cares about), plus `meta_data_get`, `phone_home_post`,
  `get_dumpstate`, `post_hosts`, `get_service_*` (most of them).
- `Error<ByteStream>` — `user_data_get` and `bootscript_get`. The 4xx
  responses on these ops are `text/yaml` or `text/plain`, not JSON, so
  progenitor cannot synthesise a typed `Error<types::Error>`; it falls back
  to the raw `ByteStream`.
- `Error<()>` — `get_hosts`, `get_endpoint_history`. The spec defines no
  4xx response for these, so progenitor emits the unit type for the error
  parameter.

Wrapper code unwrapping these `Error` values for the
`/boot/v1/bootparameters` ops only needs to handle `Error<types::Error>`.

## D. The constructor signatures

Copied verbatim from the generated file (lines 1576–1623):

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
    pub fn api_version(&self) -> &'static str { "1.0.0" }
}
```

Implication for the BSS wrapper: identical to the HSM/CFS migrations —
`Client::new` bakes a 15-second connect+request timeout. The wrapper's
`gen_client()` helper MUST use `Client::new_with_client` so the project's
shared `reqwest::Client` controls the timeout, rustls config, and
insecure-TLS toggle.

`api_version()` returns `"1.0.0"` (from the spec's `info.version: 1.0.0`,
unchanged across the Swagger→OpenAPI conversion).

## E. basePath behaviour

The BSS Swagger 2.0 spec declared `host: bootscriptserver:27778` +
`basePath: /apis/bss`. `swagger2openapi` translated those into a single
`servers` entry:

```json
"servers": [
  { "url": "http://bootscriptserver:27778/apis/bss" }
]
```

Progenitor does NOT auto-prepend `servers[0].url`. Every generated method
computes its URL as `format!("{}/<operation path>", self.baseurl)`, where
the operation path is what's literally written under `paths:` in the spec
(`/meta-data`, `/boot/v1/bootparameters`, `/boot/v1/hosts`, …).

This was verified by two wiremock unit tests in
`/tmp/progenitor-bss-smoke/src/lib.rs`:

1. With `Client::new_with_client(&server.uri(), …)`, a
   `client.meta_data_get(None)` call hits `/meta-data` — mock matches
   without the `/apis/bss` prefix.
2. With `Client::new_with_client(&format!("{}/apis/bss", server.uri()), …)`,
   the same call hits `/apis/bss/meta-data` — mock matches.

Both passed (`cargo test sanity` in the smoke crate: `test result: ok.
2 passed`).

**Operational consequence:** the BSS wrapper must build its base URL as
`format!("{}/bss", shasta_client.base_url())` because csm-rs's
`base_url` already ends in `/apis` (e.g.
`https://api.cmn.alps.cscs.ch/apis`). The operation paths in the spec
already carry the `/boot/v1/...` prefix.

Note that the bare `/meta-data`, `/user-data`, `/phone-home`, and
`/boot/v1/bootscript` paths (the cloud-init / iPXE-script endpoints) are
served from the same base — they have no `/v1` discriminator. csm-rs does
not currently call them, so this asymmetry does not affect the wrapper.

## F. Spec patches applied

Two patches were applied to the converted JSON
(`src/bss/csm_api_docs.openapi3.json`) before progenitor would produce a
buildable generated.rs. Both are committed in the same commit as this
reference doc.

### F.1. Add 14 missing operationIds

The upstream Swagger 2.0 spec only declared `operationId` on 4 of the 18
operations. progenitor 0.8 refuses to codegen anything else:

    thread 'main' panicked at build.rs:12:51:
    progenitor codegen: UnexpectedFormat("path /boot/v1/bootparameters is missing operation ID")

The patch adds the following 14 operationIds, all snake_case
`<verb>_<noun>` based on the path:

| Path | Verb | Added operationId |
|---|---|---|
| `/boot/v1/bootparameters` | GET | `get_boot_parameters` |
| `/boot/v1/bootparameters` | POST | `post_boot_parameters` |
| `/boot/v1/bootparameters` | PUT | `put_boot_parameters` |
| `/boot/v1/bootparameters` | PATCH | `patch_boot_parameters` |
| `/boot/v1/bootparameters` | DELETE | `delete_boot_parameters` |
| `/boot/v1/hosts` | GET | `get_hosts` |
| `/boot/v1/hosts` | POST | `post_hosts` |
| `/boot/v1/dumpstate` | GET | `get_dumpstate` |
| `/boot/v1/endpoint-history` | GET | `get_endpoint_history` |
| `/boot/v1/service/status` | GET | `get_service_status` |
| `/boot/v1/service/etcd` | GET | `get_service_etcd` |
| `/boot/v1/service/hsm` | GET | `get_service_hsm` |
| `/boot/v1/service/version` | GET | `get_service_version` |
| `/boot/v1/service/status/all` | GET | `get_service_status_all` |

The names for the 5 `/boot/v1/bootparameters` ops are deliberately distinct
from the existing csm-rs hand-written method names (`bootparameters_get`,
etc.) so the wrapper layer can re-export with whatever name the public API
requires without colliding with the progenitor symbols.

### F.2. Drop `default:` response from 7 operations

Progenitor 0.8 panics with `assertion failed: response_types.len() <= 1`
in `progenitor-impl-0.8.0/src/method.rs:1264` whenever a single operation
declares both a typed success response AND a typed `default:` response
whose response type differs from the success type. This is exactly the
shape the original BSS Swagger 2.0 spec uses for the 4 cloud-init /
bootscript ops and the 3 bootparameters GET/POST/PUT — `200` returns the
operation-specific payload (or no body), and `default:` returns
`#/definitions/Error`. After swagger2openapi conversion, the success and
default branches sometimes share a content type (`text/yaml`,
`text/plain`, `application/json`) but never share the response schema, so
the two distinct types collide in `extract_responses`:

    thread 'main' panicked at progenitor-impl-0.8.0/src/method.rs:1264:9:
    assertion failed: response_types.len() <= 1

The patch deletes the `default:` response entry from these 7 operations:

| Path | Verb |
|---|---|
| `/meta-data` | GET |
| `/user-data` | GET |
| `/phone-home` | POST |
| `/boot/v1/bootscript` | GET |
| `/boot/v1/bootparameters` | GET |
| `/boot/v1/bootparameters` | POST |
| `/boot/v1/bootparameters` | PUT |

Effect on the generated code: instead of an explicit `_ => default arm`
matching the `default:` schema, progenitor emits
`_ => Err(Error::UnexpectedResponse(response))`. The explicit 4xx/5xx
arms (`400`, `404`, `500`) are untouched and still decode into
`Error<types::Error>`. The Task-0 callers and the Task-1 wrapper see no
behavioural change for documented status codes; only the catch-all for
undocumented codes becomes `UnexpectedResponse` instead of
`ErrorResponse(types::Error)`.

(`/boot/v1/bootparameters` PATCH and DELETE had no `default:` to begin
with in the original spec, so they were unaffected.)

### F.3. Patches NOT required

None of the following CFS-style patches were needed for BSS:

- No `$ref` to sub-keys inside another schema's `properties` — BSS
  doesn't share property definitions between schemas.
- No `application/problem+json` content types — BSS uses
  `application/json` for `Error` bodies.
- No constrained-string newtypes leaking into headers — BSS doesn't use
  custom request headers.

## G. Generated artefact stats

- Total generated file: 2,326 lines (≈80 KB) — by far the smallest of
  the three migrations (HSM: 37,574 lines; CFS: 13,589 lines).
- Type count in `mod types`: 28 (19 structs + 9 enums; one additional
  `ConversionError` newtype lives in `mod types::error`).
- Method count on `impl Client`: 18 `pub async fn`s — one per
  operationId (4 original + 14 added in F.1).
- One `pub struct Client` at crate root, plus its inherent impls
  (`new`, `new_with_client`, `baseurl`, `client`, `api_version`).

## H. Runtime crate dependencies

The generated BSS code references the following extern crates:

| Crate | Already in csm-rs Cargo.toml? | Action |
|---|---|---|
| `progenitor_client` | yes (added for HSM) | none |
| `reqwest` | yes | none |
| `serde` | yes | none |
| `serde_json` | yes | none |

**Crates NOT referenced by the generated BSS code** (so no Cargo.toml
changes needed for BSS):

- `chrono` — `grep chrono:: generated.rs` returns 0. No date/time fields
  in any BSS schema.
- `uuid` — `grep uuid:: generated.rs` returns 0. BSS uses plain strings
  for identifiers (xnames, MACs).
- `regress` — `grep regress:: generated.rs` returns 0. No
  `pattern`-constrained string newtypes need runtime regex validation.

The `progenitor-client`, `chrono` (with `serde`), and `uuid` (with
`serde`) feature additions already committed for HSM and CFS more than
cover BSS. **No new dependencies, and no feature-flag tweaks, are
required to land Task 1.**
