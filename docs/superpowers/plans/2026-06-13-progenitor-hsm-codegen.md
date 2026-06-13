# progenitor-based HSM client codegen — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hand-written `src/hsm/` HTTP client and wire-format types with code generated from `src/hsm/csm_api_docs.yaml`, preserving the existing public `ShastaClient` API.

**Architecture:** Two-step pipeline. (1) Developer step: convert Swagger 2.0 YAML → OpenAPI 3.0 JSON via `swagger2openapi` and commit the JSON. (2) Build step: `build.rs` runs `progenitor::Generator` on the JSON; output is `include!`-d into a `pub(crate)` module. A thin wrapper layer maps each existing `hsm_*` method on `ShastaClient` to a generated client call. Hand-rolled wire types are deleted and `pub use`-aliased to generated types; projection types like `NodeSummary` move into the wrapper layer.

**Tech Stack:** Rust (edition 2021), `progenitor ~ 0.8`, `reqwest 0.12`, `serde 1`, `tokio 1.45`, `swagger2openapi` (npm CLI; developer-only).

**Source-of-truth spec:** `docs/superpowers/specs/2026-06-13-progenitor-hsm-codegen-design.md`. Read it before starting.

**Type/method name reference (filled in by Task 0):** `docs/superpowers/plans/2026-06-13-progenitor-output-reference.md`. Subsequent migration tasks consult this file because progenitor's exact mangled type/method names are not knowable until the generator runs on the converted spec.

---

## Phase 0: Feasibility verification (one task, gates everything else)

### Task 0: Run progenitor end-to-end against the converted spec; capture the name mapping

**Why this exists:** progenitor only accepts OpenAPI 3.x; `csm_api_docs.yaml` is Swagger 2.0. The conversion may fail or produce a spec progenitor rejects. The exact Rust type/method names progenitor emits are determined by its mangling rules + the spec's `operationId` and `definitions:` keys. Nothing in Phase 1+ can be concrete until we have the converted JSON in hand and have inspected the generated `.rs` file.

**Files:**
- Create: `src/hsm/csm_api_docs.openapi3.json` (committed)
- Create: `docs/superpowers/plans/2026-06-13-progenitor-output-reference.md` (committed)
- Temporary: `/tmp/progenitor-smoke/` (scratch crate; not committed)

- [ ] **Step 1: Convert the spec from Swagger 2.0 to OpenAPI 3.0**

Run:
```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
npx --yes swagger2openapi src/hsm/csm_api_docs.yaml \
    -o src/hsm/csm_api_docs.openapi3.json
```
Expected output: a non-zero-byte JSON file at `src/hsm/csm_api_docs.openapi3.json`. The CLI may print warnings about deprecated fields — that's fine. Non-zero exit code = STOP and read the error.

- [ ] **Step 2: Validate it's parseable OpenAPI 3.0 JSON**

Run:
```bash
python3 -c "import json; d=json.load(open('src/hsm/csm_api_docs.openapi3.json')); print(d['openapi']); print('paths:', len(d['paths'])); print('schemas:', len(d['components']['schemas']))"
```
Expected output: a line like `3.0.0` followed by `paths: ~70` and `schemas: ~200`. If `openapi` field is missing or the JSON does not parse, STOP — the conversion is broken.

- [ ] **Step 3: Scaffold a throwaway crate to test progenitor**

Run:
```bash
mkdir -p /tmp/progenitor-smoke && cd /tmp/progenitor-smoke && cargo init --lib --name progenitor_smoke
cp /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs/src/hsm/csm_api_docs.openapi3.json ./spec.json
```

Edit `/tmp/progenitor-smoke/Cargo.toml` to be:
```toml
[package]
name = "progenitor_smoke"
version = "0.0.0"
edition = "2021"

[dependencies]
progenitor-client = "0.8"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["serde", "v4"] }

[build-dependencies]
progenitor = "0.8"
serde_json = "1"
prettyplease = "0.2"
syn = "2"
```

- [ ] **Step 4: Add a minimal build.rs for the smoke crate**

Create `/tmp/progenitor-smoke/build.rs`:
```rust
use std::{env, fs, path::PathBuf};

fn main() {
    let spec_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("spec.json");
    println!("cargo:rerun-if-changed={}", spec_path.display());

    let src = fs::read_to_string(&spec_path).expect("read spec");
    let spec: serde_json::Value = serde_json::from_str(&src).expect("parse spec");

    let mut generator = progenitor::Generator::default();
    let tokens = generator.generate_tokens(&spec).expect("progenitor codegen");
    let ast: syn::File = syn::parse2(tokens).expect("parse generated tokens");
    let pretty = prettyplease::unparse(&ast);

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("generated.rs");
    fs::write(&out, pretty).expect("write generated.rs");
}
```

Replace the contents of `/tmp/progenitor-smoke/src/lib.rs`:
```rust
#![allow(dead_code, clippy::all)]
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
```

- [ ] **Step 5: Run the smoke build**

Run:
```bash
cd /tmp/progenitor-smoke && cargo build 2>&1 | tail -60
```
Expected: `cargo build` succeeds. If it fails, read the error. Two known failure modes:
  - **`progenitor::Generator::generate_tokens` panics on the spec** — patch the JSON (see Step 6 fallback).
  - **`syn::parse2` fails on the generated tokens** — same; spec issue surfacing through codegen.

If the build fails after up to a day of YAML/JSON patching, STOP this plan and escalate (fallback options in the design spec § Risks).

- [ ] **Step 6: Inspect the generated code and capture the type/method name reference**

Locate the generated file:
```bash
find /tmp/progenitor-smoke/target -name generated.rs -path '*build*' | head -1
```
Open it. The file is a single large Rust file with `pub struct Client { ... }`, a `pub mod types { ... }` with all schema types, and per-operation `impl Client { pub async fn ... }` methods.

Create `docs/superpowers/plans/2026-06-13-progenitor-output-reference.md` in csm-rs with this exact structure (fill in the right-hand side by reading the generated file):

```markdown
# progenitor output reference for HSM

Captured by Task 0 of the implementation plan. Updated when the YAML changes.

## Generated type names (by YAML schema name)

| YAML schema | Generated Rust type (`generated::types::…`) |
|---|---|
| Group.1.0.0 | …fill in… |
| Members.1.0.0 | …fill in… |
| Membership.1.0.0 | …fill in… |
| Component.1.0.0_Component | …fill in… |
| Component.1.0.0_ComponentCreate | …fill in… |
| Component.1.0.0_Put | …fill in… |
| ComponentArray_ComponentArray | …fill in… |
| ComponentArray_PostArray | …fill in… |
| ComponentArray_PostQuery | …fill in… |
| ComponentArray_PostByNIDQuery | …fill in… |
| RedfishEndpoint.1.0.0_RedfishEndpoint | …fill in… |
| RedfishEndpointArray_RedfishEndpointArray | …fill in… |
| CompEthInterface.1.0.0 | …fill in… |
| CompEthInterface.1.0.0_Patch | …fill in… |
| CompEthInterface.1.0.0_IPAddressMapping | …fill in… |
| HMSRole.1.0.0 | …fill in… |
| HMSState.1.0.0 | …fill in… |
| HMSType.1.0.0 | …fill in… |
| HMSFlag.1.0.0 | …fill in… |
| HMSSubRole.1.0.0 | …fill in… |
| HMSArch.1.0.0 | …fill in… |
| HMSClass.1.0.0 | …fill in… |
| NetType.1.0.0 | …fill in… |
| HWInventory.1.0.0_HWInventoryByLocation | …fill in… |
| HWInventory.1.0.0_HWInventoryByFRU | …fill in… |
| HWInventory.1.0.0_HWInventory | …fill in… |
| ResourceURI.1.0.0 | …fill in… |
| HsmActionResponse (if present; else note "not in spec") | …fill in… |

## Generated method names (by operationId)

| YAML operationId | Generated method (`generated::Client::…`) | HTTP verb + path |
|---|---|---|
| doGroupsGet | …fill in… | GET /groups |
| doGroupGet | …fill in… | GET /groups/{group_label} |
| doGroupsPost | …fill in… | POST /groups |
| doGroupDelete | …fill in… | DELETE /groups/{group_label} |
| doGroupMembersPost | …fill in… | POST /groups/{group_label}/members |
| doGroupMemberDelete | …fill in… | DELETE /groups/{group_label}/members/{xname_id} |
| doMembershipsGet | …fill in… | GET /memberships |
| doMembershipGet | …fill in… | GET /memberships/{xname} |
| doComponentsGet | …fill in… | GET /State/Components |
| doComponentGet | …fill in… | GET /State/Components/{xname} |
| doComponentsPost | …fill in… | POST /State/Components |
| doComponentPut | …fill in… | PUT /State/Components/{xname} |
| doComponentDelete | …fill in… | DELETE /State/Components/{xname} |
| doComponentsDeleteAll | …fill in… | DELETE /State/Components |
| doComponentsPostByNIDQuery | …fill in… | POST /State/Components/ByNID/Query |
| doHWInvByLocationGet | …fill in… | GET /Inventory/Hardware |
| doHWInvByLocationGetOne (xname query) | …fill in… | GET /Inventory/Hardware/Query/{xname} |
| doHWInvByLocationPost | …fill in… | POST /Inventory/Hardware |
| doRedfishEndpointsGet | …fill in… | GET /Inventory/RedfishEndpoints |
| doRedfishEndpointQueryGet | …fill in… | GET /Inventory/RedfishEndpoints/Query/{xname} |
| doRedfishEndpointPost | …fill in… | POST /Inventory/RedfishEndpoints |
| doRedfishEndpointPut | …fill in… | PUT /Inventory/RedfishEndpoints/{xname} |
| doRedfishEndpointDelete | …fill in… | DELETE /Inventory/RedfishEndpoints/{xname} |
| doCompEthInterfacesGetV2 | …fill in… | GET /Inventory/EthernetInterfaces |
| doCompEthInterfaceGetV2 | …fill in… | GET /Inventory/EthernetInterfaces/{id} |
| doCompEthInterfacePostV2 | …fill in… | POST /Inventory/EthernetInterfaces |
| doCompEthInterfacePatchV2 | …fill in… | PATCH /Inventory/EthernetInterfaces/{id} |
| doCompEthInterfaceIPAddressPostV2 | …fill in… | POST /Inventory/EthernetInterfaces/{id}/IPAddresses |
| doRoleGet | …fill in… | GET /service/values/role |

## progenitor `Error` enum variant list (paste from generated file)

…fill in the exact enum variants progenitor emitted; mapping in `wrapper/mod.rs::map_err` depends on this…

## Client constructor signature

…copy the exact signature of `Client::new` / `Client::new_with_client` from the generated file…
```

