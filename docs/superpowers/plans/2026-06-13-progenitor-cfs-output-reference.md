# Progenitor output reference — `src/cfs/csm_api_docs.yaml`

Generated against `progenitor` 0.8 (build via `progenitor::Generator::default()`
in a `build.rs` mirroring the existing HSM one). The smoke crate that produced
this reference lived in `/tmp/progenitor-cfs-smoke` and is not committed.

The spec fed to progenitor is the committed `src/cfs/csm_api_docs.yaml` (OpenAPI
3.0.2), patched in three small ways inside the smoke crate before progenitor
would accept it (see Section F). Those patches will need to be applied to the
committed `src/cfs/csm_api_docs.yaml` (or applied at build time) when the real
client is wired up in Task 1.

All generated types live in `mod types`. The client struct is `Client` at the
crate root.

## A. Generated type names (by YAML schema name)

The table maps every schema/requestBody/response referenced by Tasks 3–8 of
the implementation plan to its progenitor-generated Rust path. The full
inventory of generated types (153 total: 124 structs + 29 enums) is in the
generated file; grep `pub (struct|enum) ` to enumerate them.

| YAML name | Kind | Generated Rust type (`generated::types::…`) |
|---|---|---|
| `V2SessionCreateRequest` | requestBody → `V2SessionCreate` schema | `types::V2SessionCreate` |
| `V3SessionCreateRequest` | requestBody → `V3SessionCreate` schema | `types::V3SessionCreate` |
| `V2SessionDetails` | response → `V2Session` schema | `types::V2Session` |
| `V3SessionData` | response & schema | `types::V3SessionData` |
| `V2SessionDetailsArray` | response → `V2SessionArray` schema | `types::V2SessionArray` |
| `V3SessionDataCollection` | response & schema | `types::V3SessionDataCollection` |
| `V3SessionIdCollection` | response & schema | `types::V3SessionIdCollection` |
| `V2ComponentsCreateRequest` | requestBody → `V2ComponentStateArray` | `types::V2ComponentStateArray` |
| `V3ComponentsCreateRequest` | requestBody → `V3ComponentDataArray` | `types::V3ComponentDataArray` |
| `V2ComponentsUpdateRequest` | requestBody → `V2ComponentsUpdate` or array | see Section A.1 |
| `V3ComponentsUpdateRequest` | requestBody → `V3ComponentsUpdate` or array | see Section A.1 |
| `V2ComponentUpdateRequest` | requestBody → `V2ComponentState` | `types::V2ComponentState` |
| `V3ComponentUpdateRequest` | requestBody → `V3ComponentData` | `types::V3ComponentData` |
| `V2ComponentDetails` | response → `V2ComponentState` schema | `types::V2ComponentState` |
| `V3ComponentData` | response & schema | `types::V3ComponentData` |
| `V2ComponentDetailsArray` | response → `V2ComponentStateArray` schema | `types::V2ComponentStateArray` |
| `V3ComponentDataCollection` | response & schema | `types::V3ComponentDataCollection` |
| `V3ComponentIdCollection` | response & schema | `types::V3ComponentIdCollection` |
| `V2ConfigurationUpdateRequest` | requestBody → `V2Configuration` | `types::V2Configuration` |
| `V3ConfigurationUpdateRequest` | requestBody → `V3ConfigurationData` | `types::V3ConfigurationData` |
| `V2Options` | schema & response | `types::V2Options` |
| `V3Options` | schema & response | `types::V3Options` |
| `V2OptionsUpdateRequest` | requestBody → `V2Options` | `types::V2Options` |
| `V3OptionsUpdateRequest` | requestBody → `V3Options` | `types::V3Options` |
| `V3SourceCreateRequest` | requestBody → `V3SourceCreateData` | `types::V3SourceCreateData` |
| `V3SourceUpdateRequest` | requestBody → `V3SourceUpdateData` | `types::V3SourceUpdateData` |
| `V3SourceRestoreRequest` | requestBody → `V3SourceRestoreData` | `types::V3SourceRestoreData` |
| `Version` | schema & response | `types::Version` |
| `Healthz` | schema & response | `types::Healthz` |

Other key schemas progenitor emitted that wrappers may reference:

| YAML schema | Generated Rust type |
|---|---|
| `V2Session` | `types::V2Session` |
| `V2SessionArray` | `types::V2SessionArray` (newtype `pub struct V2SessionArray(pub Vec<V2Session>);`) |
| `V3ComponentDataArray` | `types::V3ComponentDataArray` (newtype `pub struct V3ComponentDataArray(pub Vec<V3ComponentData>);`) |
| `V2ComponentStateArray` | `types::V2ComponentStateArray` (newtype `pub struct V2ComponentStateArray(pub Vec<V2ComponentState>);`) |
| `V2Configuration` | `types::V2Configuration` |
| `V2ConfigurationArray` | `types::V2ConfigurationArray` |
| `V3ConfigurationData` | `types::V3ConfigurationData` |
| `V3ConfigurationDataCollection` | `types::V3ConfigurationDataCollection` |
| `V3SourceData` | `types::V3SourceData` |
| `V3SourceDataCollection` | `types::V3SourceDataCollection` |
| `V3SourceCreateData` | `types::V3SourceCreateData` |
| `V3SourceUpdateData` | `types::V3SourceUpdateData` |
| `V3SourceRestoreData` | `types::V3SourceRestoreData` |
| `V2ComponentsUpdate` | `types::V2ComponentsUpdate` |
| `V3ComponentsUpdate` | `types::V3ComponentsUpdate` |
| `ProblemDetails` | `types::ProblemDetails` |
| `TenantName` | (see Section F — patched out; wrappers should use `&str`) |

Notes on the mangling:
- CFS operationIds are already snake_case in the YAML (`create_session_v3`,
  `get_options_v2`, etc.), so progenitor's typify pass passes them through
  unchanged. There is no CamelCase→snake_case mapping to track for methods.
- Schema names retain their CamelCase. Acronyms in CFS schema names are
  uniform `V2`/`V3` prefixes that typify lowercases as `v2`/`v3` only inside
  method names, never inside type names. Type names stay verbatim.
- Several "request body" names in the OpenAPI `components.requestBodies`
  section do NOT produce their own Rust types — they `$ref` a schema in
  `components.schemas`, and progenitor inlines the underlying schema's type
  name into the generated method signature. So `V2OptionsUpdateRequest`
  does not exist as a Rust struct; the underlying `V2Options` is what
  `patch_options_v2` takes.
- Similarly several "response" names in `components.responses` are
  pure indirection: `V2SessionDetails` resolves to schema `V2Session`,
  `V2SessionDetailsArray` to schema `V2SessionArray`, etc.

## A.1. Schemas referenced by the plan that are NOT generated (or appear under a different name)

The implementation plan references several names that progenitor does not
emit as standalone Rust types. Below is the mapping to substitute:

| Plan-mentioned name | Reality |
|---|---|
| `V2SessionDetails` | Not generated. Use `types::V2Session`. The OpenAPI `components.responses.V2SessionDetails` is a `$ref` to `components.schemas.V2Session`. |
| `V2SessionDetailsArray` | Not generated. Use `types::V2SessionArray`. |
| `V2ComponentDetails` | Not generated. Use `types::V2ComponentState`. |
| `V2ComponentDetailsArray` | Not generated. Use `types::V2ComponentStateArray`. |
| `V2ComponentsCreateRequest` | Not generated as a struct. Operation `put_components_v2` takes `&types::V2ComponentStateArray` (the underlying schema). |
| `V3ComponentsCreateRequest` | Not generated as a struct. Operation `put_components_v3` takes `&types::V3ComponentDataArray`. |
| `V2ComponentsUpdateRequest` | The `patch_components_v2` operation takes a `oneOf` of array vs `V2ComponentsUpdate`, materialised as `types::PatchComponentsV2Body` (auto-generated enum wrapper). |
| `V3ComponentsUpdateRequest` | Same shape: `types::PatchComponentsV3Body` (auto-generated enum wrapper). |
| `V2ComponentUpdateRequest` | Not generated as a struct. `patch_component_v2` takes `&types::V2ComponentState`. |
| `V3ComponentUpdateRequest` | Not generated as a struct. `patch_component_v3` takes `&types::V3ComponentData`. |
| `V2ConfigurationUpdateRequest` | Not generated. `put_configuration_v2` takes `&types::V2Configuration`. `patch_configuration_v2` takes a `oneOf` materialised as an inline enum (see generated file). |
| `V3ConfigurationUpdateRequest` | Not generated. `put_configuration_v3` takes `&types::V3ConfigurationData`. |
| `V2OptionsUpdateRequest` | Not generated. `patch_options_v2` takes `&types::V2Options`. |
| `V3OptionsUpdateRequest` | Not generated. `patch_options_v3` takes `&types::V3Options`. |
| `V3SourceCreateRequest` | Not generated. `post_source_v3` takes `&types::V3SourceCreateData`. |
| `V3SourceUpdateRequest` | Not generated. `patch_source_v3` takes `&types::V3SourceUpdateData`. |
| `V3SourceRestoreRequest` | Not generated. `restore_source_v3` takes `&types::V3SourceRestoreData`. |
| `V2ConfigurationDetails` / `V2ConfigurationDetailsArray` | Not generated. Use `types::V2Configuration` and `types::V2ConfigurationArray`. |
| `V3ConfigurationData` (response) | Plan name matches schema name; generated as `types::V3ConfigurationData`. |
| `V3SourceData` / `V3SourceDataCollection` | Plan names match schema names; generated as `types::V3SourceData`, `types::V3SourceDataCollection`. |

