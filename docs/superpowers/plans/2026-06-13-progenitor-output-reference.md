# Progenitor output reference — `src/hsm/csm_api_docs.openapi3.json`

Generated against `progenitor` 0.8 (build via `progenitor::Generator::default()` in
`build.rs`). The smoke crate that produced this reference lived in
`/tmp/progenitor-smoke` and is not committed.

The spec fed to progenitor is the committed
`src/hsm/csm_api_docs.openapi3.json` which has been patched in three small
ways relative to the raw `swagger2openapi` output (see "Spec patches required"
at the bottom).

All generated types live in `mod types`. The client struct is `Client` at the
crate root.

## A. Generated type names

The table below lists the schemas referenced by Tasks 4–11 of the
implementation plan. The full inventory of generated types (361 total:
228 structs + 133 enums) is in the generated file; grep `pub (struct|enum) `
(egrep / `grep -E` syntax — the parens denote alternation) to enumerate them.

YAML schema | Rust path
----------- | ---------
`Group.1.0.0` | `types::Group100`
`Members.1.0.0` | `types::Members100`
`Membership.1.0.0` | `types::Membership100`
`Component.1.0.0_Component` | `types::Component100Component`
`Component.1.0.0_ComponentCreate` | `types::Component100ComponentCreate`
`Component.1.0.0_Put` | `types::Component100Put`
`ComponentArray_ComponentArray` | `types::ComponentArrayComponentArray`
`ComponentArray_PostArray` | `types::ComponentArrayPostArray`
`ComponentArray_PostQuery` | `types::ComponentArrayPostQuery`
`ComponentArray_PostByNIDQuery` | `types::ComponentArrayPostByNidQuery`
`RedfishEndpoint.1.0.0_RedfishEndpoint` | `types::RedfishEndpoint100RedfishEndpoint`
`RedfishEndpointArray_RedfishEndpointArray` | `types::RedfishEndpointArrayRedfishEndpointArray`
`CompEthInterface.1.0.0` | `types::CompEthInterface100`
`CompEthInterface.1.0.0_Patch` | `types::CompEthInterface100Patch`
`CompEthInterface.1.0.0_IPAddressMapping` | `types::CompEthInterface100IpAddressMapping`
`HMSRole.1.0.0` | `types::HmsRole100` (newtype `pub struct HmsRole100(pub String);`)
`HMSState.1.0.0` | `types::HmsState100` (enum)
`HMSType.1.0.0` | `types::HmsType100` (enum)
`HMSFlag.1.0.0` | `types::HmsFlag100` (enum)
`HMSSubRole.1.0.0` | `types::HmsSubRole100` (newtype `pub struct HmsSubRole100(pub String);`)
`HMSArch.1.0.0` | `types::HmsArch100` (enum)
`HMSClass.1.0.0` | `types::HmsClass100` (enum)
`NetType.1.0.0` | `types::NetType100` (enum)
`HWInventory.1.0.0_HWInventoryByLocation` | `types::HwInventory100HwInventoryByLocation`
`HWInventory.1.0.0_HWInventoryByFRU` | `types::HwInventory100HwInventoryByFru`
`HWInventory.1.0.0_HWInventory` | `types::HwInventory100HwInventory`
`ResourceURI.1.0.0` | `types::ResourceUri100`

Notes on the mangling:
- Progenitor delegates type naming to `typify`, which lowercases the
  acronyms (`HMS` → `Hms`, `HW` → `Hw`, `URI` → `Uri`, `IP` → `Ip`, `FRU`
  → `Fru`, `NID` → `Nid`). Plan code that referenced `HMSState100` must
  use `HmsState100` etc.
- The `.1.0.0` version suffix collapses to `100` (dots stripped). e.g.
  `Component.1.0.0_Component` → `Component100Component`. Both
  dot-separated (`.1.0.0`) and underscore-separated (`_1_0_0`) version
  suffixes collapse identically.
- Composite names like `ComponentArray_PostByNIDQuery` keep their inner
  parts but mangle each (`ComponentArrayPostByNidQuery`).

## Schemas referenced by the plan that are NOT generated (or appear under a different name)

