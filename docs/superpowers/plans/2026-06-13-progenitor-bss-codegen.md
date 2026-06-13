# progenitor-based BSS client codegen — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hand-written `src/bss/` HTTP client and the `BootParameters` wire type with code generated from `src/bss/csm_api_docs.yaml`, preserving the existing public `ShastaClient::bss_bootparameters_*` API.

**Architecture:** Two-step pipeline (BSS spec is Swagger 2.0, same as HSM). (1) Developer step: convert Swagger 2.0 YAML → OpenAPI 3.0 JSON via `swagger2openapi` and commit the JSON; identify and apply any progenitor-rejection patches. (2) `build.rs` runs `progenitor::Generator` on the JSON and writes `$OUT_DIR/bss_generated.rs` alongside `hsm_generated.rs` and `cfs_generated.rs`. Output is `include!`-d into a single `pub(crate)` `src/bss/generated.rs`. A thin wrapper layer at `src/bss/wrapper/` maps the 6 existing `bss_bootparameters_*` methods on `ShastaClient` to generated client calls. Because the BSS module's `dispatcher_conv.rs` is small (35 lines) and there is only one hand-rolled type (`BootParameters`), this migration is a genuine opportunity to fully adopt the generated `BootParameters` type rather than the "move-the-file, defer-routing" outcome that CFS landed.

**Tech Stack:** Rust (edition 2021), `progenitor ~ 0.8`, `openapiv3 ~ 2`, `reqwest 0.12`, `serde 1`, `tokio 1.45`, `swagger2openapi` (npm CLI, developer-only).

**Reference documents:**
- Source design spec: `docs/superpowers/specs/2026-06-13-progenitor-hsm-codegen-design.md` — the BSS migration applies the same architectural decisions.
- HSM precedent plan (also Swagger 2.0): `docs/superpowers/plans/2026-06-13-progenitor-hsm-codegen.md` — closest structural template.
- CFS precedent plan (OpenAPI 3 native): `docs/superpowers/plans/2026-06-13-progenitor-cfs-codegen.md` — its build.rs `generate_one` helper is the foundation BSS extends to a third spec.
- HSM output reference: `docs/superpowers/plans/2026-06-13-progenitor-output-reference.md` — format Task 0 below replicates.
- CFS output reference: `docs/superpowers/plans/2026-06-13-progenitor-cfs-output-reference.md` — second example of the same doc format.
- Memory: `[Partial progenitor migration is OK]` — methods can stay on raw reqwest with documented rationale; same policy applies. **However**, the BSS surface is small enough that full migration is a realistic target this wave.

**BSS-specific deltas from the HSM and CFS migrations:**
- **Spec is Swagger 2.0** (`swagger: '2.0'` at line 1), like HSM. Needs `swagger2openapi` conversion. Unlike HSM, no `Makefile convert-spec` target exists yet for BSS — Task 1 either adds one or re-uses HSM's pattern.
- **Single resource, single API version**: `bootparameters` with 6 methods at `/boot/v1/bootparameters`. No v2/v3 split, no per-resource sub-modules.
- **Single hand-written type**: `BootParameters` at `src/bss/types.rs:16`. Spec defines it under `definitions.BootParams` (line 780 of YAML).
- **Small `dispatcher_conv.rs`**: 35 lines (vs HSM 100s, CFS 218). The "swap the public type to the generated one" outcome is realistic here — flag this as a decision point in Task 3.
- **Tiny scale overall**: 990-line spec, 10 schema definitions, 6 public methods. The whole migration is closer to 4-5 tasks than HSM's 13 or CFS's 10.
- **Spec also defines `meta-data`, `user-data`, `phone-home`, `bootscript`, `hosts`, `dumpstate`, `endpoint-history`, `service/*` endpoints** that csm-rs does NOT expose in its public API. Out of scope — the migration only covers what `src/bss/http_client/mod.rs` currently does.

**Type/method name reference (filled in by Task 0):** `docs/superpowers/plans/2026-06-13-progenitor-bss-output-reference.md`. Subsequent tasks consult this file because progenitor's exact mangled type/method names are not knowable until the generator runs on the converted spec.

---

## Phase 0: Feasibility verification

### Task 0: Convert the spec, run progenitor end-to-end, capture the name mapping

**Why this exists:** progenitor needs OpenAPI 3.0; the BSS spec is Swagger 2.0. Run the same convert+patch loop HSM did to surface any progenitor-rejection issues before the rest of the plan commits to specific type/method names.