The rule is: when the plan references a name ending in `Request`,
`Details`, or `DetailsArray`, progenitor probably emitted a different
name. Always look up the operation in Section B and read the actual
signature in `generated.rs`.

## B. Generated method names (by operationId)

CFS operationIds are already snake_case in the YAML, so the Rust method
name equals the operationId verbatim. There are 51 operations in total.

| YAML operationId | Generated method | HTTP verb + path |
|---|---|---|
| `get_version` | `Client::get_version` | GET `/` |
| `get_versions` | `Client::get_versions` | GET `/versions` |
| `get_healthz` | `Client::get_healthz` | GET `/healthz` |
| `get_versions_v2` | `Client::get_versions_v2` | GET `/v2` |
| `get_options_v2` | `Client::get_options_v2` | GET `/v2/options` |
| `patch_options_v2` | `Client::patch_options_v2` | PATCH `/v2/options` |
| `get_sessions_v2` | `Client::get_sessions_v2` | GET `/v2/sessions` |
| `create_session_v2` | `Client::create_session_v2` | POST `/v2/sessions` |
| `delete_sessions_v2` | `Client::delete_sessions_v2` | DELETE `/v2/sessions` |
| `get_session_v2` | `Client::get_session_v2` | GET `/v2/sessions/{session_name}` |
| `patch_session_v2` | `Client::patch_session_v2` | PATCH `/v2/sessions/{session_name}` |
| `delete_session_v2` | `Client::delete_session_v2` | DELETE `/v2/sessions/{session_name}` |
| `get_components_v2` | `Client::get_components_v2` | GET `/v2/components` |
| `put_components_v2` | `Client::put_components_v2` | PUT `/v2/components` |
| `patch_components_v2` | `Client::patch_components_v2` | PATCH `/v2/components` |
| `get_component_v2` | `Client::get_component_v2` | GET `/v2/components/{component_id}` |
| `put_component_v2` | `Client::put_component_v2` | PUT `/v2/components/{component_id}` |
| `patch_component_v2` | `Client::patch_component_v2` | PATCH `/v2/components/{component_id}` |
| `delete_component_v2` | `Client::delete_component_v2` | DELETE `/v2/components/{component_id}` |
| `get_configurations_v2` | `Client::get_configurations_v2` | GET `/v2/configurations` |
| `get_configuration_v2` | `Client::get_configuration_v2` | GET `/v2/configurations/{configuration_id}` |
| `put_configuration_v2` | `Client::put_configuration_v2` | PUT `/v2/configurations/{configuration_id}` |
| `patch_configuration_v2` | `Client::patch_configuration_v2` | PATCH `/v2/configurations/{configuration_id}` |
| `delete_configuration_v2` | `Client::delete_configuration_v2` | DELETE `/v2/configurations/{configuration_id}` |
| `get_versions_v3` | `Client::get_versions_v3` | GET `/v3` |
| `get_options_v3` | `Client::get_options_v3` | GET `/v3/options` |
| `patch_options_v3` | `Client::patch_options_v3` | PATCH `/v3/options` |
| `get_sessions_v3` | `Client::get_sessions_v3` | GET `/v3/sessions` |
| `create_session_v3` | `Client::create_session_v3` | POST `/v3/sessions` |
| `delete_sessions_v3` | `Client::delete_sessions_v3` | DELETE `/v3/sessions` |
| `get_session_v3` | `Client::get_session_v3` | GET `/v3/sessions/{session_name}` |
| `patch_session_v3` | `Client::patch_session_v3` | PATCH `/v3/sessions/{session_name}` |
| `delete_session_v3` | `Client::delete_session_v3` | DELETE `/v3/sessions/{session_name}` |
| `get_components_v3` | `Client::get_components_v3` | GET `/v3/components` |
| `put_components_v3` | `Client::put_components_v3` | PUT `/v3/components` |
| `patch_components_v3` | `Client::patch_components_v3` | PATCH `/v3/components` |
| `get_component_v3` | `Client::get_component_v3` | GET `/v3/components/{component_id}` |
| `put_component_v3` | `Client::put_component_v3` | PUT `/v3/components/{component_id}` |
| `patch_component_v3` | `Client::patch_component_v3` | PATCH `/v3/components/{component_id}` |
| `delete_component_v3` | `Client::delete_component_v3` | DELETE `/v3/components/{component_id}` |
| `get_configurations_v3` | `Client::get_configurations_v3` | GET `/v3/configurations` |
| `get_configuration_v3` | `Client::get_configuration_v3` | GET `/v3/configurations/{configuration_id}` |
| `put_configuration_v3` | `Client::put_configuration_v3` | PUT `/v3/configurations/{configuration_id}` |
| `patch_configuration_v3` | `Client::patch_configuration_v3` | PATCH `/v3/configurations/{configuration_id}` |
| `delete_configuration_v3` | `Client::delete_configuration_v3` | DELETE `/v3/configurations/{configuration_id}` |
| `get_sources_v3` | `Client::get_sources_v3` | GET `/v3/sources` |
| `post_source_v3` | `Client::post_source_v3` | POST `/v3/sources` |
| `get_source_v3` | `Client::get_source_v3` | GET `/v3/sources/{source_id}` |
| `patch_source_v3` | `Client::patch_source_v3` | PATCH `/v3/sources/{source_id}` |
| `restore_source_v3` | `Client::restore_source_v3` | POST `/v3/sources/{source_id}` |
| `delete_source_v3` | `Client::delete_source_v3` | DELETE `/v3/sources/{source_id}` |