The implementation plan for Tasks 9, 10, and 11 mentions a wrapper return
type named `HsmActionResponse`. No schema by that name exists in
`src/hsm/csm_api_docs.openapi3.json` (verified by `grep -i HsmActionResponse`
on both the YAML and JSON specs), so progenitor does not emit a
`types::HsmActionResponse`. The endpoints those tasks touch actually return
the schema `Response.1.0.0` (mangled to `types::Response100`), which is a
small two-field struct:

```rust
pub struct Response100 {
    pub code: String,
    pub message: String,
}
```

Use the table below to substitute the real generated return type into
wrapper code:

Endpoint | Generated method | Actual response type
-------- | ---------------- | --------------------
`POST /Inventory/Hardware` | `Client::do_hw_inv_by_location_post` | `types::Response100`
`POST /Inventory/RedfishEndpoints` | `Client::do_redfish_endpoints_post` | `Vec<types::ResourceUri100>` (NOT `Response100` — returns the array of created resource URIs)
`PUT /Inventory/RedfishEndpoints/{xname}` | `Client::do_redfish_endpoint_put` | `types::RedfishEndpoint100RedfishEndpoint` (the updated endpoint object, NOT `Response100`)
`DELETE /Inventory/RedfishEndpoints/{xname}` | `Client::do_redfish_endpoint_delete` | `types::Response100`
`POST /Inventory/EthernetInterfaces/{id}/IPAddresses` | `Client::do_comp_eth_interface_ip_addresses_post_v2` | `types::ResourceUri100` (a single URI, NOT `Response100`)

`POST /Inventory/Hardware` and `DELETE /Inventory/RedfishEndpoints/{xname}`
share the same small `Response100` shape. The other three endpoints each
return a different concrete type and must be wrapped accordingly. There
is no single sentinel response shared by all five; wrapper code in Tasks
9/10/11 should drop the `HsmActionResponse` name and use the per-endpoint
generated types listed above.

## B. Generated method names

Section B lists every operationId referenced by Tasks 4–11 of the
implementation plan. For the full inventory of generated methods (132 in
total at time of writing), grep `pub async fn ` in the generated file at
`/tmp/progenitor-smoke/target/debug/build/progenitor_smoke-*/out/generated.rs`
or `operationId:` in `src/hsm/csm_api_docs.openapi3.json`.

Each method is `Client::<name>` (`impl Client { pub async fn <name>(...) }`).
operationId values come from `src/hsm/csm_api_docs.yaml` (Swagger 2.0
source). Where the table shows `(synthesized)`, the YAML lacked an
operationId on that path; the spec-patching script under
"Spec patches required" added one.