**Files:**
- Create: `src/bss/csm_api_docs.openapi3.json` (committed, alongside the YAML)
- Create: `docs/superpowers/plans/2026-06-13-progenitor-bss-output-reference.md` (committed)
- Possibly modify: `src/bss/csm_api_docs.openapi3.json` (post-conversion patches as needed)
- Temporary: `/tmp/progenitor-bss-smoke/` (scratch crate; not committed)

- [ ] **Step 1: Convert the spec from Swagger 2.0 to OpenAPI 3.0**

Run from repo root:
```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
npx --yes swagger2openapi src/bss/csm_api_docs.yaml -o src/bss/csm_api_docs.openapi3.json
```
Expected: non-zero-byte JSON file. Warnings about deprecated fields are fine.

- [ ] **Step 2: Validate the JSON parses as OpenAPI 3.0**

```bash
python3 -c "import json; d=json.load(open('src/bss/csm_api_docs.openapi3.json')); print(d['openapi']); print('paths:', len(d['paths'])); print('schemas:', len(d['components']['schemas']))"
```
Expected: `3.0.0` (the converter's target version), `paths: ~13`, `schemas: ~10`.

- [ ] **Step 3: Scaffold a throwaway crate to test progenitor against the BSS spec**

```bash
mkdir -p /tmp/progenitor-bss-smoke && cd /tmp/progenitor-bss-smoke && cargo init --lib --name progenitor_bss_smoke
cp /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs/src/bss/csm_api_docs.openapi3.json ./spec.json
```

Set `/tmp/progenitor-bss-smoke/Cargo.toml`:
```toml
[package]
name = "progenitor_bss_smoke"
version = "0.0.0"
edition = "2021"

[dependencies]
progenitor-client = "0.8"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
regress = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["serde", "v4"] }

[build-dependencies]
progenitor = "0.8"
openapiv3 = "2"
serde_json = "1"
prettyplease = "0.2"
syn = "2"

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt"] }
wiremock = "0.6"
```

- [ ] **Step 4: Minimal build.rs for the smoke crate**

Create `/tmp/progenitor-bss-smoke/build.rs`:
```rust
use std::{env, fs, path::PathBuf};

fn main() {
    let spec_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("spec.json");
    println!("cargo:rerun-if-changed={}", spec_path.display());

    let src = fs::read_to_string(&spec_path).expect("read spec");
    let spec: openapiv3::OpenAPI =
        serde_json::from_str(&src).expect("spec.json is not valid OpenAPI 3.0");

    let mut generator = progenitor::Generator::default();
    let tokens = generator.generate_tokens(&spec).expect("progenitor codegen");
    let ast: syn::File = syn::parse2(tokens).expect("parse generated tokens");
    let pretty = prettyplease::unparse(&ast);

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("generated.rs");
    fs::write(&out, pretty).expect("write generated.rs");
}
```

Replace `/tmp/progenitor-bss-smoke/src/lib.rs`:
```rust
#![allow(dead_code, clippy::all)]
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
```

- [ ] **Step 5: Run the smoke build**

```bash
cd /tmp/progenitor-bss-smoke && cargo build 2>&1 | tail -60
```
Expected: succeeds. Known failure modes (same as HSM Task 0 and CFS Task 0):
- `progenitor::Generator::generate_tokens` panics → patch the JSON (commit the patches in the same Task 0 commit that lands the JSON).
- `syn::parse2` fails → same; spec issue surfacing through codegen.

If the build fails after up to a day of patching, STOP this plan and report BLOCKED.

- [ ] **Step 6: Inspect the generated code and capture the type/method name reference**

Locate the generated file:
```bash
find /tmp/progenitor-bss-smoke/target -name generated.rs -path '*build*' | head -1
```
Read it. Create `docs/superpowers/plans/2026-06-13-progenitor-bss-output-reference.md` modelled on the HSM and CFS reference docs. Use sections A–H exactly like those (see `docs/superpowers/plans/2026-06-13-progenitor-output-reference.md` and `docs/superpowers/plans/2026-06-13-progenitor-cfs-output-reference.md` as templates).

  **A. Generated type names (by YAML schema name)** — fill in for all 10 schemas. The list (from the YAML):
  ```
  BootParams
  CloudInit
  CloudInitMetadata
  CloudInitUserData
  CloudInitPhoneHome
  Component
  StateInfo
  HostInfo
  EndpointAccess
  Error
  ```

  **A.1. Schemas referenced by the plan that are NOT generated** — empty list if none, otherwise like HSM's A.1.

  **B. Generated method names (by operationId)** — list every operationId in the spec by grepping `operationId:` in `src/bss/csm_api_docs.openapi3.json` (or the YAML). For each one, find the generated Rust method name in the generated file. Focus on the 6 operations csm-rs actually uses (all under `/boot/v1/bootparameters`); list the others under a "not in csm-rs public API" subsection. Plan-mentioned operationIds you'll need:
  ```
  (GET    /boot/v1/bootparameters with no body)         — list/filter
  (GET    /boot/v1/bootparameters with body — same path) — query
  (PUT    /boot/v1/bootparameters)                       — replace
  (POST   /boot/v1/bootparameters)                       — create
  (PATCH  /boot/v1/bootparameters)                       — partial update
  (DELETE /boot/v1/bootparameters)                       — delete (csm-rs's `bss_bootparameters_delete` if present, else absent)
  ```
  The YAML likely names these by their HTTP-verb prefix; capture the actual names progenitor emits.

  **C. progenitor `Error` enum variant list** — copy from the generated file (or progenitor-client 0.8 source if identical to HSM/CFS).

  **D. `Client::new` / `Client::new_with_client` constructor signatures** — copy from the generated file.

  **E. basePath behaviour** — the BSS spec declares `host: bootscriptserver:27778` + `basePath: /apis/bss`. csm-rs's `base_url` already includes `/apis`. So the wrapper would override the server URL to `format!("{}/bss", client.base_url())` and operation paths (`/meta-data`, `/boot/v1/bootparameters`, etc.) would supply the suffixes. Verify with a wiremock test (Step 7).

  **F. Spec patches applied** — describe any JSON patches needed. If none, write "None required — the converted OpenAPI 3.0 spec was accepted as-is." This is the most likely outcome given BSS's small size.

  **G. Generated artefact stats** — total lines, struct count, enum count, method count.

  **H. Runtime crate dependencies** — verify whether the BSS-generated code needs any deps that aren't already in csm-rs's Cargo.toml. After HSM/CFS migrations csm-rs already has `progenitor-client = "0.8"`, `regress = "0.10"`, `chrono` with `serde`, and `uuid` with `serde`. Most likely outcome: no new deps. Verify by grepping the generated file for `chrono::`, `uuid::`, `regress::`, etc.

- [ ] **Step 7: URL/basePath sanity check**

Add a wiremock test to `/tmp/progenitor-bss-smoke/src/lib.rs` that hits a small operation (e.g. `GET /meta-data`). Look up the actual generated method name from your Section B. The test should confirm:
- The generated client honours an overridden basePath (whatever you pass to `Client::new_with_client`).
- progenitor does (or does not) auto-prepend the spec's `host` + `basePath`.

Capture the finding in Section E of the reference doc.

```rust
#[cfg(test)]
mod sanity {
    use super::*;

    #[tokio::test]
    async fn meta_data_uses_overridden_baseurl() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/meta-data"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let http = reqwest::Client::new();
        let client = Client::new_with_client(&server.uri(), http);
        // Replace `metadata_get` with the actual generated name from the reference doc.
        let _ = client.metadata_get().await.expect("call");
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}
```

Run:
```bash
cd /tmp/progenitor-bss-smoke && cargo test sanity -- --nocapture
```
Expected: `1 passed`.

- [ ] **Step 8: Commit the converted spec + reference doc IN THE CSM-RS REPO**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
git add src/bss/csm_api_docs.openapi3.json docs/superpowers/plans/2026-06-13-progenitor-bss-output-reference.md
git commit -m "chore(bss): commit converted OpenAPI 3.0 spec + progenitor output reference"
```

(If you had to apply JSON patches to make progenitor accept the spec, document them in Section F and stage them in this same commit.)

---

## Phase 1: Build infrastructure

### Task 1: Extend build.rs to also generate the BSS client

**Files:**
- Modify: `build.rs`
- Possibly modify: `Cargo.toml` (only if Section H of the reference doc identified a new dep)
- Possibly modify: `Makefile` (only if it exists and you want a `convert-bss-spec` target — optional)

The existing `build.rs` runs `generate_one` for HSM (JSON) and CFS (YAML). Extend it to also call `generate_one` for BSS (JSON, since Task 0 committed the converted JSON).

- [ ] **Step 1: Read the current build.rs**

```bash
cat build.rs
```

Note the existing two `generate_one` calls: one for HSM (`src/hsm/csm_api_docs.openapi3.json` → `hsm_generated.rs`, `SpecFormat::Json`), one for CFS (`src/cfs/csm_api_docs.yaml` → `cfs_generated.rs`, `SpecFormat::Yaml`).

- [ ] **Step 2: Add a third `generate_one` invocation for BSS**

Edit `build.rs`. Find the line where the CFS `generate_one` call ends. After it, add:
```rust
    // BSS: OpenAPI 3.0 JSON (converted from the upstream Swagger 2.0).
    generate_one(
        &manifest_dir.join("src/bss/csm_api_docs.openapi3.json"),
        &out_dir.join("bss_generated.rs"),
        SpecFormat::Json,
    );
```

The full main() block should now have three `generate_one` calls.

- [ ] **Step 3: Build to exercise all three spec codegens**

```bash
cargo build 2>&1 | tail -10
```
Expected: `Compiling csm-rs (build script)` then `Compiling csm-rs` then `Finished`. The build script runs all three `generate_one` calls; only `hsm_generated.rs` and `cfs_generated.rs` are currently `include!`-d (Task 2 wires in `bss_generated.rs`).

If progenitor panics on the BSS spec at this step, the panic message will say which `generate_one` invocation failed and re-check Task 0's findings.

- [ ] **Step 4: Verify all three generated files exist**

```bash
find target -name 'hsm_generated.rs' -path '*out*' | head -1
find target -name 'cfs_generated.rs' -path '*out*' | head -1
find target -name 'bss_generated.rs' -path '*out*' | head -1
```
Expected: one line each. All three must be present.

- [ ] **Step 5: Handle any new runtime dep (Section H of the reference doc)**

If Section H said the BSS-generated code uses a crate not already in `Cargo.toml` (very unlikely after HSM/CFS), add it now. Otherwise skip this step.

- [ ] **Step 6: Commit**

```bash
git add build.rs
# Stage Cargo.toml only if you actually edited it in Step 5.
git commit -m "feat(bss): extend build.rs to generate BSS client from OpenAPI 3.0 JSON"
```

---

### Task 2: Create `src/bss/generated.rs` and the BSS wrapper skeleton

**Files:**
- Create: `src/bss/generated.rs`
- Create: `src/bss/wrapper/mod.rs`
- Modify: `src/bss/mod.rs`

This task creates the BSS-side analogues of `src/hsm/generated.rs`, `src/hsm/wrapper/mod.rs` and the corresponding CFS files. The structure is intentionally symmetric. Because BSS has only one resource (bootparameters), the wrapper layer does NOT need per-resource sub-modules — everything lives in `src/bss/wrapper/mod.rs`.

- [ ] **Step 1: Create `src/bss/generated.rs`**

```rust
//! progenitor-generated BSS client. Output of `build.rs`, included verbatim.
//!
//! `pub(crate)` because only the wrapper layer in `crate::bss::wrapper`
//! and `types.rs` re-export aliases are allowed to touch the generated
//! symbols. Public consumers go through `ShastaClient`.
#![allow(
  dead_code,
  clippy::all,
  missing_docs,
  non_camel_case_types,
  non_snake_case,
  unused_comparisons,
  unused_imports
)]
include!(concat!(env!("OUT_DIR"), "/bss_generated.rs"));
```

(`unused_comparisons` is included because CFS Task 2 discovered progenitor emits `if value.len() < 0usize` length validators that aren't covered by `clippy::all`. Include it preemptively for BSS — if BSS doesn't emit those, the allow is harmless.)

- [ ] **Step 2: Create `src/bss/wrapper/mod.rs`**

Open `docs/superpowers/plans/2026-06-13-progenitor-bss-output-reference.md`:
- Section D — verify the `Client::new_with_client` signature matches HSM/CFS (it should — progenitor 0.8 emits the same shape).
- Section C — verify the `progenitor_client::Error<E>` variants. Same 8 variants as HSM/CFS.

Write `src/bss/wrapper/mod.rs` as a direct adaptation of `src/hsm/wrapper/mod.rs` (the proven HSM template). The differences from HSM:
- The `generated` module path is `crate::bss::generated` (not `crate::hsm::generated`).
- The baseurl formula is `format!("{}/bss", client.base_url())` (not `format!("{}/smd/hsm/v2", ...)`).
- The error messages mention CFS — adjust them to "BSS".

This file holds BOTH the shared helpers AND the per-method wrappers (no sub-modules needed — there's only one resource):

```rust
//! Thin wrapper bridging the generated BSS client to the public
//! `ShastaClient` API. Mirrors `crate::hsm::wrapper` — see its
//! module-level docs for the design rationale.
//!
//! Responsibilities:
//!  - construct a per-call generated `Client` with bearer auth baked in;
//!  - override the spec's basePath with the URL shape csm-rs has always
//!    used (`{base_url}/bss`);
//!  - map `progenitor_client::Error<T>` into `crate::error::Error` (async,
//!    reads the body for UnexpectedResponse/ErrorResponse — same idiom
//!    as `crate::hsm::wrapper::map_err`).
//!
//! `impl ShastaClient { pub async fn bss_bootparameters_*() }` blocks
//! live in this file directly because BSS has only one resource
//! (bootparameters). For per-method routing decisions (progenitor vs
//! raw `reqwest`), see the doc-comment above each method.