If a YAML schema or operationId in the table above is missing from the generated file, write "NOT GENERATED" in the right column and note in the migration task it appears in. (Missing operationId → progenitor may have synthesised a name; missing schema → likely an `additionalProperties: true` case that became `serde_json::Value`.)

- [ ] **Step 7: Run the URL/auth sanity check**

Add this test to `/tmp/progenitor-smoke/src/lib.rs` (look up the generated `Client::new` signature and the `do_role_get`-equivalent method name from the reference file you just wrote):

```rust
#[cfg(test)]
mod sanity {
    use super::*;

    #[tokio::test]
    async fn role_endpoint_url_matches_csm_expectations() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/service/values/role"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({"Role":[]})))
            .mount(&server)
            .await;

        let http = reqwest::Client::new();
        // <<< Replace `Client::new` and `do_role_get` with the actual generated names. >>>
        let client = Client::new_with_client(&server.uri(), http);
        let _ = client.do_role_get().await.expect("call");
        // Sanity: one matching request was received.
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}
```

Add to `/tmp/progenitor-smoke/Cargo.toml`:
```toml
[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt"] }
wiremock = "0.6"
```

Run:
```bash
cd /tmp/progenitor-smoke && cargo test sanity -- --nocapture
```
Expected: `1 passed`. This proves progenitor's client constructor + a method call hit a real HTTP server with the right path. If it fails because of wrong basePath behaviour (`/apis/smd/hsm/v2/service/values/role` instead of `/service/values/role`), record that in the reference file under a new "## basePath behaviour" section — Phase 1 needs it.

- [ ] **Step 8: Commit the converted spec and the reference document**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
git add src/hsm/csm_api_docs.openapi3.json docs/superpowers/plans/2026-06-13-progenitor-output-reference.md
git commit -m "$(printf 'chore(hsm): commit converted OpenAPI 3.0 spec + progenitor output reference\n\nCaptured by Task 0 of the progenitor HSM codegen plan.')"
```

---

## Phase 1: Build infrastructure

### Task 1: Add progenitor dependencies + Makefile target

**Files:**
- Modify: `Cargo.toml`
- Create: `Makefile`

- [ ] **Step 1: Read current Cargo.toml dependency layout**

Run:
```bash
sed -n '60,120p' Cargo.toml
```
Note the existing `[dependencies]` and any `[build-dependencies]` block (likely absent).

- [ ] **Step 2: Add progenitor-client to `[dependencies]`**

Edit `Cargo.toml`. Find the line beginning `reqwest = ` (around line 78). After the runtime deps in that block, add:
```toml
progenitor-client = "0.8"
```

- [ ] **Step 3: Add a `[build-dependencies]` block at the bottom of `Cargo.toml`**

Append after the last existing dependency entry, before `[features]` if one exists:
```toml
[build-dependencies]
progenitor = "0.8"
serde_json = "1"
prettyplease = "0.2"
syn = "2"
```

- [ ] **Step 4: Create the convert-spec Makefile**

Create `Makefile` at repo root:
```makefile
.PHONY: convert-spec

# Convert Swagger 2.0 YAML to OpenAPI 3.0 JSON for progenitor consumption.
# Re-run whenever src/hsm/csm_api_docs.yaml changes. The JSON is committed.
convert-spec:
	npx --yes swagger2openapi src/hsm/csm_api_docs.yaml \
		-o src/hsm/csm_api_docs.openapi3.json
```

- [ ] **Step 5: Verify nothing broke**

Run:
```bash
cargo check 2>&1 | tail -20
```
Expected: `Checking csm-rs …` and `Finished`. New dependencies download but no compile errors yet (no code references them).

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock Makefile
git commit -m "chore(hsm): add progenitor build dependencies + convert-spec target"
```

---

### Task 2: Create `build.rs`

**Files:**
- Create: `build.rs` (at repo root)

- [ ] **Step 1: Write `build.rs`**

Create `build.rs` at repo root with this exact content:
```rust
//! Build-time codegen of the HSM HTTP client from the committed
//! OpenAPI 3.0 spec at `src/hsm/csm_api_docs.openapi3.json`.
//!
//! Output is `$OUT_DIR/hsm_generated.rs`, included from
//! `src/hsm/generated.rs`. Re-runs when the JSON changes.

use std::{env, fs, path::PathBuf};

fn main() {
    let spec_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src/hsm/csm_api_docs.openapi3.json");
    println!("cargo:rerun-if-changed={}", spec_path.display());
    println!("cargo:rerun-if-changed=build.rs");

    let src = fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", spec_path.display()));
    let spec: serde_json::Value =
        serde_json::from_str(&src).expect("csm_api_docs.openapi3.json is not valid JSON");

    let mut generator = progenitor::Generator::default();
    let tokens = generator
        .generate_tokens(&spec)
        .expect("progenitor codegen failed; re-run `make convert-spec` and check the JSON");
    let ast: syn::File = syn::parse2(tokens).expect("generated tokens do not parse");
    let pretty = prettyplease::unparse(&ast);

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("hsm_generated.rs");
    fs::write(&out, pretty).unwrap_or_else(|e| panic!("write {}: {e}", out.display()));
}
```

- [ ] **Step 2: Run `cargo build` to exercise the build script**

Run:
```bash
cargo build 2>&1 | tail -40
```
Expected: `Compiling csm-rs (build script)` then `Compiling csm-rs` then `Finished`. The build script runs; no compile errors in library code yet (the generated file is not yet included).

If the build script panics, the error message will tell you which `expect` fired. Match against the failure modes in Task 0 Step 5.

- [ ] **Step 3: Verify the generated file actually got produced**

Run:
```bash
find target -name hsm_generated.rs | head -1
```
Expected: one line, e.g. `target/debug/build/csm-rs-xxxx/out/hsm_generated.rs`. If no output, the build script didn't write the file — recheck Step 1.

- [ ] **Step 4: Commit**

```bash
git add build.rs
git commit -m "feat(hsm): add build.rs that runs progenitor on the OpenAPI 3.0 spec"
```

---

### Task 3: Create `src/hsm/generated.rs` and the wrapper module skeleton

**Files:**
- Create: `src/hsm/generated.rs`
- Create: `src/hsm/wrapper/mod.rs`
- Modify: `src/hsm/mod.rs`

- [ ] **Step 1: Create `src/hsm/generated.rs`**