Notes:
- `post_source_v3` and `restore_source_v3` both bind to `POST` on
  `/v3/sources` and `/v3/sources/{source_id}` respectively — different
  paths, same verb. They are distinct generated methods.
- Path parameters like `{component_id}`, `{session_name}`, `{source_id}`
  produce newtype wrapper structs in `mod types` (e.g.
  `types::DeleteSourceV3SourceId(String)`) because their schemas have
  `pattern` constraints. Wrappers must build these via `TryFrom<&str>`
  rather than pass `&str` directly. This mirrors HSM's
  `XnameForeignKey`/`GroupLabel` newtypes.

## C. The `Error` enum

Defined in `progenitor-client 0.8.0` at
`progenitor_client/src/progenitor_client.rs:236`. It is `pub use`d through
the generated file.

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

Three different `E` parameters appear in the generated CFS code:

- `Error<()>` — operations where the spec's error responses define no
  schema (handful of cases, e.g. `get_versions_v2`).
- `Error<types::Healthz>` — `get_healthz` only. The spec uses the same
  `Healthz` schema for the 200 success and the 500/503 failure bodies, so
  progenitor emits `ResponseValue<Healthz>` for both branches.
- `Error<types::ProblemDetails>` — the majority. After the
  `application/problem+json` → `application/json` patch (Section F),
  every documented 4xx/5xx response returns the spec's `ProblemDetails`
  schema, and progenitor materialises it as a typed
  `Error::ErrorResponse(ResponseValue<ProblemDetails>)`.