use crate::{ShastaClient, bss::generated, error::Error};

pub(crate) fn gen_client(
    client: &ShastaClient,
    token: &str,
) -> Result<generated::Client, Error> {
    let inner = crate::common::http::build_client_with_auth(
        client.root_cert(),
        client.socks5_proxy(),
        Some(token),
    )?;
    let baseurl = format!("{}/bss", client.base_url());
    Ok(generated::Client::new_with_client(&baseurl, inner))
}

#[allow(clippy::enum_glob_use, clippy::match_same_arms)]
pub(crate) async fn map_err<E: std::fmt::Debug>(
    err: progenitor_client::Error<E>,
) -> Error {
    use progenitor_client::Error::*;
    match err {
        InvalidRequest(s) => Error::Message(format!("BSS invalid request: {s}")),
        CommunicationError(e) => Error::NetError(e),
        InvalidUpgrade(e) => Error::NetError(e),
        ErrorResponse(rv) => {
            let status = rv.status();
            Error::Message(format!(
                "BSS error response: status={status} body={:?}",
                rv.into_inner()
            ))
        }
        ResponseBodyError(e) => Error::NetError(e),
        InvalidResponsePayload(_, e) => Error::SerdeJsonError(e),
        UnexpectedResponse(resp) => {
            let status = resp.status();
            let url = resp.url().clone();
            let body = resp
                .text()
                .await
                .unwrap_or_else(|e| format!("<body read failed: {e}>"));
            Error::Message(format!(
                "BSS unexpected response: status={status} url={url} body={body}"
            ))
        }
        PreHookError(s) => Error::Message(format!("BSS pre-hook error: {s}")),
    }
}