```rust
//! progenitor-generated HSM client. Output of `build.rs`, included verbatim.
//!
//! `pub(crate)` because only the wrapper layer in `crate::hsm::wrapper`
//! and per-resource `types.rs` re-export aliases are allowed to touch
//! the generated symbols. Public consumers go through `ShastaClient`.
#![allow(dead_code, clippy::all, missing_docs, non_camel_case_types, non_snake_case, unused_imports)]
include!(concat!(env!("OUT_DIR"), "/hsm_generated.rs"));
```

- [ ] **Step 2: Create `src/hsm/wrapper/mod.rs`**

Open `docs/superpowers/plans/2026-06-13-progenitor-output-reference.md` and look up:
  (a) the exact constructor signature for the generated `Client`,
  (b) the exact variant names of progenitor's `Error` enum.

Replace `__CLIENT_NEW__` and the error variant arms below with the captured names.

```rust
//! Thin wrapper bridging the generated HSM client to the public
//! `ShastaClient` API. Responsibilities:
//!  - construct a per-call generated `Client` with bearer auth baked in;
//!  - override the spec's basePath with the URL shape csm-rs has always used;
//!  - map `progenitor_client::Error<T>` into `crate::error::Error`.
//!
//! Per-resource wrapper files (`group.rs`, `memberships.rs`, …) hold
//! `impl ShastaClient { pub async fn hsm_*() }` blocks that delegate
//! to the generated client.

use crate::{ShastaClient, error::Error, hsm::generated};

/// Build a generated HSM `Client` bound to the caller's token. Cheap to
/// recreate per call: `reqwest::Client` clones are `Arc`-internal, and
/// the generated `Client` is a thin newtype around it.
pub(crate) fn gen_client(client: &ShastaClient, token: &str) -> generated::Client {
    // Inject the bearer token as a default header on a fresh reqwest::Client
    // built from the configured one. This avoids depending on progenitor's
    // (version-sensitive) middleware story.
    let mut headers = reqwest::header::HeaderMap::new();
    let auth = format!("Bearer {token}");
    let mut value = reqwest::header::HeaderValue::from_str(&auth)
        .expect("bearer token contained header-invalid bytes");
    value.set_sensitive(true);
    headers.insert(reqwest::header::AUTHORIZATION, value);

    // Rebuild a reqwest::Client carrying the bearer header and the same
    // TLS / proxy config as the shared one.
    let inner = reqwest::Client::builder()
        .default_headers(headers)
        .add_root_certificate(
            reqwest::Certificate::from_pem(client.root_cert()).expect("invalid root cert"),
        );
    let inner = match client.socks5_proxy() {
        Some(p) => inner.proxy(reqwest::Proxy::all(p).expect("invalid socks5 proxy")),
        None => inner,
    };
    let inner = inner.build().expect("reqwest client build failed");

    // Override spec basePath: csm-rs's `base_url` already ends in `/apis`.
    let baseurl = format!("{}/smd/hsm/v2", client.base_url());
    generated::Client::new_with_client(&baseurl, inner)  // __CLIENT_NEW__ — verify against reference
}

/// Map a generated `Error` into the crate's `Error` enum.
pub(crate) fn map_err<E: std::fmt::Debug>(
    err: progenitor_client::Error<E>,
) -> Error {
    use progenitor_client::Error::*;
    match err {
        // Variant names taken from the progenitor reference captured in Task 0.
        // Adjust this match arm-by-arm against the actual enum.
        CommunicationError(e) => Error::NetError(e),
        ErrorResponse(rv) => Error::CsmError(format!("HSM {}: {:?}", rv.status(), rv.into_inner())),
        InvalidRequest(s) => Error::CsmError(format!("invalid request: {s}")),
        InvalidResponsePayload(_, e) => Error::JsonError(e),
        other => Error::CsmError(format!("{other:?}")),
    }
}
```

- [ ] **Step 3: Wire the new modules into `src/hsm/mod.rs`**

Open `src/hsm/mod.rs`. After the existing `pub mod service;` line (and before `pub mod types;`), add:
```rust
pub(crate) mod generated;
mod wrapper;
```

- [ ] **Step 4: Verify it compiles**

Run:
```bash
cargo build 2>&1 | tail -30
```
Expected: `Finished`. If `progenitor_client::Error` variant names don't match, the compiler will tell you exactly which arms are wrong; update them against the names captured in the reference file from Task 0.

- [ ] **Step 5: Commit**

```bash
git add src/hsm/generated.rs src/hsm/wrapper src/hsm/mod.rs
git commit -m "feat(hsm): scaffold generated module + wrapper skeleton (gen_client, map_err)"
```

---

## Phase 2: Per-resource migrations (smallest → biggest)

> **For every migration task below, the same rhythm applies:**
> 1. Write the wrapper file using the generated method name looked up from the Task 0 reference.
> 2. Replace `*/types.rs` content with `pub use` aliases (looking up generated type names from the same reference).
> 3. Port `#[cfg(test)]` tests against the new aliases — they verify the types still round-trip.
> 4. Delete the old `http_client*.rs`.
> 5. `cargo build && cargo test`.
> 6. `(cd ../manta && cargo check)` if a manta checkout is present.
> 7. Commit.

### Task 4: Migrate `service/values/role` (1 method — smallest)

**Files:**
- Create: `src/hsm/wrapper/service_values.rs`
- Modify: `src/hsm/service/values/role/types.rs`
- Modify: `src/hsm/service/values/role/mod.rs`
- Delete: `src/hsm/service/values/role/http_client.rs`

- [ ] **Step 1: Look up the generated names from the reference file**

Open `docs/superpowers/plans/2026-06-13-progenitor-output-reference.md` and note:
  - The Rust type for YAML schema implied by `GET /service/values/role` response (likely something like `RoleArray`, `HMSRole100`, or `serde_json::Value`).
  - The method name on `Client` for `operationId: doRoleGet`.

Carry these into Steps 2–3.

- [ ] **Step 2: Create the wrapper file**

Create `src/hsm/wrapper/service_values.rs`:
```rust
//! Wrapper for `/service/values/role`. Replaces
//! `src/hsm/service/values/role/http_client.rs`.

use crate::{ShastaClient, error::Error, hsm::service::values::role::types::Role};

use super::{gen_client, map_err};

impl ShastaClient {
    /// `GET /smd/hsm/v2/service/values/role` — list known component roles.
    pub async fn hsm_service_values_role_get(
        &self,
        token: &str,
    ) -> Result<Role, Error> {
        // Replace `do_role_get` with the name captured in the reference file.
        let rv = gen_client(self, token).do_role_get().await.map_err(map_err)?;
        Ok(rv.into_inner())
    }
}
```

If the generated response type differs from the existing `Role` struct in shape (i.e., the round-trip test in Step 3 fails), wrap the generated call in a `From` conversion at the return site rather than changing the public type.

- [ ] **Step 3: Replace `types.rs` with a pure re-export**

Open `src/hsm/service/values/role/types.rs`. Replace ENTIRE contents with:
```rust
//! Re-export of the progenitor-generated `Role` schema.

pub use crate::hsm::generated::types::Role as Role;  // adjust name per Task 0 reference
```

If `Role` is the field-by-field hand-rolled struct today and the generated equivalent has the same JSON shape, this swap is invisible to consumers.

- [ ] **Step 4: Register the wrapper file in `src/hsm/wrapper/mod.rs`**

At the bottom of `src/hsm/wrapper/mod.rs`, add:
```rust
mod service_values;
```

- [ ] **Step 5: Update `src/hsm/service/values/role/mod.rs`**

Open it and:
  - Delete the `pub mod http_client;` line.
  - Keep `pub mod types;` and any `pub use types::*;` re-export.

- [ ] **Step 6: Delete the old http client**

Run:
```bash
git rm src/hsm/service/values/role/http_client.rs
```

- [ ] **Step 7: Build and test**

Run:
```bash
cargo build 2>&1 | tail -20
cargo test --lib hsm::service 2>&1 | tail -20
```
Expected: builds; any existing tests for `Role` still pass against the re-exported alias.

- [ ] **Step 8: Sanity-check downstream compiles (if a manta checkout is present)**

Run:
```bash
test -d ../manta && (cd ../manta && cargo check 2>&1 | tail -10) || echo "no ../manta — skip"
```
Expected: `Finished` or `no ../manta — skip`. If errors mention `hsm_service_values_role_get` or `Role`, the public API drifted — investigate before continuing.

- [ ] **Step 9: Commit**

```bash
git add src/hsm/wrapper/service_values.rs src/hsm/wrapper/mod.rs \
        src/hsm/service/values/role/types.rs src/hsm/service/values/role/mod.rs
git commit -m "refactor(hsm): generate /service/values/role wrapper from progenitor"
```

---

### Task 5: Migrate `memberships` (2 methods — no projections)