Wrapper code unwrapping these `Error` values must be parameterised over
`E` or handle each variant.

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
    pub fn api_version(&self) -> &'static str { "0000.0000.0000" }
}
```

Implication for the CFS wrapper: `Client::new` bakes a 15-second
connect+request timeout. The wrapper's `gen_client()` helper MUST use
`Client::new_with_client` so the project's shared `reqwest::Client`
controls the timeout, rustls config, and insecure-TLS toggle. Calling
`Client::new` would silently override the rest of the codebase's
expectations.

Note: `api_version()` returns `"0000.0000.0000"` because the CFS spec's
`info.version` is literally `"0000.0000.0000"`. This is a placeholder
upstream uses; it carries no useful information.

## E. basePath behaviour

The CFS spec declares one server:

```yaml
servers:
  - url: https://api-gw-service-nmn.local/apis/cfs
    description: The production CFS API server
```

Progenitor does NOT auto-prepend `servers[0].url`. Every generated method
computes its URL as `format!("{}/<operation path>", self.baseurl)`,
where the operation path is what's literally written under `paths:` in
the spec (`/healthz`, `/v2/sessions`, `/v3/sources/{source_id}`, …).

This was verified by two wiremock unit tests in
`/tmp/progenitor-cfs-smoke/src/lib.rs`:

1. With `Client::new(&format!("{}/apis/cfs", server.uri()))`, a
   `get_healthz()` call hits `/apis/cfs/healthz` — mock matches.
2. With `Client::new(&server.uri())`, the same call hits `/healthz` —
   mock matches without the `/apis/cfs` prefix.

Both passed (`cargo test --lib` in the smoke crate: `test result: ok. 2 passed`).

Operational consequence: the CFS wrapper must build its base URL as
`format!("{}/cfs", shasta_client.base_url())` because csm-rs's
`base_url` already ends in `/apis` (e.g.
`https://api.cmn.alps.cscs.ch/apis`). The operation paths in the spec
already carry the `/v2/...` and `/v3/...` prefixes.

## F. Spec patches applied