pub(crate) async fn run<F, Fut, T, E>(
    client: &ShastaClient,
    token: &str,
    op: F,
) -> Result<T, Error>
where
    F: FnOnce(generated::Client) -> Fut,
    Fut: std::future::Future<
        Output = Result<
            progenitor_client::ResponseValue<T>,
            progenitor_client::Error<E>,
        >,
    >,
    E: std::fmt::Debug,
{
    let gc = gen_client(client, token)?;
    match op(gc).await {
        Ok(rv) => Ok(rv.into_inner()),
        Err(e) => Err(map_err(e).await),
    }
}

// Task 3 inserts an `impl ShastaClient { ... }` block here with all
// 6 `bss_bootparameters_*` wrapper methods.
```

- [ ] **Step 3: Wire the new modules into `src/bss/mod.rs`**

Open `src/bss/mod.rs` and read it. Add (near other `pub mod` lines):
```rust
pub(crate) mod generated;
mod wrapper;
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build 2>&1 | tail -10
```
Expected: `Finished`. Dead-code warnings on `gen_client`, `map_err`, `run` are expected — Task 3 will wire them in.

If `progenitor_client::Error` variant names don't match your `map_err`, the compiler tells you which arms are wrong; check Section C of the reference doc.

- [ ] **Step 5: Commit**

```bash
git add src/bss/generated.rs src/bss/wrapper/ src/bss/mod.rs
git commit -m "feat(bss): scaffold generated module + wrapper skeleton"
```

---

## Phase 2: Migrate the bootparameters resource

### Task 3: Migrate `bss::bootparameters` (6 methods + decide type strategy)

**Files:**
- Modify: `src/bss/wrapper/mod.rs` (append the `impl ShastaClient { pub async fn bss_bootparameters_*() }` block)
- Modify: `src/bss/types.rs` (decision below)
- Modify: `src/bss/http_client/mod.rs` (most likely: delete the file's content; or downsize to a re-export shim)
- Modify: `src/bss/dispatcher_conv.rs` (only if type strategy choice forces it)
- Delete: `src/bss/http_client/mod.rs` (likely) OR retain as a re-export shim
- Modify: `src/bss/utils.rs` (only if it imports types being changed)

This task is the ONE per-resource migration. Unlike CFS where every per-resource task documented a "0/N routed" outcome due to manta-backend-dispatcher coupling, BSS's `dispatcher_conv.rs` is small (35 lines) and there is only one hand-rolled type (`BootParameters`). That makes full progenitor adoption realistic — but the implementer should make the call based on what the generated `BootParams` type actually looks like vs the hand-written `BootParameters`.

#### Step 1: Inventory the existing public API

```bash
grep -nE "pub async fn bss_bootparameters_" src/bss/http_client/mod.rs
```
Expected 6 entries:
```
bss_bootparameters_get
bss_bootparameters_get_all
bss_bootparameters_get_multiple
bss_bootparameters_put
bss_bootparameters_post
bss_bootparameters_patch
```

For each, read the actual signature. Those signatures must be preserved byte-for-byte by the wrapper file.

#### Step 2: Compare the hand-written `BootParameters` type to progenitor's generated `BootParams`

Read `src/bss/types.rs` lines 16-onwards. Note every field, its type, and any `#[serde(...)]` annotations.