operationId (YAML) | Rust method | Verb + path
------------------ | ----------- | -----------
`doGroupsGet` | `Client::do_groups_get` | GET `/groups`
`doGroupGet` | `Client::do_group_get` | GET `/groups/{group_label}`
`doGroupsPost` | `Client::do_groups_post` | POST `/groups`
`doGroupDelete` | `Client::do_group_delete` | DELETE `/groups/{group_label}`
`doGroupPatch` | `Client::do_group_patch` | PATCH `/groups/{group_label}`
`doGroupMembersGet` | `Client::do_group_members_get` | GET `/groups/{group_label}/members`
`doGroupMembersPost` | `Client::do_group_members_post` | POST `/groups/{group_label}/members`
`doGroupMemberDelete` | `Client::do_group_member_delete` | DELETE `/groups/{group_label}/members/{xname_id}`
`doMembershipsGet` | `Client::do_memberships_get` | GET `/memberships`
`doMembershipGet` | `Client::do_membership_get` | GET `/memberships/{xname}`
`doComponentsGet` | `Client::do_components_get` | GET `/State/Components`
`doComponentGet` | `Client::do_component_get` | GET `/State/Components/{xname}`
`doComponentsPost` | `Client::do_components_post` | POST `/State/Components`
`doComponentPut` | `Client::do_component_put` | PUT `/State/Components/{xname}`
`doComponentDelete` | `Client::do_component_delete` | DELETE `/State/Components/{xname}`
`doComponentsDeleteAll` | `Client::do_components_delete_all` | DELETE `/State/Components`
`doComponentsQueryPost` | `Client::do_components_query_post` | POST `/State/Components/Query`
`doComponentByNIDGet` | `Client::do_component_by_nid_get` | GET `/State/Components/ByNID/{nid}`
`doComponentByNIDQueryPost` | `Client::do_component_by_nid_query_post` | POST `/State/Components/ByNID/Query`
`doHWInvByLocationGetAll` | `Client::do_hw_inv_by_location_get_all` | GET `/Inventory/Hardware`
`doHWInvByLocationGet` | `Client::do_hw_inv_by_location_get` | GET `/Inventory/Hardware/{xname}`
`doHWInvByLocationPost` | `Client::do_hw_inv_by_location_post` | POST `/Inventory/Hardware`
`doHWInvByLocationDelete` | `Client::do_hw_inv_by_location_delete` | DELETE `/Inventory/Hardware/{xname}`
`doHWInvByLocationDeleteAll` | `Client::do_hw_inv_by_location_delete_all` | DELETE `/Inventory/Hardware`
`doHWInvByFRUGetAll` | `Client::do_hw_inv_by_fru_get_all` | GET `/Inventory/HardwareByFRU`
`doHWInvByFRUGet` | `Client::do_hw_inv_by_fru_get` | GET `/Inventory/HardwareByFRU/{fruid}`
`doHWInvByFRUDelete` | `Client::do_hw_inv_by_fru_delete` | DELETE `/Inventory/HardwareByFRU/{fruid}`
`doHWInvByFRUDeleteAll` | `Client::do_hw_inv_by_fru_delete_all` | DELETE `/Inventory/HardwareByFRU`
`doRedfishEndpointsGet` | `Client::do_redfish_endpoints_get` | GET `/Inventory/RedfishEndpoints`
`doRedfishEndpointGet` | `Client::do_redfish_endpoint_get` | GET `/Inventory/RedfishEndpoints/{xname}`
`doRedfishEndpointQueryGet` | `Client::do_redfish_endpoint_query_get` | GET `/Inventory/RedfishEndpoints/Query/{xname}`
`doRedfishEndpointsPost` | `Client::do_redfish_endpoints_post` | POST `/Inventory/RedfishEndpoints`
`doRedfishEndpointsDeleteAll` | `Client::do_redfish_endpoints_delete_all` | DELETE `/Inventory/RedfishEndpoints`
`doRedfishEndpointPut` | `Client::do_redfish_endpoint_put` | PUT `/Inventory/RedfishEndpoints/{xname}`
`doRedfishEndpointDelete` | `Client::do_redfish_endpoint_delete` | DELETE `/Inventory/RedfishEndpoints/{xname}`
`doRedfishEndpointPatch` | `Client::do_redfish_endpoint_patch` | PATCH `/Inventory/RedfishEndpoints/{xname}`
`doCompEthInterfacesGetV2` | `Client::do_comp_eth_interfaces_get_v2` | GET `/Inventory/EthernetInterfaces`
`doCompEthInterfaceGetV2` | `Client::do_comp_eth_interface_get_v2` | GET `/Inventory/EthernetInterfaces/{ethInterfaceID}`
`doCompEthInterfacePostV2` | `Client::do_comp_eth_interface_post_v2` | POST `/Inventory/EthernetInterfaces`
`doCompEthInterfacePatchV2` | `Client::do_comp_eth_interface_patch_v2` | PATCH `/Inventory/EthernetInterfaces/{ethInterfaceID}`
`doCompEthInterfaceDeleteV2` | `Client::do_comp_eth_interface_delete_v2` | DELETE `/Inventory/EthernetInterfaces/{ethInterfaceID}`
`doCompEthInterfaceDeleteAllV2` | `Client::do_comp_eth_interface_delete_all_v2` | DELETE `/Inventory/EthernetInterfaces`
`doCompEthInterfaceIPAddressesGetV2` | `Client::do_comp_eth_interface_ip_addresses_get_v2` | GET `/Inventory/EthernetInterfaces/{ethInterfaceID}/IPAddresses`
`doCompEthInterfaceIPAddressesPostV2` | `Client::do_comp_eth_interface_ip_addresses_post_v2` | POST `/Inventory/EthernetInterfaces/{ethInterfaceID}/IPAddresses`
`doCompEthInterfaceIPAddressDeleteV2` | `Client::do_comp_eth_interface_ip_address_delete_v2` | DELETE `/Inventory/EthernetInterfaces/{ethInterfaceID}/IPAddresses/{ipAddress}`
`doCompEthInterfaceIPAddressPatchV2` | `Client::do_comp_eth_interface_ip_address_patch_v2` | PATCH `/Inventory/EthernetInterfaces/{ethInterfaceID}/IPAddresses/{ipAddress}`
`doValuesGet` | `Client::do_values_get` | GET `/service/values`
`doRoleValuesGet` | `Client::do_role_values_get` | GET `/service/values/role`
`doArchValuesGet` | `Client::do_arch_values_get` | GET `/service/values/arch`
`doClassValuesGet` | `Client::do_class_values_get` | GET `/service/values/class`
`doStateValuesGet` | `Client::do_state_values_get` | GET `/service/values/state`
`doTypeValuesGet` | `Client::do_type_values_get` | GET `/service/values/type`
`doFlagValuesGet` | `Client::do_flag_values_get` | GET `/service/values/flag`
`doNetTypeValuesGet` | `Client::do_net_type_values_get` | GET `/service/values/nettype`
`doSubRoleValuesGet` | `Client::do_sub_role_values_get` | GET `/service/values/subrole`