**Files:**
- Create: `src/hsm/wrapper/memberships.rs`
- Modify: `src/hsm/memberships/types.rs`
- Modify: `src/hsm/memberships/mod.rs`
- Delete: `src/hsm/memberships/http_client.rs`

- [ ] **Step 1: Look up the generated names**

From `docs/superpowers/plans/2026-06-13-progenitor-output-reference.md`:
  - Type for YAML `Membership.1.0.0`.
  - Methods for `operationId: doMembershipsGet` and `doMembershipGet`.

- [ ] **Step 2: Create the wrapper file**

Create `src/hsm/wrapper/memberships.rs`:
```rust
//! Wrapper for `/memberships`. Replaces
//! `src/hsm/memberships/http_client.rs`.

use crate::{ShastaClient, error::Error, hsm::memberships::types::Membership};

use super::{gen_client, map_err};

impl ShastaClient {
    /// `GET /smd/hsm/v2/memberships`.
    pub async fn hsm_memberships_get_all(
        &self,
        token: &str,
    ) -> Result<Vec<Membership>, Error> {
        let rv = gen_client(self, token).do_memberships_get().await.map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `GET /smd/hsm/v2/memberships/{xname}`.
    pub async fn hsm_memberships_get_xname(
        &self,
        token: &str,
        xname: &str,
    ) -> Result<Membership, Error> {
        let rv = gen_client(self, token).do_membership_get(xname).await.map_err(map_err)?;
        Ok(rv.into_inner())
    }
}
```

- [ ] **Step 3: Replace `types.rs` with a pure re-export**

`src/hsm/memberships/types.rs` becomes:
```rust
//! Re-export of the progenitor-generated `Membership` schema.

pub use crate::hsm::generated::types::Membership_1_0_0 as Membership;  // adjust per Task 0 reference
```

- [ ] **Step 4: Register the wrapper file**

In `src/hsm/wrapper/mod.rs`, add: `mod memberships;`

- [ ] **Step 5: Update `src/hsm/memberships/mod.rs`**

Delete the `pub mod http_client;` line.

- [ ] **Step 6: Delete the old http client**

```bash
git rm src/hsm/memberships/http_client.rs
```

- [ ] **Step 7: Build and test**

```bash
cargo build 2>&1 | tail -20
cargo test --lib hsm::memberships 2>&1 | tail -20
```

- [ ] **Step 8: Downstream check**

```bash
test -d ../manta && (cd ../manta && cargo check 2>&1 | tail -10) || echo "no ../manta — skip"
```

- [ ] **Step 9: Commit**

```bash
git add src/hsm/wrapper/memberships.rs src/hsm/wrapper/mod.rs \
        src/hsm/memberships/types.rs src/hsm/memberships/mod.rs
git commit -m "refactor(hsm): generate /memberships wrapper from progenitor"
```

---

### Task 6: Migrate `group` (5 methods + GroupExt for inherent methods)

**Files:**
- Create: `src/hsm/wrapper/group.rs`
- Create: `src/hsm/group/ext.rs`
- Modify: `src/hsm/group/types.rs`
- Modify: `src/hsm/group/mod.rs`
- Delete: `src/hsm/group/http_client.rs`

- [ ] **Step 1: Look up the generated names**

From the reference file: types for `Group.1.0.0`, `Members.1.0.0`; methods for `doGroupsGet`, `doGroupGet`, `doGroupsPost`, `doGroupDelete`, `doGroupMembersPost`, `doGroupMemberDelete`.

- [ ] **Step 2: Create the extension trait that preserves Group's inherent methods**

Create `src/hsm/group/ext.rs`:
```rust
//! Convenience methods previously defined as inherent `impl Group` in
//! `types.rs`. Now exposed as a trait because `Group` is a generated
//! type and inherent impl blocks would collide with the generated impls.
//!
//! Callers add `use csm_rs::hsm::group::GroupExt;` to keep using these.

use super::types::{Group, Members};

pub trait GroupExt: Sized {
    /// Build a new `Group` with `label` and an optional `members.ids` list.
    fn new_with_members(label: &str, member_vec_opt: Option<Vec<&str>>) -> Self;

    /// All member xnames as owned `Vec<String>`; empty if the field is absent.
    fn get_members(&self) -> Vec<String>;

    /// Same as `get_members` but distinguishes "no members" from "field missing".
    fn get_members_opt(&self) -> Option<Vec<String>>;

    /// Append `xnames` to `members.ids`. Returns the resulting list.
    fn add_xnames(&mut self, xnames: &[String]) -> Vec<String>;
}

impl GroupExt for Group {
    fn new_with_members(label: &str, member_vec_opt: Option<Vec<&str>>) -> Self {
        let members_opt = member_vec_opt.map(|member_vec| Members {
            ids: Some(member_vec.iter().map(|&id| id.to_string()).collect()),
        });
        Self {
            label: label.to_string(),
            description: None,
            tags: None,
            members: members_opt,
            exclusive_group: None,
        }
    }

    fn get_members(&self) -> Vec<String> {
        self.members
            .as_ref()
            .and_then(|m| m.ids.clone())
            .unwrap_or_default()
    }

    fn get_members_opt(&self) -> Option<Vec<String>> {
        self.members.as_ref().and_then(|m| m.ids.clone())
    }

    fn add_xnames(&mut self, xnames: &[String]) -> Vec<String> {
        if let Some(members) = self.members.as_mut() {
            if let Some(ids) = members.ids.as_mut() {
                ids.extend_from_slice(xnames);
            }
        }
        self.get_members()
    }
}
```

If the generated `Group` struct has different field names (e.g. `exclusive_group` vs `exclusiveGroup` vs `exclusiveGroup_`), update the field accesses in this file. The compiler will tell you which fields are wrong.

- [ ] **Step 3: Create the wrapper file**

Create `src/hsm/wrapper/group.rs`:
```rust
//! Wrapper for `/groups`. Replaces `src/hsm/group/http_client.rs`.

use crate::{
    ShastaClient,
    error::Error,
    hsm::{group::types::Group, types::ResourceURI},
};

use super::{gen_client, map_err};

impl ShastaClient {
    /// `GET /smd/hsm/v2/groups` — list all groups.
    pub async fn hsm_groups_get_all(
        &self,
        token: &str,
    ) -> Result<Vec<Group>, Error> {
        let rv = gen_client(self, token)
            .do_groups_get(None, None)  // group=, tag=
            .await
            .map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `GET /smd/hsm/v2/groups/{label}` — one group by label.
    pub async fn hsm_group_get_one(
        &self,
        token: &str,
        label: &str,
    ) -> Result<Group, Error> {
        let rv = gen_client(self, token)
            .do_group_get(label)
            .await
            .map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `POST /smd/hsm/v2/groups` — create a new group.
    pub async fn hsm_group_post(
        &self,
        token: &str,
        group: &Group,
    ) -> Result<ResourceURI, Error> {
        let rv = gen_client(self, token)
            .do_groups_post(group)
            .await
            .map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `DELETE /smd/hsm/v2/groups/{label}`.
    pub async fn hsm_group_delete(
        &self,
        token: &str,
        label: &str,
    ) -> Result<(), Error> {
        gen_client(self, token).do_group_delete(label).await.map_err(map_err)?;
        Ok(())
    }

    /// `POST /smd/hsm/v2/groups/{label}/members`.
    pub async fn hsm_group_member_post(
        &self,
        token: &str,
        label: &str,
        member_id: &str,
    ) -> Result<(), Error> {
        gen_client(self, token)
            .do_group_members_post(label, &serde_json::json!({"id": member_id}))
            .await
            .map_err(map_err)?;
        Ok(())
    }

    /// `DELETE /smd/hsm/v2/groups/{label}/members/{member_id}`.
    pub async fn hsm_group_member_delete(
        &self,
        token: &str,
        label: &str,
        member_id: &str,
    ) -> Result<(), Error> {
        gen_client(self, token)
            .do_group_member_delete(label, member_id)
            .await
            .map_err(map_err)?;
        Ok(())
    }
}
```

Method names and parameter shapes will differ from these guesses depending on what progenitor emitted. Use the reference file from Task 0 and the compiler errors to nail down the exact signatures.

- [ ] **Step 4: Replace `src/hsm/group/types.rs` with re-exports**

```rust
//! Re-export of the progenitor-generated `Group`/`Members` schemas.
//!
//! `Member` (the singular helper) has no spec counterpart and stays
//! hand-rolled because some callers serialize a single-member body.

use serde::{Deserialize, Serialize};

pub use crate::hsm::generated::types::Group_1_0_0 as Group;       // adjust per reference
pub use crate::hsm::generated::types::Members_1_0_0 as Members;   // adjust per reference

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Member {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
```