Three patches were required to coax progenitor 0.8 into producing a
Rust-compilable generated.rs. The patches are now committed directly to
`src/cfs/csm_api_docs.yaml` (mirroring the HSM "patched JSON gets
committed" approach — Option A in the original task plan), with inline
comments at each patched region pointing back at this section. Task 1's
`build.rs` therefore consumes the already-patched YAML and needs no
programmatic spec rewriting.

Originally these patches lived only in the smoke crate's local spec
(`/tmp/progenitor-cfs-smoke/spec.yaml`); they were applied to the
committed spec in the same commit as this reference doc.

1. **Inline the 12 `V2Options` property refs into `V3Options/properties/*`
   (12 refs replaced).** progenitor's typify pass refuses to follow
   `$ref` values that point at a sub-key inside another schema's
   `properties` table — it only resolves refs to top-level schemas in
   `components.schemas`. The CFS spec uses 12 such property-level refs
   in `V2Options` to share definitions with `V3Options`. Each `$ref:
   '#/components/schemas/V3Options/properties/<field>'` was replaced
   with the inline scalar type definition from `V3Options.properties.<field>`.

   First failure observed without the patch:

       thread 'main' panicked at typify-impl-0.2.0/src/convert.rs:1285:32:
       $ref #/components/schemas/V3Options/properties/additional_inventory_url is missing

2. **Rewrite `application/problem+json` content entries to
   `application/json` (8 occurrences = 7 content-type keys + 1
   description-text occurrence).** progenitor's "extract response"
   step considers only the `application/json` media type for the typed
   return path; non-json media types fall through to
   `progenitor_client::ByteStream`. The CFS spec uses
   `application/problem+json` (RFC 7807) for the 7 error responses
   with body schemas (`BadRequest`, `ResourceNotFound`,
   `ConflictingSessionName`, `ConflictingSourceName`,
   `ForbiddenOperation`, `JobAlreadySet`, `KafkaTimeout`) — that's
   the 7 content-type keys. The 8th occurrence is the prose mention
   in the `info.description` text near line 111 of the spec, which
   also referenced `application/problem+json`. (`ResourceDeleted`
   is not in the list because the response has no `content` block at
   all — just a description — so there was nothing to patch.) With
   the original content-types in place, every method that referenced
   one of those error responses got
   `Result<…, Error<progenitor_client::ByteStream>>`, and because
   `ByteStream` does not implement `Debug`, the generated file failed
   to compile (51 instances of `error[E0277]: ByteStream doesn't
   implement Debug`).

3. **Inline the `TenantName` schema into the `Cray-Tenant-Name` header
   parameter as a plain `type: string` (1 parameter rewritten).** The
   `V3TenantHeaderParam` parameter references `components.schemas.TenantName`
   which is a constrained string (`maxLength: 127`). progenitor emits
   that as a newtype `pub struct TenantName(String);` — and then for
   each of the 15 operations that include the header, generates
   `header_map.append("Cray-Tenant-Name", HeaderValue::try_from(v)?)`,
   where `v: &TenantName`. `HeaderValue` has no `TryFrom<&TenantName>`
   impl and `TenantName` does not auto-deref into anything
   `HeaderValue` accepts, so the file fails to compile with 15
   `error[E0277]: HeaderValue: From<&PatchXxxV3CrayTenantName>` errors.

   The patched header inlines `schema: { type: string }` (with no
   length constraint, otherwise progenitor still emits an
   operation-specific newtype `PatchConfigurationV3CrayTenantName`).
   This is a hack acceptable for the smoke; the real fix is either a
   custom progenitor `replace` directive or a build-time spec rewrite.

After all three patches: `cargo build` succeeds with only 18 trivial
clippy warnings (all `value.len() < 0usize` dead-comparison warnings on
generated `min_length = 0` validators).

The smoke crate verifies these patches: copy `src/cfs/csm_api_docs.yaml`
(already patched) into `/tmp/progenitor-cfs-smoke/spec.yaml` and run
`cargo build` — it should compile with only 18 trivial dead-comparison
clippy warnings.

**Approach chosen for Task 1:** Option A (mirror HSM) — patches are
committed directly to `src/cfs/csm_api_docs.yaml`. Each patched region
carries an inline `# PATCH (csm-rs):` comment block citing this section
so a future engineer re-running `make convert-spec`-style flows knows
not to revert them.

Rejected alternatives:
- Option B (patch the YAML in `build.rs` before parsing into
  `openapiv3::OpenAPI`): cleaner for upstream tracking but adds runtime
  string-rewriting code to build.rs and makes the patches less visible
  in code review.
- Option C (use progenitor's `Generator::with_replace`): doesn't help
  with the V2Options property-ref panic, which fires before generator
  config takes effect.

## G. Generated artefact stats

- Total generated file: 13,589 lines (≈400 KB).
- Type count in `mod types`: 153 (124 structs + 29 enums; an `impl
  Deref/DerefMut/FromStr/TryFrom` block follows most newtypes).
- Method count on `impl Client`: 51 `pub async fn`s, exactly one per
  operationId — no synthesised methods (CFS spec has operationIds on
  every operation already).
- One `pub struct Client` at crate root, plus its inherent impls.

Compare to HSM: HSM generated 37,574 lines / 361 types / 132 methods.
CFS is roughly 1/3 the size, matching the plan's "smaller scale" note.

## H. Runtime crate dependencies

The generated CFS code references the following extern crates:

| Crate | Already in csm-rs Cargo.toml? | Action |
|---|---|---|
| `progenitor_client` | yes (added for HSM) | none |
| `regress` | yes (added for HSM) | none |
| `reqwest` | yes | none |
| `serde` | yes | none |
| `chrono` | yes, but `default-features = false, features = ["clock"]` | **MUST add `"serde"` feature**: generated code has 9 fields of type `chrono::DateTime<chrono::offset::Utc>` (session start/completion times, etc.). Without the `serde` feature, those fields fail to derive Serialize/Deserialize. |
| `uuid` | yes, but `features = ["fast-rng", "v4"]` (no `"serde"`) | **MUST add `"serde"` feature**: generated code uses `uuid::Uuid` for 2 fields — `image_id` and `result_id` on `SessionStatusArtifactsSectionItem` (generated.rs around line 2752). Without the `serde` feature, those `Option<uuid::Uuid>` fields fail to derive Serialize/Deserialize. |
| `serde_json` | yes | none |

Bottom line: two changes to csm-rs Cargo.toml are needed — add `"serde"`
to the `chrono` features list, and add `"serde"` to the `uuid` features
list. The HSM-added deps (`regress`, `progenitor-client`) cover
everything else.