The plan's task list mentioned `doRoleGet` as a "GET /service/values/role" stand-in;
the real operationId is `doRoleValuesGet` (above). Likewise the plan
guessed `doHWInvByLocationGet (or equivalent)`; the real spec exposes both
`doHWInvByLocationGetAll` (collection) and `doHWInvByLocationGet` (single
xname), matching `/Inventory/Hardware` and `/Inventory/Hardware/{xname}`.

Notes on the mangling:
- Operation IDs are `CamelCase` in the spec and become `snake_case` Rust
  identifiers (`typify::sanitize(..., Case::Snake)`). Embedded acronyms
  do NOT preserve casing — e.g. `doHWInvByLocationGet` becomes
  `do_hw_inv_by_location_get` (not `do_h_w_inv_...`). `IPAddress` becomes
  `ip_address`. `V2` stays as `v2`.

## C. The `Error` enum

This enum is defined in `progenitor-client 0.8.0` at
`progenitor_client/src/progenitor_client.rs:236`. It is `pub use`d
through the generated file. Every generated method returns
`Result<ResponseValue<T>, Error<()>>` (the `()` because we stripped
`default` error responses — see Section F).

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

Important: with `E = ()`, an HTTP 4xx/5xx that returns a body becomes
`Error::UnexpectedResponse(reqwest::Response)` (NOT `Error::ErrorResponse`).
Downstream code that wants the response body (e.g. to surface a CSM
`Problem7807` payload) must read the `reqwest::Response` itself.

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
    pub fn api_version(&self) -> &'static str { "1.0.0" }
}
```

Implication for our HSM wrapper: `Client::new` bakes in a hard-coded 15s
connect + request timeout (see the `from_secs(15)` lines above). The
wrapper's `gen_client()` helper MUST use `Client::new_with_client` (not
`Client::new`) so the project's shared `reqwest::Client` controls the
timeout — along with rustls config, the insecure-TLS toggle, and any
other settings. Calling `Client::new` would silently override whatever
timeout the rest of the codebase expects.

## E. basePath behaviour

The spec declares one server:

```yaml
servers:
  - url: "https://sms/apis/smd/hsm/v2"