- [ ] **Step 5: Update `src/hsm/group/mod.rs`**

Open it. Delete `pub mod http_client;`. Add:
```rust
pub mod ext;
pub use ext::GroupExt;
```

If a `tests` module is declared, leave it alone — it will compile against the new aliases.

- [ ] **Step 6: Register the wrapper file**

In `src/hsm/wrapper/mod.rs` add: `mod group;`

- [ ] **Step 7: Delete the old http client**

```bash
git rm src/hsm/group/http_client.rs
```

- [ ] **Step 8: Build and test**

```bash
cargo build 2>&1 | tail -30
cargo test --lib hsm::group 2>&1 | tail -30
```
Expected: build succeeds; existing `src/hsm/group/tests.rs` passes against the alias.

- [ ] **Step 9: Downstream check**

```bash
test -d ../manta && (cd ../manta && cargo check 2>&1 | tail -10) || echo "no ../manta — skip"
```
If manta breaks because callers used `group.label` syntax with a different field name on the generated type, alias the field via a `From`/`Into` conversion in `wrapper/group.rs` rather than changing the public type.

- [ ] **Step 10: Commit**

```bash
git add src/hsm/wrapper/group.rs src/hsm/wrapper/mod.rs \
        src/hsm/group/ext.rs src/hsm/group/types.rs src/hsm/group/mod.rs
git commit -m "refactor(hsm): generate /groups wrapper from progenitor + GroupExt trait"
```

---

### Task 7: Migrate `component` (5+ methods, 15-field struct)

**Files:**
- Create: `src/hsm/wrapper/component.rs`
- Modify: `src/hsm/component/types.rs`
- Modify: `src/hsm/component/mod.rs`
- Delete: `src/hsm/component/http_client.rs`

- [ ] **Step 1: Look up generated names**

From the reference file: types for `Component.1.0.0_Component`, `ComponentArray_ComponentArray`, `ComponentArray_PostArray`, `ComponentArray_PostQuery`, `ComponentArray_PostByNIDQuery`, `Component.1.0.0_ComponentCreate`, `Component.1.0.0_Put`. Methods for `doComponentsGet`, `doComponentGet`, `doComponentsPost`, `doComponentPut`, `doComponentDelete`, `doComponentsDeleteAll`, `doComponentsPostByNIDQuery`.

- [ ] **Step 2: Create the wrapper file**

Create `src/hsm/wrapper/component.rs`:
```rust
//! Wrapper for `/State/Components`. Replaces
//! `src/hsm/component/http_client.rs`.

use crate::{
    ShastaClient,
    error::Error,
    hsm::component::types::{
        Component, ComponentArray, ComponentArrayPostArray, ComponentArrayPostByNidQuery,
        ComponentArrayPostQuery, ComponentCreate, ComponentPut,
    },
};

use super::{gen_client, map_err};

impl ShastaClient {
    /// `GET /smd/hsm/v2/State/Components` — list/filter components.
    pub async fn hsm_components_get_all(
        &self,
        token: &str,
    ) -> Result<ComponentArray, Error> {
        let rv = gen_client(self, token).do_components_get(/* query params */).await.map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `GET /smd/hsm/v2/State/Components/{xname}` — one component.
    pub async fn hsm_component_get_one(
        &self,
        token: &str,
        xname: &str,
    ) -> Result<Component, Error> {
        let rv = gen_client(self, token).do_component_get(xname).await.map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `POST /smd/hsm/v2/State/Components` — create components in bulk.
    pub async fn hsm_components_post(
        &self,
        token: &str,
        body: &ComponentArrayPostArray,
    ) -> Result<(), Error> {
        gen_client(self, token).do_components_post(body).await.map_err(map_err)?;
        Ok(())
    }

    /// `PUT /smd/hsm/v2/State/Components/{xname}` — replace one.
    pub async fn hsm_component_put(
        &self,
        token: &str,
        xname: &str,
        body: &ComponentPut,
    ) -> Result<(), Error> {
        gen_client(self, token).do_component_put(xname, body).await.map_err(map_err)?;
        Ok(())
    }

    /// `POST /smd/hsm/v2/State/Components/ByNID/Query`.
    pub async fn hsm_components_by_nid_query_post(
        &self,
        token: &str,
        body: &ComponentArrayPostByNidQuery,
    ) -> Result<ComponentArray, Error> {
        let rv = gen_client(self, token)
            .do_components_post_by_nid_query(body).await.map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `DELETE /smd/hsm/v2/State/Components/{xname}`.
    pub async fn hsm_component_delete(
        &self,
        token: &str,
        xname: &str,
    ) -> Result<(), Error> {
        gen_client(self, token).do_component_delete(xname).await.map_err(map_err)?;
        Ok(())
    }

    /// `DELETE /smd/hsm/v2/State/Components` — remove all components.
    pub async fn hsm_components_delete_all(
        &self,
        token: &str,
    ) -> Result<(), Error> {
        gen_client(self, token).do_components_delete_all().await.map_err(map_err)?;
        Ok(())
    }
}
```

Adjust the `do_components_get` call once you read the generated signature — progenitor turns each query parameter into a positional argument.

- [ ] **Step 3: Replace `src/hsm/component/types.rs` with re-exports**

```rust
//! Re-exports of the progenitor-generated Component-family schemas.

pub use crate::hsm::generated::types::{
    Component_1_0_0_Component       as Component,
    Component_1_0_0_ComponentCreate as ComponentCreate,
    Component_1_0_0_Put             as ComponentPut,
    ComponentArray_ComponentArray    as ComponentArray,
    ComponentArray_PostArray         as ComponentArrayPostArray,
    ComponentArray_PostQuery         as ComponentArrayPostQuery,
    ComponentArray_PostByNIDQuery    as ComponentArrayPostByNidQuery,
};
// Adjust names per Task 0 reference.
```

- [ ] **Step 4: Update `src/hsm/component/mod.rs`**

Delete `pub mod http_client;` (and any sub-modules of it that no longer exist).

- [ ] **Step 5: Register the wrapper file**

In `src/hsm/wrapper/mod.rs`: `mod component;`

- [ ] **Step 6: Delete the old http client**

```bash
git rm src/hsm/component/http_client.rs
```

- [ ] **Step 7: Build and test**

```bash
cargo build 2>&1 | tail -30
cargo test --lib hsm::component 2>&1 | tail -30
```

- [ ] **Step 8: Downstream check**

```bash
test -d ../manta && (cd ../manta && cargo check 2>&1 | tail -10) || echo "no ../manta — skip"
```

- [ ] **Step 9: Commit**

```bash
git add src/hsm/wrapper/component.rs src/hsm/wrapper/mod.rs \
        src/hsm/component/types.rs src/hsm/component/mod.rs
git commit -m "refactor(hsm): generate /State/Components wrapper from progenitor"
```

---

### Task 8: Migrate `component_status` (chunking-loop stays in wrapper)

**Files:**
- Create: `src/hsm/wrapper/component_status.rs`
- Modify: `src/hsm/component_status/mod.rs`
- Delete: `src/hsm/component_status/http_client/mod.rs`

- [ ] **Step 1: Read the existing chunking logic so it can be preserved**

Open `src/hsm/component_status/http_client/mod.rs` and note:
  - the per-batch ID limit (likely a `const` near the top of the file),
  - the iterator/chunks pattern that builds repeated `GET /State/Components?id=…&id=…` calls,
  - error and result merging logic.

- [ ] **Step 2: Create the wrapper file**

Create `src/hsm/wrapper/component_status.rs`:
```rust
//! Wrapper for chunked `GET /State/Components?id=…`. Replaces the
//! handcoded chunking client at `src/hsm/component_status/http_client/`.

use crate::{
    ShastaClient,
    error::Error,
    hsm::component::types::{Component, ComponentArray},
};

use super::{gen_client, map_err};

// Match this to the chunk size the old http_client used.
// Use the same constant — search the deleted file's history if needed.
const HSM_QUERY_IDS_CHUNK: usize = 200;

impl ShastaClient {
    /// `GET /smd/hsm/v2/State/Components?id=&id=…`, chunked across
    /// multiple requests when `xnames.len()` exceeds the per-request limit.
    pub async fn hsm_component_status_get(
        &self,
        token: &str,
        xnames: &[String],
    ) -> Result<Vec<Component>, Error> {
        let mut out = Vec::with_capacity(xnames.len());
        for chunk in xnames.chunks(HSM_QUERY_IDS_CHUNK) {
            let ids: Vec<&str> = chunk.iter().map(String::as_str).collect();
            let rv = gen_client(self, token)
                .do_components_get(Some(ids.as_slice()), /* other params */)
                .await
                .map_err(map_err)?;
            let ComponentArray { components, .. } = rv.into_inner();
            if let Some(c) = components {
                out.extend(c);
            }
        }
        Ok(out)
    }
}
```