Then look at the generated `BootParams` type in `target/.../bss_generated.rs` (build it first if needed: `cargo build`). Note every field on the generated side.

Compare:
- Field names: do they line up? If progenitor preserves PascalCase, the hand-written struct's `#[serde(rename = "Foo")]` annotations should produce the same wire shape.
- Field types: is the generated version stricter (typed enums, newtypes, `chrono::DateTime`)? Does it use `Option<T>` where the hand-written used `T`?
- Field count: does either side have fields the other lacks?

#### Step 3: DECISION — full type swap vs hand-written retention

Based on Step 2's comparison:

**Choose A — Full type swap (recommended if the shapes are close)**:
- Replace `src/bss/types.rs` content with `pub use crate::bss::generated::types::BootParams as BootParameters;` (adjust the actual generated name).
- Adjust `src/bss/dispatcher_conv.rs` From-impls to work against the generated type. The dispatcher mirror in `manta-backend-dispatcher` is `manta_backend_dispatcher::types::BootParameters` — check its field shape and write conversions.
- This is the migration outcome the BSS plan exists to attempt: real progenitor adoption, not just "move the file".

**Choose B — Keep hand-written types** (only if A would force a breaking public API change OR cascade through dispatcher_conv beyond what's acceptable):
- Leave `src/bss/types.rs` unchanged.
- Document the choice in the wrapper docstring with the specific contractual reason.
- Use the generated types only inside the wrapper file (as request/response shapes for the progenitor calls) and convert at the boundary.

Document the choice clearly in the commit message either way.

#### Step 4: Decide per-method routing

For each of the 6 methods, look at the generated equivalent:
- Does the generated method's signature match the historical csm-rs signature shape? (Both accept the right query params, both return the right type after Step 3's decision.)
- Are there off-spec query params the historical csm-rs sends (e.g. `?limit=100000` as CFS had)?
- Are there tolerant body shape requirements (e.g. `handle_json_or_text_response`)?