```

Progenitor does NOT auto-prepend `servers[0].url`. Every generated
method computes its URL as `format!("{}/<operation path>", self.baseurl)`,
where the operation path is what's literally written under `paths:` in
the spec (`/groups`, `/State/Components`, `/Inventory/RedfishEndpoints`,
…). This was verified by two wiremock unit tests:

1. With `Client::new(&format!("{}/apis/smd/hsm/v2", server.uri()))`, a
   `do_groups_get` call hits `/apis/smd/hsm/v2/groups` — mock matches.
2. With `Client::new(&server.uri())`, the same call hits `/groups` —
   mock matches without the `apis/smd/hsm/v2` prefix.

Operational consequence: the HSM wrapper around `Client` must build its
base URL as `format!("{}/apis/smd/hsm/v2", shasta_host)`. The existing
hand-written client uses the same convention (see
`src/hsm/component/http_client.rs` — the current file is flat, not split
into a `v2/mod.rs`).

## F. Spec patches required

`swagger2openapi` alone does NOT produce a spec progenitor 0.8 accepts.
Three patches were applied to `src/hsm/csm_api_docs.openapi3.json` after
the YAML→JSON conversion. The patched file is what's committed. The
patching script is reproducible from this document.

1. **Synthesise operationIds on `/locks/*` paths (13 ops).** Progenitor
   refuses any operation without an `operationId`. The original YAML
   leaves them off on the lock endpoints. Synthesised names follow the
   pattern `do<PathParts><Verb>` (e.g. `POST /locks/lock` →
   `doLocksLockPost`). These ops are out of scope for the HSM wrapper but
   their presence does not hurt us — they're just extra methods on
   `Client`.

2. **Drop `application/problem+json` content entries (446 entries
   touched).** Every response has both `application/json` and
   `application/problem+json` with the same schema. Progenitor only
   wires `application/json` into the typed return path, but its
   `extract_responses` step asserts a single content type when picking a
   success body. Removing the `problem+json` clones avoids the assert.

3. **Strip non-2xx (and `default`) responses (348 entries total: 229
   non-2xx + 119 `default`).** Progenitor's success-extractor groups
   together 2xx + default responses and asserts that the resulting set
   of types has exactly one element. The spec uses `Problem7807` as the
   schema for `default`/4xx/5xx responses, which differs from the 2xx
   schema, tripping this assertion on most operations. We drop the error
   responses entirely; downstream callers handle non-2xx as
   `Error::UnexpectedResponse(reqwest::Response)` (Section C) and read
   the body manually if they need the Problem details. Once we have a
   real client in place, we can selectively reintroduce typed error
   responses by patching the JSON further, but it's not required for
   Phase 1.

The patching is implemented as small Python scripts that read and
rewrite the JSON in place. They are not committed; the resulting JSON
is. Re-deriving the JSON is:

```
npx --yes swagger2openapi src/hsm/csm_api_docs.yaml -o src/hsm/csm_api_docs.openapi3.json
# then apply the three patches above
```

## G. Smoke-build outcome

- `cargo build` against the patched spec compiles cleanly (37,574 lines
  of generated code; 1 `pub struct Client`, 361 type definitions in
  `mod types` (228 structs + 133 enums), 132 `pub async fn` methods on
  `Client`).
- Both wiremock unit tests pass (`cargo test --lib`).
- Doctests fail because progenitor's generated rustdoc copies the CSM
  description literally; some of those descriptions contain text the
  doctest harness tries to parse as Rust (`Prior State -> New State`).
  This is irrelevant for the build crate that will consume the generated
  output — we only `include!` it, not run its doctests.

## H. Runtime dependency we added

Progenitor's typify-emitted format validators reference
`regress::Regex`, so the consumer crate (the smoke and ultimately the
hsm build) must declare `regress = "0.10"` as a runtime dep. The
`Cargo.toml` shape used in the smoke is the recommended starting point
for the real HSM build target:

```toml
[dependencies]
progenitor-client = "0.8"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["serde", "v4"] }
regress = "0.10"

[build-dependencies]
progenitor = "0.8"
serde_json = "1"
prettyplease = "0.2"
syn = "2"
openapiv3 = "2"
```

Note also `openapiv3 = "2"` in build-dependencies: progenitor 0.8's
`Generator::generate_tokens` takes `&openapiv3::OpenAPI`, not
`&serde_json::Value`. The build script must parse the JSON into the
typed struct first.