Replace `do_components_get(Some(ids.as_slice()), …)` with the actual signature progenitor emitted. The query parameter for ids may be a `Vec<String>` instead of `&[&str]` — read the signature.

- [ ] **Step 3: Update `src/hsm/component_status/mod.rs`**

Delete the `pub mod http_client;` line (and any related sub-mod imports).

- [ ] **Step 4: Register the wrapper file**

In `src/hsm/wrapper/mod.rs`: `mod component_status;`

- [ ] **Step 5: Delete the old http client**

```bash
git rm -r src/hsm/component_status/http_client
```

- [ ] **Step 6: Build and test**

```bash
cargo build 2>&1 | tail -30
cargo test --lib hsm::component_status 2>&1 | tail -30
```

- [ ] **Step 7: Downstream check**

```bash
test -d ../manta && (cd ../manta && cargo check 2>&1 | tail -10) || echo "no ../manta — skip"
```

- [ ] **Step 8: Commit**

```bash
git add src/hsm/wrapper/component_status.rs src/hsm/wrapper/mod.rs \
        src/hsm/component_status/mod.rs
git commit -m "refactor(hsm): generate chunked /State/Components GET wrapper from progenitor"
```

---

### Task 9: Migrate `hw_component` (1415-line types.rs; projection types move to wrapper)

**Files:**
- Create: `src/hsm/wrapper/hw_component.rs`
- Create: `src/hsm/wrapper/hw_component_types.rs`
- Modify: `src/hsm/hw_inventory/hw_component/types.rs`
- Modify: `src/hsm/hw_inventory/hw_component/mod.rs`
- Modify: `src/hsm/hw_inventory/hw_component/utils.rs` (only if it imports types being moved)
- Delete: `src/hsm/hw_inventory/hw_component/http_client.rs`

- [ ] **Step 1: Move projection types to the wrapper layer**

Cut the entire `NodeSummary`, `ArtifactSummary`, `ArtifactType` blocks and their `impl`s out of `src/hsm/hw_inventory/hw_component/types.rs`. Paste them verbatim into a new file at `src/hsm/wrapper/hw_component_types.rs` and prepend this module header:

```rust
//! Projection types returned by csm-rs's `hsm_hw_inventory_get`. They
//! have no spec equivalent; the wrapper builds them from progenitor's
//! generated `HWInventoryByLocation` type.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum_macros::{AsRefStr, Display, EnumString, IntoStaticStr};

use crate::error::Error;

// <paste the cut NodeSummary, ArtifactSummary, ArtifactType, helper fns here>
```

Adjust the `use serde_json::Value` import — if you keep `try_from_csm_value` for backwards compat, leave it; the `try_from_generated` is added in Step 2.

- [ ] **Step 2: Add `try_from_generated` constructors**

In `src/hsm/wrapper/hw_component_types.rs`, after the existing `impl NodeSummary`, append:

```rust
impl NodeSummary {
    /// Build a `NodeSummary` from progenitor's `HWInventoryByLocation`
    /// payload's first node entry.
    pub fn try_from_generated(
        v: &crate::hsm::generated::types::HwInventory_1_0_0_HwInventoryByLocation,  // adjust per reference
    ) -> Result<Self, Error> {
        // Easiest correct implementation: serialize the generated value to
        // JSON and reuse the existing pointer-based projection. Avoids
        // re-deriving every field-by-field walk while we wait for typed
        // schemas to stabilize.
        let value = serde_json::to_value(v)
            .map_err(Error::JsonError)?;
        Self::try_from_csm_value(&value)
    }
}
```

If the existing `try_from_csm_value` does not exist — i.e. the projection logic uses standalone helper functions — wrap the helpers in a `try_from_csm_value` first, then delegate. This keeps Task 9 a non-rewrite of the projection.

- [ ] **Step 3: Replace `src/hsm/hw_inventory/hw_component/types.rs` with pure re-exports**

```rust
//! Re-exports of the progenitor-generated HW-inventory schemas plus a
//! re-export of the projection types that live in the wrapper module.

pub use crate::hsm::generated::types::{
    HwInventory_1_0_0_HwInventoryByLocation as HWInventory,     // adjust per reference
    HwInventory_1_0_0_HwInventoryByFRU      as HWInventoryByFRU,
    // …add every additional HWInvByLoc* / HWInvByFRU* re-export the
    // pre-migration types.rs file used to expose. The list lives in
    // git history of this file.
};

// Projection types are owned by the wrapper layer but exposed here so
// the existing public path stays valid.
pub use crate::hsm::wrapper::hw_component_types::{ArtifactSummary, ArtifactType, NodeSummary};
```

Inspect the pre-migration `git show HEAD~1:src/hsm/hw_inventory/hw_component/types.rs` to enumerate the full re-export list. Every `pub struct` in the old file becomes one `pub use` line here.

- [ ] **Step 4: Create the wrapper file with the projection at `hsm_hw_inventory_get`**

Create `src/hsm/wrapper/hw_component.rs`:
```rust
//! Wrapper for `/Inventory/Hardware`. Replaces
//! `src/hsm/hw_inventory/hw_component/http_client.rs`.

use crate::{
    ShastaClient,
    error::Error,
    hsm::{
        hw_inventory::hw_component::types::HWInventory,
        types::HsmActionResponse,
    },
    hsm::wrapper::hw_component_types::NodeSummary,
};

use super::{gen_client, map_err};

impl ShastaClient {
    /// `GET /smd/hsm/v2/Inventory/Hardware` projecting the response into a
    /// `NodeSummary`.
    pub async fn hsm_hw_inventory_get(
        &self,
        token: &str,
        xname: &str,
    ) -> Result<NodeSummary, Error> {
        let rv = gen_client(self, token)
            .do_hw_inv_by_location_get(/* params */)
            .await
            .map_err(map_err)?;
        let inv = rv.into_inner();
        NodeSummary::try_from_generated(&inv)
    }

    /// `GET /smd/hsm/v2/Inventory/Hardware/Query/{xname}` — typed query.
    pub async fn hsm_hw_inventory_get_query(
        &self,
        token: &str,
        xname: &str,
    ) -> Result<HWInventory, Error> {
        let rv = gen_client(self, token)
            .do_hw_inv_by_location_query_get(xname)
            .await
            .map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `POST /smd/hsm/v2/Inventory/Hardware`.
    pub async fn hsm_hw_inventory_post(
        &self,
        token: &str,
        body: &HWInventory,  // adjust to the generated request body type
    ) -> Result<HsmActionResponse, Error> {
        let rv = gen_client(self, token)
            .do_hw_inv_by_location_post(body)
            .await
            .map_err(map_err)?;
        Ok(rv.into_inner())
    }
}
```

- [ ] **Step 5: Update `src/hsm/hw_inventory/hw_component/mod.rs`**

Delete `pub mod http_client;`. Keep the existing `pub use types::*;` style re-export (or whatever was there).

- [ ] **Step 6: Patch imports in `src/hsm/hw_inventory/hw_component/utils.rs`**

If `utils.rs` imports `NodeSummary`/`ArtifactSummary` from `types::`, change to import from `crate::hsm::wrapper::hw_component_types::` or rely on the re-export added in Step 3.

Run `cargo check` after this step to confirm there are no dangling imports.

- [ ] **Step 7: Register the wrapper files**

In `src/hsm/wrapper/mod.rs`, add:
```rust
mod hw_component;
pub(crate) mod hw_component_types;
```

(Public to crate so `types.rs` can re-export from it.)

- [ ] **Step 8: Delete the old http client**

```bash
git rm src/hsm/hw_inventory/hw_component/http_client.rs
```

- [ ] **Step 9: Build and test**

```bash
cargo build 2>&1 | tail -30
cargo test --lib hsm::hw_inventory::hw_component 2>&1 | tail -30
```
Expected: existing pointer-based projection tests (e.g. the ones that exercise `try_from_csm_value` on JSON fixtures) still pass; the new `try_from_generated` path is exercised by any wrapper-level test you wish to add.

- [ ] **Step 10: Downstream check**

```bash
test -d ../manta && (cd ../manta && cargo check 2>&1 | tail -10) || echo "no ../manta — skip"
```

- [ ] **Step 11: Commit**