Likely candidates:
- **Through progenitor**: all 6 methods if Step 3 chose A. The bootparameters resource is unusually clean — a single struct, no version split, small dispatcher_conv.
- **Stay raw**: any method that the contracts make impossible. Document the WHY in the wrapper docstring with a concrete reason.

#### Step 5: Append the wrapper methods to `src/bss/wrapper/mod.rs`

After the `run` helper, append an `impl ShastaClient { ... }` block with all 6 wrappers. Each method either calls `run(self, token, |c| c.<generated_method>(...))` or copies the existing raw-reqwest implementation from `src/bss/http_client/mod.rs`.

Example shape for a routed method (replace `do_bootparameters_get` with the actual generated name from Section B):

```rust
impl ShastaClient {
    /// `GET /apis/bss/boot/v1/bootparameters` — list/filter boot parameters.
    pub async fn bss_bootparameters_get_all(
        &self,
        token: &str,
    ) -> Result<Vec<BootParameters>, Error> {
        run(self, token, |c| async move {
            c.do_bootparameters_get().await
        }).await
    }
    // … 5 more methods …
}
```

Use `BootParameters` in the public signatures (it's either the type alias from Step 3A, or the hand-written struct from Step 3B — either way the caller-visible name is unchanged).

The module docstring at the top of `src/bss/wrapper/mod.rs` should be updated to list routing decisions per method (which are progenitor, which are raw, why).

#### Step 6: Update `src/bss/http_client/mod.rs`

Option A — DELETE the file: the wrapper now owns the methods. Remove the file with `git rm` and drop any `pub mod http_client;` from `src/bss/mod.rs`.

Option B — re-export shim: keep `src/bss/http_client/mod.rs` as a minimal shim with `pub use super::wrapper::*;` if any external code imports `crate::bss::http_client::*`. Grep `grep -rn "bss::http_client" src/ tests/` to see if removal would break anything.

Pick A unless the grep finds external consumers.

#### Step 7: Fix collateral

Run `cargo build 2>&1` and address every error. Likely sites:
- `src/bss/utils.rs` — may have imports against the old layout.
- `src/bss/dispatcher_conv.rs` — may need updates if Step 3A swapped the types.
- `src/backend_connector/bss.rs` (if it exists) — consumer that goes through the dispatcher.
- `tests/*.rs` — integration tests that hit BSS.

#### Step 8: Build, test, downstream check

```bash
cargo build 2>&1 | tail -10
cargo test --lib 2>&1 | tail -5
cargo test --test shasta_client_hsm 2>&1 | tail -5
cargo test --test backend_connector 2>&1 | tail -5
cargo test --test shasta_client_cfs 2>&1 | tail -5
(cd ../manta && cargo check 2>&1 | tail -5) 2>/dev/null || echo "no ../manta"
```

If a `tests/shasta_client_bss.rs` exists, run it too:
```bash
cargo test --test shasta_client_bss 2>&1 | tail -5
```

Baselines (post-CFS migration): 157 lib / 12 shasta_client_hsm / 32 backend_connector / 12 shasta_client_cfs.

#### Step 9: Commit

```bash
git add # modified + new files
git commit -m "$(printf 'refactor(bss): generate /boot/v1/bootparameters wrapper from progenitor\n\nType strategy: <A: swapped to generated BootParams via pub use alias\nOR B: kept hand-written, conversion at wrapper boundary>.\n\nFully-progenitor-routed methods: <list>.\nKept on raw reqwest with documented rationale: <list>.\n\nBehaviour delta: <if any field-shape changes from the type swap, document them here>.')"
```

---

## Phase 3: Final cleanup and verification

### Task 4: End-to-end verification + module docs

**Files:**
- Modify: `src/bss/mod.rs` (append the "How this module is built" doc section)

- [ ] **Step 1: Confirm no remaining `src/bss/http_client/mod.rs`**

```bash
find src/bss -path '*/http_client/*' -print
```
If Task 3 deleted the file (Option A), this should be empty. If Task 3 retained a shim (Option B), it'll print the shim — that's fine.

- [ ] **Step 2: Full build + test + clippy sweep**

```bash
cargo build 2>&1 | tail -5
cargo test --lib 2>&1 | tail -5
cargo test --test shasta_client_hsm 2>&1 | tail -5
cargo test --test backend_connector 2>&1 | tail -5
cargo test --test shasta_client_cfs 2>&1 | tail -5
cargo clippy --lib 2>&1 | grep -c "warning:" || true
```

If `tests/shasta_client_bss.rs` exists, also run:
```bash
cargo test --test shasta_client_bss 2>&1 | tail -5
```

Baselines (post-CFS migration): 157 lib / 12 shasta_client_hsm / 32 backend_connector / 12 shasta_client_cfs. Confirm no regressions. New clippy warnings from `src/bss/generated.rs` are silenced by its `#![allow(...)]` block; any OTHER module's new warnings must be addressed.

- [ ] **Step 3: Append the codegen pipeline note to `src/bss/mod.rs`**

Open `src/bss/mod.rs`. After the existing module-level `//!` doc, append:

```rust
//!
//! ## How this module is built
//!
//! Wire-format types and the underlying HTTP client surface are
//! generated from `src/bss/csm_api_docs.yaml`. Mirrors the HSM pipeline
//! documented in [`crate::hsm`]: the spec is Swagger 2.0, so it gets
//! converted to OpenAPI 3.0 via `swagger2openapi` and the converted
//! JSON (`src/bss/csm_api_docs.openapi3.json`) is committed alongside
//! the YAML.
//!
//! 1. **Developer step:** `npx swagger2openapi src/bss/csm_api_docs.yaml
//!    -o src/bss/csm_api_docs.openapi3.json` — re-run whenever the YAML
//!    changes; the JSON is committed.
//! 2. `build.rs` runs `progenitor` on the JSON and writes the generated
//!    client to `$OUT_DIR/bss_generated.rs`.
//! 3. `src/bss/generated.rs` `include!`s the file as a `pub(crate)`
//!    module — only the wrapper layer and `types.rs` re-export aliases
//!    are allowed to touch it.
//! 4. `src/bss/wrapper/mod.rs` glues the generated client (and where the
//!    spec/contract drifts, raw `reqwest` calls) to the public
//!    `ShastaClient::bss_*` API. Per-method routing decisions
//!    (progenitor vs raw `reqwest`) are documented in the file's
//!    module docstring.
//!
//! `src/bss/types.rs` holds the public `BootParameters` type. Depending
//! on what the migration in commit history chose:
//! - either it is a `pub use` alias to the progenitor-generated type, or
//! - it is hand-written because a wholesale swap would have cascaded
//!   through `dispatcher_conv` in a way the migration deferred.
```

- [ ] **Step 4: Commit**

```bash
git add src/bss/mod.rs
git commit -m "docs(bss): describe the progenitor-driven codegen pipeline in module docs"
```

- [ ] **Step 5: Verify final git history matches the migration order**

```bash
git log --oneline | head -10
```
Expected: commits from Task 0 → Task 4 in order.

- [ ] **Step 6: Downstream check**

```bash
(cd ../manta && cargo check 2>&1 | tail -5) 2>/dev/null || echo "no ../manta — skip"
```

---

## Self-review notes (kept for the executor)

- **Spec coverage**: The plan only migrates the 6 `bss_bootparameters_*` methods that currently exist in csm-rs's public API. The YAML defines 10+ schemas and 13+ paths; the rest (`/meta-data`, `/user-data`, `/phone-home`, `/boot/v1/bootscript`, `/boot/v1/hosts`, `/boot/v1/dumpstate`, `/boot/v1/endpoint-history`, `/boot/v1/service/*`) are out of scope because csm-rs doesn't expose them. If a future need arises, they can be added as follow-up wrappers; the generated client already has methods for them after Task 1.

- **No placeholders**: The Task 0 reference doc has fill-in markers (the structure the implementer populates by reading the generated file) — same pattern as HSM and CFS Task 0. Subsequent tasks reference the existing source as the authoritative naming source, with the plan's verbatim names as starting points.

- **Type consistency**: All public method names match historical `bss_bootparameters_*` naming. `BootParameters` is the public type name throughout (whether aliased to generated or kept hand-written).

- **Why so few tasks (5 vs HSM's 13, CFS's 10)?**: BSS has one resource, six methods, one hand-rolled type, and a 35-line dispatcher_conv. The migration genuinely is smaller. No per-resource split is needed (no v2/v3, no multiple resources), and the "decide type strategy" beat happens once in Task 3 rather than 6 times like CFS.

- **Why is full progenitor adoption realistic here when CFS landed 0/N?**: BSS lacks the deep `manta-backend-dispatcher` coupling that blocked CFS. Only one hand-written type and only 35 lines of dispatcher_conv. If the generated `BootParams` shape is close to the hand-written `BootParameters`, the swap is a small set of `From` impls — not the 200+-line dispatcher rewrites CFS would have needed. Task 3's Step 3 makes the call based on the actual comparison.

- **Out-of-scope for this plan**: extending `bss::wrapper` to expose `meta-data`, `user-data`, `phone-home`, `bootscript`, `hosts`, or the service endpoints. Their generated methods exist after Task 1 — wrapping them is a follow-up.