```bash
git add src/hsm/wrapper/hw_component.rs src/hsm/wrapper/hw_component_types.rs \
        src/hsm/wrapper/mod.rs \
        src/hsm/hw_inventory/hw_component/types.rs \
        src/hsm/hw_inventory/hw_component/mod.rs \
        src/hsm/hw_inventory/hw_component/utils.rs
git commit -m "refactor(hsm): generate /Inventory/Hardware wrapper + move NodeSummary to wrapper layer"
```

---

### Task 10: Migrate `redfish_endpoint` (fixes RediscoverOnUpdate typo)

**Files:**
- Create: `src/hsm/wrapper/redfish_endpoint.rs`
- Modify: `src/hsm/hw_inventory/redfish_endpoint/types.rs`
- Modify: `src/hsm/hw_inventory/redfish_endpoint/mod.rs`
- Delete: `src/hsm/hw_inventory/redfish_endpoint/http_client.rs`

- [ ] **Step 1: Look up the generated names**

From the reference file: types for `RedfishEndpoint.1.0.0_RedfishEndpoint`, `RedfishEndpointArray_RedfishEndpointArray`. Methods for `doRedfishEndpointsGet`, `doRedfishEndpointPost`, `doRedfishEndpointPut`, `doRedfishEndpointDelete`, `doRedfishEndpointQueryGet`.

Also: **note** that adopting the generated `RedfishEndpoint` fixes the existing `RediscoveryOnUpdate` vs `RediscoverOnUpdate` field-name typo. Call this out in the commit message so reviewers see it as a behaviour delta.

- [ ] **Step 2: Create the wrapper file**

Create `src/hsm/wrapper/redfish_endpoint.rs`:
```rust
//! Wrapper for `/Inventory/RedfishEndpoints`. Replaces
//! `src/hsm/hw_inventory/redfish_endpoint/http_client.rs`.

use crate::{
    ShastaClient,
    error::Error,
    hsm::{
        hw_inventory::redfish_endpoint::types::{RedfishEndpoint, RedfishEndpointArray},
        types::HsmActionResponse,
    },
};

use super::{gen_client, map_err};

impl ShastaClient {
    /// `GET /smd/hsm/v2/Inventory/RedfishEndpoints` (filterable).
    #[allow(clippy::too_many_arguments)]
    pub async fn hsm_redfish_get(
        &self,
        token: &str,
        id: Option<&str>,
        fqdn: Option<&str>,
        r#type: Option<&str>,
        uuid: Option<&str>,
        macaddr: Option<&str>,
        ip_address: Option<&str>,
        last_status: Option<&str>,
    ) -> Result<RedfishEndpointArray, Error> {
        let rv = gen_client(self, token)
            .do_redfish_endpoints_get(id, fqdn, r#type, uuid, macaddr, ip_address, last_status)
            .await
            .map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `GET /smd/hsm/v2/Inventory/RedfishEndpoints/Query/{xname}`.
    pub async fn hsm_redfish_get_query(
        &self,
        token: &str,
        xname: &str,
    ) -> Result<RedfishEndpointArray, Error> {
        let rv = gen_client(self, token)
            .do_redfish_endpoint_query_get(xname)
            .await
            .map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `POST /smd/hsm/v2/Inventory/RedfishEndpoints`.
    pub async fn hsm_redfish_post(
        &self,
        token: &str,
        body: &RedfishEndpoint,
    ) -> Result<HsmActionResponse, Error> {
        let rv = gen_client(self, token).do_redfish_endpoint_post(body).await.map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `PUT /smd/hsm/v2/Inventory/RedfishEndpoints/{xname}`.
    pub async fn hsm_redfish_put(
        &self,
        token: &str,
        xname: &str,
        body: &RedfishEndpoint,
    ) -> Result<HsmActionResponse, Error> {
        let rv = gen_client(self, token)
            .do_redfish_endpoint_put(xname, body)
            .await
            .map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `DELETE /smd/hsm/v2/Inventory/RedfishEndpoints/{xname}`.
    pub async fn hsm_redfish_delete_one(
        &self,
        token: &str,
        xname: &str,
    ) -> Result<HsmActionResponse, Error> {
        let rv = gen_client(self, token).do_redfish_endpoint_delete(xname).await.map_err(map_err)?;
        Ok(rv.into_inner())
    }
}
```

- [ ] **Step 3: Replace `types.rs` with re-exports**

```rust
//! Re-exports of progenitor-generated RedfishEndpoint schemas.

pub use crate::hsm::generated::types::{
    RedfishEndpoint_1_0_0_RedfishEndpoint as RedfishEndpoint,           // adjust
    RedfishEndpointArray_RedfishEndpointArray as RedfishEndpointArray,  // adjust
};
```

`DiscoveryInfo` is inlined inside the generated `RedfishEndpoint` (the spec defines it as an inline `properties:` block, not a top-level schema). It is not separately re-exportable; if downstream code references it, expose via a `pub type DiscoveryInfo = …` alias only after confirming the generated nested type name.

- [ ] **Step 4: Update `mod.rs`**

Delete `pub mod http_client;` in `src/hsm/hw_inventory/redfish_endpoint/mod.rs`.

- [ ] **Step 5: Register the wrapper file**

In `src/hsm/wrapper/mod.rs`: `mod redfish_endpoint;`

- [ ] **Step 6: Delete the old http client**

```bash
git rm src/hsm/hw_inventory/redfish_endpoint/http_client.rs
```

- [ ] **Step 7: Build and test**

```bash
cargo build 2>&1 | tail -30
cargo test --lib hsm::hw_inventory::redfish_endpoint 2>&1 | tail -30
```

- [ ] **Step 8: Downstream check**

```bash
test -d ../manta && (cd ../manta && cargo check 2>&1 | tail -10) || echo "no ../manta — skip"
```

- [ ] **Step 9: Commit (note the typo fix in the message)**

```bash
git add src/hsm/wrapper/redfish_endpoint.rs src/hsm/wrapper/mod.rs \
        src/hsm/hw_inventory/redfish_endpoint/types.rs \
        src/hsm/hw_inventory/redfish_endpoint/mod.rs
git commit -m "$(printf 'refactor(hsm): generate /Inventory/RedfishEndpoints wrapper from progenitor\n\nBEHAVIOUR DELTA: the previous hand-rolled RedfishEndpoint had\n#[serde(rename = \"RediscoveryOnUpdate\")] — the spec field is\n\"RediscoverOnUpdate\" (no \"y\"). The generated type uses the correct\nname, so this field now (de)serializes correctly. Today it was always\nsilently None/dropped on the wire.')"
```

---

### Task 11: Migrate `ethernet_interfaces` (resolves EthernetInterface/CompEthInterface ambiguity)

**Files:**
- Create: `src/hsm/wrapper/ethernet_interfaces.rs`
- Modify: `src/hsm/hw_inventory/ethernet_interfaces/types.rs`
- Modify: `src/hsm/hw_inventory/ethernet_interfaces/mod.rs`
- Delete: `src/hsm/hw_inventory/ethernet_interfaces/http_client.rs`

- [ ] **Step 1: Look up the generated names**

From the reference file: types for `CompEthInterface.1.0.0`, `CompEthInterface.1.0.0_Patch`, `CompEthInterface.1.0.0_IPAddressMapping`. Methods for `doCompEthInterfacesGetV2`, `doCompEthInterfaceGetV2`, `doCompEthInterfacePostV2`, `doCompEthInterfacePatchV2`, `doCompEthInterfaceIPAddressPostV2`.

- [ ] **Step 2: Resolve the type duplication**

The pre-migration `types.rs` had TWO competing types (`EthernetInterface` and `ComponentEthernetInterface`), both intended for the same `CompEthInterface.1.0.0` schema, with structural inconsistencies vs the wire. Pick **one** alias for the generated type:

```rust
//! Re-exports of progenitor-generated CompEthInterface schemas.

pub use crate::hsm::generated::types::{
    CompEthInterface_1_0_0 as EthernetInterface,                    // adjust per reference
    CompEthInterface_1_0_0_Patch as EthernetInterfacePatch,         // adjust
    CompEthInterface_1_0_0_IPAddressMapping as IpAddressMapping,    // adjust
};

// `ComponentEthernetInterface` was a partial duplicate of EthernetInterface
// pre-migration. It is dropped here; if any consumer imports it, alias it
// to `EthernetInterface` to preserve the public path:
pub use EthernetInterface as ComponentEthernetInterface;
```

Run `cargo check` after writing this — the compiler will surface every consumer that used the deprecated dual type.

- [ ] **Step 3: Create the wrapper file**

Create `src/hsm/wrapper/ethernet_interfaces.rs`:
```rust
//! Wrapper for `/Inventory/EthernetInterfaces`. Replaces
//! `src/hsm/hw_inventory/ethernet_interfaces/http_client.rs`.

use crate::{
    ShastaClient,
    error::Error,
    hsm::{
        hw_inventory::ethernet_interfaces::types::{
            EthernetInterface, EthernetInterfacePatch, IpAddressMapping,
        },
        types::{HsmActionResponse, ResourceURI},
    },
};

use super::{gen_client, map_err};

impl ShastaClient {
    /// `GET /smd/hsm/v2/Inventory/EthernetInterfaces` (filterable).
    #[allow(clippy::too_many_arguments)]
    pub async fn hsm_ethernet_interfaces_get_all(
        &self,
        token: &str,
        mac_address: Option<&str>,
        ip_address: Option<&str>,
        network: Option<&str>,
        component_id: Option<&str>,
        r#type: Option<&str>,
        older_than: Option<&str>,
        newer_than: Option<&str>,
    ) -> Result<Vec<EthernetInterface>, Error> {
        let rv = gen_client(self, token)
            .do_comp_eth_interfaces_get_v2(
                mac_address, ip_address, network, component_id, r#type, older_than, newer_than,
            )
            .await
            .map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `GET /smd/hsm/v2/Inventory/EthernetInterfaces/{id}`.
    pub async fn hsm_ethernet_interface_get(
        &self,
        token: &str,
        id: &str,
    ) -> Result<EthernetInterface, Error> {
        let rv = gen_client(self, token).do_comp_eth_interface_get_v2(id).await.map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `POST /smd/hsm/v2/Inventory/EthernetInterfaces`.
    pub async fn hsm_ethernet_interface_post(
        &self,
        token: &str,
        body: &EthernetInterface,
    ) -> Result<ResourceURI, Error> {
        let rv = gen_client(self, token).do_comp_eth_interface_post_v2(body).await.map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `PATCH /smd/hsm/v2/Inventory/EthernetInterfaces/{id}`.
    pub async fn hsm_ethernet_interface_patch(
        &self,
        token: &str,
        id: &str,
        body: &EthernetInterfacePatch,
    ) -> Result<EthernetInterface, Error> {
        let rv = gen_client(self, token).do_comp_eth_interface_patch_v2(id, body).await.map_err(map_err)?;
        Ok(rv.into_inner())
    }

    /// `POST /smd/hsm/v2/Inventory/EthernetInterfaces/{id}/IPAddresses`.
    pub async fn hsm_ethernet_interface_ip_post(
        &self,
        token: &str,
        id: &str,
        body: &IpAddressMapping,
    ) -> Result<HsmActionResponse, Error> {
        let rv = gen_client(self, token).do_comp_eth_interface_ip_address_post_v2(id, body).await.map_err(map_err)?;
        Ok(rv.into_inner())
    }
}
```

- [ ] **Step 4: Update `mod.rs`**

Delete `pub mod http_client;` in `src/hsm/hw_inventory/ethernet_interfaces/mod.rs`.

- [ ] **Step 5: Register the wrapper file**

In `src/hsm/wrapper/mod.rs`: `mod ethernet_interfaces;`

- [ ] **Step 6: Delete the old http client**

```bash
git rm src/hsm/hw_inventory/ethernet_interfaces/http_client.rs
```

- [ ] **Step 7: Build and test**

```bash
cargo build 2>&1 | tail -30
cargo test --lib hsm::hw_inventory::ethernet_interfaces 2>&1 | tail -30
```

- [ ] **Step 8: Downstream check**

```bash
test -d ../manta && (cd ../manta && cargo check 2>&1 | tail -10) || echo "no ../manta — skip"
```

- [ ] **Step 9: Commit (note the casing/shape fixes in the message)**

```bash
git add src/hsm/wrapper/ethernet_interfaces.rs src/hsm/wrapper/mod.rs \
        src/hsm/hw_inventory/ethernet_interfaces/types.rs \
        src/hsm/hw_inventory/ethernet_interfaces/mod.rs
git commit -m "$(printf 'refactor(hsm): generate /Inventory/EthernetInterfaces wrapper from progenitor\n\nBEHAVIOUR DELTA: hand-rolled EthernetInterface had snake_case fields\nwithout #[serde(rename)] (would not deserialize the PascalCase spec\nshape) and ip_address: Option<String> (spec is IPAddresses: array).\nThe generated CompEthInterface now used here resolves both. The\nlegacy ComponentEthernetInterface name is preserved as an alias.')"
```

---

## Phase 3: Final cleanup and verification

### Task 12: End-to-end verification + documentation

**Files:**
- Modify: `src/hsm/csm_api_docs.yaml` and `src/hsm/csm_api_docs.openapi3.json` (no edits — just confirm they're in sync)
- Modify: `src/hsm/mod.rs` (only if any submodule `pub use` paths need touch-up)
- Optionally Modify: `CONTRIBUTING.md` or `README.md` (only if these exist and document the HSM client manually)

- [ ] **Step 1: Confirm there are no remaining `http_client*.rs` files in `src/hsm/`**

Run:
```bash
find src/hsm -name 'http_client*' -print
```
Expected: empty output. If anything is left, it was missed by an earlier task — investigate and clean up before continuing.

- [ ] **Step 2: Full build + test sweep**

Run:
```bash
cargo build 2>&1 | tail -20
cargo test --lib 2>&1 | tail -30
cargo clippy --lib 2>&1 | tail -30
```
Expected: build, test, clippy all pass. New clippy warnings introduced by generated code are silenced by the `#![allow(...)]` block in `src/hsm/generated.rs`; any warnings outside that file must be addressed.

- [ ] **Step 3: Document the codegen pipeline in the module-level rustdoc**

Open `src/hsm/mod.rs`. After the existing module-level `//!` doc, append:

```rust
//!
//! ## How this module is built
//!
//! Wire-format types and the underlying HTTP client are **generated**
//! from `src/hsm/csm_api_docs.yaml`:
//!
//! 1. Developer step: `make convert-spec` converts the Swagger 2.0 YAML
//!    to OpenAPI 3.0 (`src/hsm/csm_api_docs.openapi3.json`). Re-run
//!    whenever the YAML changes; the JSON is committed.
//! 2. `build.rs` runs `progenitor` on the JSON and writes the generated
//!    client to `$OUT_DIR/hsm_generated.rs`.
//! 3. `src/hsm/generated.rs` includes the file as a `pub(crate)` module.
//! 4. `src/hsm/wrapper/` glues the generated client to the public
//!    `ShastaClient::hsm_*` API, preserving the historic signatures.
//!
//! Per-resource `types.rs` files are pure re-exports of generated
//! types. Projection types like `NodeSummary` live in the wrapper
//! module (`src/hsm/wrapper/hw_component_types.rs`) and are re-exported
//! through the existing public paths.
```

- [ ] **Step 4: Downstream final check**

```bash
test -d ../manta && (cd ../manta && cargo build 2>&1 | tail -20) || echo "no ../manta — skip"
```
Expected: build succeeds.

- [ ] **Step 5: Commit**

```bash
git add src/hsm/mod.rs
git commit -m "docs(hsm): describe the progenitor-driven codegen pipeline in module docs"
```

- [ ] **Step 6: Verify final git history matches the design's migration order**

Run:
```bash
git log --oneline | head -15
```
Expected: a sequence of commits corresponding to Tasks 0 → 12 in order. If reordering is desirable for review (e.g. squash adjacent typo fixes), use a separate non-interactive rebase planning step outside the scope of this plan.

---

## Self-review notes (kept for the executor)

- **Spec coverage**: Every section of `docs/superpowers/specs/2026-06-13-progenitor-hsm-codegen-design.md` maps to a task above. Architecture → Tasks 1–3. Type strategy → per-resource types.rs in Tasks 4–11. Projection-types-in-wrapper → Task 9 specifically. Auth/basePath → Task 3 wrapper/mod.rs. Migration order → Tasks 4→11 follow the order in the spec verbatim. Phase 0 gate → Task 0. Risks (RediscoverOnUpdate, EthernetInterface) → called out in commit messages for Tasks 10 and 11.
- **No placeholders**: All "fill in" markers are inside the Task 0 reference document (which is itself an artefact populated by Step 6 of Task 0), not in code blocks. Where progenitor's emitted names are unknowable in advance, the plan uses an educated guess as the placeholder and explicitly instructs the engineer to substitute against the reference. The plan does not contain bare "TBD" or "TODO" markers.
- **Type consistency**: Method names used across tasks (`hsm_groups_get_all`, `hsm_group_get_one`, `hsm_components_get_all`, etc.) match the migration order and the existing naming convention in `src/client.rs`. Field names on extension traits (`GroupExt::get_members`) match the historic inherent methods that were on `Group`.
