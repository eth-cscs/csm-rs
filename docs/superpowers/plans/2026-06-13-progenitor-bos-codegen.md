# progenitor-based BOS client codegen — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hand-written `src/bos/` HTTP client and wire-format types with code generated from `src/bos/csm_api_docs.yaml` (OpenAPI 3.0.3), preserving the existing public `ShastaClient::bos_*` API and the v1/v2 split.

**Architecture:** Single-step pipeline (no Swagger 2.0 conversion needed — BOS spec is already OpenAPI 3.0.3). `build.rs` runs `progenitor::Generator` on `csm_api_docs.yaml` directly and writes `$OUT_DIR/bos_generated.rs` alongside the existing HSM/CFS/BSS outputs. Output is `include!`-d into a single `pub(crate)` `src/bos/generated.rs` containing only v2 endpoints (the spec has no v1). A thin wrapper layer at `src/bos/wrapper/` maps the 11 existing `bos_*` methods on `ShastaClient` to generated client calls (v2) or to verbatim raw-reqwest re-locations (v1, which has no spec coverage), organised by API version (`wrapper/v1/` and `wrapper/v2/`) so the version boundary is visible in the directory tree.

**Tech Stack:** Rust (edition 2021), `progenitor ~ 0.8` (already pulled in), `openapiv3 ~ 2` (already pulled in), `serde_yaml 0.9` (already pulled in by the CFS work), `reqwest 0.12`, `serde 1`, `tokio 1.45`.

**Reference documents:**
- Source design spec: `docs/superpowers/specs/2026-06-13-progenitor-hsm-codegen-design.md`.
- HSM precedent plan (Swagger 2.0): `docs/superpowers/plans/2026-06-13-progenitor-hsm-codegen.md`.
- CFS precedent plan (OpenAPI 3 native, dual v2/v3 surface): `docs/superpowers/plans/2026-06-13-progenitor-cfs-codegen.md` — **structurally the closest precedent**; the BOS v1/v2 split copies CFS's `wrapper/v2/`/`wrapper/v3/` subfolder pattern.
- BSS precedent plan: `docs/superpowers/plans/2026-06-13-progenitor-bss-codegen.md` — smallest precedent.
- Output reference docs to follow as templates: `docs/superpowers/plans/2026-06-13-progenitor-output-reference.md` (HSM), `2026-06-13-progenitor-cfs-output-reference.md`, `2026-06-13-progenitor-bss-output-reference.md`.
- Memory: `[Partial progenitor migration is OK]` — methods can stay on raw reqwest with documented rationale where generated contracts conflict with real CSM, or where the spec doesn't cover the method at all.

**BOS-specific deltas from HSM / CFS / BSS:**
- **Spec is OpenAPI 3.0.3 natively** (`openapi: "3.0.3"` at line 2). No `swagger2openapi` conversion needed. Spec feeds progenitor directly via `build.rs`.
- **csm-rs has v1 methods that the spec does NOT cover**: `bos_session_v1_post`, `bos_template_v1_get`, `bos_template_v1_post` — total 3 methods. The BOS spec is v2-only; v1 was deprecated upstream. **These v1 methods cannot be progenitor-routed; they migrate as file relocations only.** Task 3 handles them.
- **No `make convert-spec` target needed for BOS** (like CFS) — the YAML feeds progenitor directly.
- **Mid-sized scale**: 2,024-line spec, 79 schemas, ~13 v2 operations, 11 public csm-rs methods (3 v1 + 8 v2). Smaller than CFS, larger than BSS.
- **dispatcher_conv coupling exists only on v2** (`session/http_client/v2/dispatcher_conv.rs` 106 lines, `template/http_client/v2/dispatcher_conv.rs` 120 lines); v1 has no dispatcher_conv. Same "keep hand-written types when dispatcher_conv is large" pattern that CFS established applies — likely outcome is hand-written types stay for v2.
- **`bos_health_check` is a singleton method** (not bound to a `session` or `template` resource) — handle it in `src/bos/wrapper/health_check.rs` (or as a method on the wrapper root, no v1/v2 split).
- **Spec defines new endpoints csm-rs doesn't expose** (`/v2/components`, `/v2/applystaged`, `/v2/options`, `/v2/sessiontemplatetemplate`, `/v2/sessiontemplatesvalid/{...}`, `/v2/sessions/{id}/status`) — out of scope for this plan; generated methods will exist after Task 1 and can be wrapped as follow-ups.

**Type/method name reference (filled in by Task 0):** `docs/superpowers/plans/2026-06-13-progenitor-bos-output-reference.md`. Subsequent migration tasks consult this file because progenitor's exact mangled type/method names are not knowable until the generator runs on the spec.

---

## Phase 0: Feasibility verification (one task, gates everything else)

### Task 0: Run progenitor end-to-end against the BOS spec; capture the name mapping

**Why this exists:** progenitor accepts OpenAPI 3.x natively, so no conversion is required, but the spec may still contain quirks progenitor refuses (HSM needed 3 patches, CFS needed 3, BSS needed 2). The exact Rust type/method names progenitor emits are determined by its mangling rules + the spec's `operationId` and `components.schemas` keys. Nothing in Phase 1+ can be concrete until we have inspected the generated `.rs` file.

**Files:**
- Create: `docs/superpowers/plans/2026-06-13-progenitor-bos-output-reference.md` (committed)
- Possibly modify: `src/bos/csm_api_docs.yaml` (post-conversion patches if progenitor refuses)
- Temporary: `/tmp/progenitor-bos-smoke/` (scratch crate; not committed)

- [ ] **Step 1: Validate the spec parses as OpenAPI 3.0**

Run from repo root:
```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
python3 -c "import yaml; d=yaml.safe_load(open('src/bos/csm_api_docs.yaml')); print(d['openapi']); print('paths:', len(d['paths'])); print('schemas:', len(d['components']['schemas']))"
```
Expected: `3.0.3`, `paths: 15`, `schemas: 79`.

- [ ] **Step 2: Scaffold a throwaway crate**

```bash
mkdir -p /tmp/progenitor-bos-smoke && cd /tmp/progenitor-bos-smoke && cargo init --lib --name progenitor_bos_smoke
cp /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs/src/bos/csm_api_docs.yaml ./spec.yaml
```

Set `/tmp/progenitor-bos-smoke/Cargo.toml`:
```toml
[package]
name = "progenitor_bos_smoke"
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
serde_yaml = "0.9"
prettyplease = "0.2"
syn = "2"

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt"] }
wiremock = "0.6"
```

- [ ] **Step 3: Minimal build.rs for the smoke crate**

Create `/tmp/progenitor-bos-smoke/build.rs`:
```rust
use std::{env, fs, path::PathBuf};

fn main() {
    let spec_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("spec.yaml");
    println!("cargo:rerun-if-changed={}", spec_path.display());

    let src = fs::read_to_string(&spec_path).expect("read spec");
    let spec: openapiv3::OpenAPI =
        serde_yaml::from_str(&src).expect("spec.yaml is not valid OpenAPI 3.0");

    let mut generator = progenitor::Generator::default();
    let tokens = generator.generate_tokens(&spec).expect("progenitor codegen");
    let ast: syn::File = syn::parse2(tokens).expect("parse generated tokens");
    let pretty = prettyplease::unparse(&ast);

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("generated.rs");
    fs::write(&out, pretty).expect("write generated.rs");
}
```

Replace `/tmp/progenitor-bos-smoke/src/lib.rs`:
```rust
#![allow(dead_code, clippy::all)]
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
```

- [ ] **Step 4: Run the smoke build**

```bash
cd /tmp/progenitor-bos-smoke && cargo build 2>&1 | tail -60
```
Expected: succeeds. Known failure modes (same as HSM/CFS/BSS Task 0):
- `progenitor::Generator::generate_tokens` panics → patch the YAML (commit patches in the same Task 0 commit).
- `syn::parse2` fails → same; spec issue surfacing through codegen.

If the build fails after up to a day of patching, STOP and report BLOCKED.

- [ ] **Step 5: Inspect the generated code and capture the type/method name reference**

Locate:
```bash
find /tmp/progenitor-bos-smoke/target -name generated.rs -path '*build*' | head -1
```
Open it. Create `docs/superpowers/plans/2026-06-13-progenitor-bos-output-reference.md` IN THE CSM-RS REPO with the same A-H structure as the HSM/CFS/BSS reference docs. Use the BSS reference doc (`docs/superpowers/plans/2026-06-13-progenitor-bss-output-reference.md`) as the closest structural template.

Specifically populate:

**A. Generated type names (by YAML schema name)** — 79 schemas total. The plan's tasks need at minimum these (look up each in the generated file):
```
V2SessionTemplate
V2Session
V2SessionStatus
V2SessionCreateResponse
V2SessionTemplateValidationStatus
V2Component
V2ComponentArray
V2ApplyStaged
V2Options
HealthCheckResponse
Version
```
List others if encountered.

**A.1. Schemas referenced by the plan that are NOT generated** — empty list if none.

**B. Generated method names (by operationId)** — grep `operationId:` in `src/bos/csm_api_docs.yaml`. For each one, find the corresponding generated Rust method name. Focus on the 8 v2 operations that csm-rs actually uses today:
```
sessiontemplates list (GET /v2/sessiontemplates)
sessiontemplate get (GET /v2/sessiontemplates/{session_template_id})
sessiontemplate put (PUT /v2/sessiontemplates/{session_template_id})
sessiontemplate delete (DELETE /v2/sessiontemplates/{session_template_id})
sessions list (GET /v2/sessions)
session post (POST /v2/sessions)
session delete (DELETE /v2/sessions/{session_id})
healthz (GET /v2/healthz)
```
List the others under a "not in csm-rs public API" subsection.

**C. progenitor `Error` enum variants** — same 8 as HSM/CFS/BSS; copy verbatim from the generated file or from progenitor-client 0.8.

**D. `Client::new` / `Client::new_with_client` signatures** — copy from the generated file.

**E. basePath behaviour** — BOS spec uses `servers: ...`. Look up the actual server URL. csm-rs's `base_url` includes `/apis`. So the wrapper override would be `format!("{}/bos", client.base_url())` and operation paths supply the `/v2/...` prefixes. Verify with a wiremock test (Step 6).

**F. Spec patches applied** — describe any YAML edits needed. If none, write "None required — the OpenAPI 3.0.3 spec was accepted as-is."

**G. Generated artefact stats** — total lines, struct count, enum count, method count.

**H. Runtime crate deps** — verify whether BOS-generated code needs any deps that aren't already in csm-rs's `Cargo.toml`. After HSM/CFS/BSS migrations csm-rs already has `progenitor-client = "0.8"`, `regress = "0.10"`, `chrono` with `serde`, `uuid` with `serde`. Verify by grepping the generated file for `chrono::`, `uuid::`, `regress::`. Likely no new deps.

- [ ] **Step 6: URL/basePath sanity check**

Add a wiremock test to `/tmp/progenitor-bos-smoke/src/lib.rs` hitting a small operation (e.g. `GET /v2/healthz` or `GET /v2/version`). Look up the actual generated method name from your Section B. The test should confirm:
- The generated client honours an overridden basePath (whatever you pass to `Client::new_with_client`).
- progenitor does (or does not) auto-prepend the spec's `servers[0].url`.

Capture the finding in Section E of the reference doc.

```rust
#[cfg(test)]
mod sanity {
    use super::*;

    #[tokio::test]
    async fn healthz_uses_overridden_baseurl() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/v2/healthz"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let http = reqwest::Client::new();
        let client = Client::new_with_client(&server.uri(), http);
        // Replace `get_healthz_v2` with the actual generated name from the reference doc.
        let _ = client.get_healthz_v2().await.expect("call");
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}
```

Run:
```bash
cd /tmp/progenitor-bos-smoke && cargo test sanity -- --nocapture
```
Expected: `1 passed`.

- [ ] **Step 7: Commit the reference doc IN THE CSM-RS REPO**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
git add docs/superpowers/plans/2026-06-13-progenitor-bos-output-reference.md
# If you had to apply YAML patches to make progenitor accept the spec, also stage src/bos/csm_api_docs.yaml.
git commit -m "chore(bos): commit progenitor output reference for BOS spec"
```

---

## Phase 1: Build infrastructure

### Task 1: Extend build.rs to also generate the BOS client

**Files:**
- Modify: `build.rs`

The existing `build.rs` runs `generate_one` for HSM (JSON), CFS (YAML), and BSS (JSON). Extend it to ALSO call `generate_one` for BOS (YAML, since the BOS spec is OpenAPI 3.0.3 YAML).

- [ ] **Step 1: Read the current build.rs**

```bash
cat build.rs
```

Note the existing three `generate_one` calls.

- [ ] **Step 2: Add a fourth `generate_one` invocation for BOS**

Edit `build.rs`. Find where the BSS `generate_one` call ends. After it, add:
```rust
    // BOS: OpenAPI 3.0.3 YAML (upstream-tracked directly).
    generate_one(
        &manifest_dir.join("src/bos/csm_api_docs.yaml"),
        &out_dir.join("bos_generated.rs"),
        SpecFormat::Yaml,
    );
```

The main() block should now have four `generate_one` calls.

- [ ] **Step 3: Build to exercise all four spec codegens**

```bash
cargo build 2>&1 | tail -10
```
Expected: `Compiling csm-rs (build script)` → `Compiling csm-rs` → `Finished`. The build script runs all four; only `hsm_generated.rs`, `cfs_generated.rs`, `bss_generated.rs` are currently `include!`-d (Task 2 wires in `bos_generated.rs`).

If progenitor panics on the BOS spec, the panic says which `generate_one` failed; re-check Task 0's findings.

- [ ] **Step 4: Verify all four generated files exist**

```bash
find target -name 'hsm_generated.rs' -path '*out*' | head -1
find target -name 'cfs_generated.rs' -path '*out*' | head -1
find target -name 'bss_generated.rs' -path '*out*' | head -1
find target -name 'bos_generated.rs' -path '*out*' | head -1
```
All four must be present.

- [ ] **Step 5: Commit**

```bash
git add build.rs
git commit -m "feat(bos): extend build.rs to generate BOS client from OpenAPI 3.0 YAML"
```

---

### Task 2: Create `src/bos/generated.rs` and the BOS wrapper skeleton

**Files:**
- Create: `src/bos/generated.rs`
- Create: `src/bos/wrapper/mod.rs` — shared `gen_client` / `map_err` / `run` + `mod v1; mod v2; mod health_check;`
- Create: `src/bos/wrapper/v1/mod.rs` — empty placeholder; per-resource modules added by Task 3
- Create: `src/bos/wrapper/v2/mod.rs` — empty placeholder; per-resource modules added by Tasks 4-5
- Modify: `src/bos/mod.rs`

This task creates the BOS-side analogues of `src/cfs/generated.rs` + `src/cfs/wrapper/mod.rs` + `wrapper/v2/mod.rs` + `wrapper/v3/mod.rs`. The structure is symmetric to CFS.

- [ ] **Step 1: Create `src/bos/generated.rs`**

```rust
//! progenitor-generated BOS client. Output of `build.rs`, included verbatim.
//!
//! `pub(crate)` because only the wrapper layer in `crate::bos::wrapper`
//! and per-resource `types.rs` re-export aliases are allowed to touch
//! the generated symbols. Public consumers go through `ShastaClient`.
#![allow(
  dead_code,
  clippy::all,
  missing_docs,
  non_camel_case_types,
  non_snake_case,
  unused_comparisons,
  unused_imports
)]
include!(concat!(env!("OUT_DIR"), "/bos_generated.rs"));
```

- [ ] **Step 2: Create `src/bos/wrapper/mod.rs`**

Open `docs/superpowers/plans/2026-06-13-progenitor-bos-output-reference.md`:
- Section D — verify `Client::new_with_client` signature.
- Section C — verify `progenitor_client::Error<E>` variants.

Write `src/bos/wrapper/mod.rs` modelled on `src/cfs/wrapper/mod.rs`. The only differences from CFS:
- The `generated` module is `crate::bos::generated`.
- The baseurl formula is `format!("{}/bos", client.base_url())`.
- Error messages mention "BOS".

```rust
//! Thin wrapper bridging the generated BOS client to the public
//! `ShastaClient` API. Mirrors `crate::hsm::wrapper` — see its
//! module-level docs for the design rationale.
//!
//! Responsibilities:
//!  - construct a per-call generated `Client` with bearer auth baked in;
//!  - override the spec's basePath with the URL shape csm-rs has always
//!    used (`{base_url}/bos` — v2 prefixes come from operation paths);
//!  - map `progenitor_client::Error<T>` into `crate::error::Error`.
//!
//! Per-resource wrapper files (`v2/session.rs`, `v2/template.rs`,
//! `health_check.rs`, etc.) hold `impl ShastaClient { pub async fn
//! bos_*() }` blocks that delegate to the generated client via the
//! `run` adapter. v1 wrappers live in `v1/` and are pure raw-reqwest
//! file relocations — the upstream BOS spec is v2-only, so v1 cannot
//! be routed through progenitor.

use crate::{ShastaClient, bos::generated, error::Error};

pub(crate) fn gen_client(
    client: &ShastaClient,
    token: &str,
) -> Result<generated::Client, Error> {
    let inner = crate::common::http::build_client_with_auth(
        client.root_cert(),
        client.socks5_proxy(),
        Some(token),
    )?;
    let baseurl = format!("{}/bos", client.base_url());
    Ok(generated::Client::new_with_client(&baseurl, inner))
}

#[allow(clippy::enum_glob_use, clippy::match_same_arms)]
pub(crate) async fn map_err<E: std::fmt::Debug>(
    err: progenitor_client::Error<E>,
) -> Error {
    use progenitor_client::Error::*;
    match err {
        InvalidRequest(s) => Error::Message(format!("BOS invalid request: {s}")),
        CommunicationError(e) => Error::NetError(e),
        InvalidUpgrade(e) => Error::NetError(e),
        ErrorResponse(rv) => {
            let status = rv.status();
            Error::Message(format!(
                "BOS error response: status={status} body={:?}",
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
                "BOS unexpected response: status={status} url={url} body={body}"
            ))
        }
        PreHookError(s) => Error::Message(format!("BOS pre-hook error: {s}")),
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

mod v1;
mod v2;
mod health_check;
```

- [ ] **Step 3: Create `src/bos/wrapper/v1/mod.rs`**

```rust
//! `manta`-facing BOS v1 wrapper methods. The upstream BOS spec is
//! v2-only — these are kept on raw `reqwest` and migrate as pure file
//! relocations. Per-resource sub-modules (`session`, `template`) attach
//! `impl ShastaClient { pub async fn bos_<resource>_v1_*() }` blocks
//! to the public client.
//!
//! See `crate::bos::wrapper` for the shared `gen_client` / `map_err`
//! / `run` helpers — they're version-agnostic but v1 methods do not
//! use them (no spec to drive progenitor against).

// Per-resource modules are added by Task 3:
//   mod session;
//   mod template;
```

- [ ] **Step 4: Create `src/bos/wrapper/v2/mod.rs`**

```rust
//! `manta`-facing BOS v2 wrapper methods. Per-resource sub-modules
//! (`session`, `template`) attach
//! `impl ShastaClient { pub async fn bos_<resource>_v2_*() }` blocks
//! to the public client. Each sub-module's docstring records the
//! per-method routing decision (generated client vs raw reqwest).
//!
//! See `crate::bos::wrapper` for the shared `gen_client` / `map_err`
//! / `run` helpers.

// Per-resource modules are added by Tasks 4-5:
//   mod session;
//   mod template;
```

- [ ] **Step 5: Create `src/bos/wrapper/health_check.rs`** (a placeholder for now — Task 6 fills it in)

```rust
//! Wrapper for `GET /v2/healthz` and `GET /v2/version`. Replaces
//! `src/bos/health_check.rs`.
//!
//! Task 6 inserts the `impl ShastaClient` block here.
```

- [ ] **Step 6: Wire the new modules into `src/bos/mod.rs`**

Open `src/bos/mod.rs` and read it. Add (near other `pub mod` lines):
```rust
pub(crate) mod generated;
mod wrapper;
```

- [ ] **Step 7: Verify it compiles**

```bash
cargo build 2>&1 | tail -10
```
Expected: `Finished`. Dead-code warnings expected on `gen_client`, `map_err`, `run`. If `progenitor_client::Error` variant names don't match, the compiler tells you which arms are wrong; check Section C of the reference doc.

- [ ] **Step 8: Commit**

```bash
git add src/bos/generated.rs src/bos/wrapper/ src/bos/mod.rs
git commit -m "feat(bos): scaffold generated module + wrapper skeleton (v1/v2 split)"
```

---

## Phase 2: Per-resource migrations

For every migration task below, the same rhythm applies:
1. Read the existing `src/bos/<resource>/http_client/v{N}/mod.rs` to capture each `pub async fn` signature byte-for-byte — these are the canonical public API names. The plan's verbatim names are sketches; the existing source wins.
2. Look up each generated method/type name in `docs/superpowers/plans/2026-06-13-progenitor-bos-output-reference.md`.
3. For v2 methods: decide for each whether to route through progenitor's generated client OR keep on raw `reqwest` with documented rationale in the wrapper module docstring. The partial-migration policy applies — both choices are acceptable, but every non-wrap needs a concrete WHY.
4. For v1 methods: route is impossible (no spec coverage). Keep on raw reqwest. The wrapper file is a pure file relocation.
5. Replace `*/http_client/v{N}/types.rs` with pure `pub use` re-exports of generated types, UNLESS doing so would cascade through `dispatcher_conv.rs`. If it would, keep hand-written types (HSM/CFS/BSS precedent).
6. Delete the old `http_client/v{N}/mod.rs` after the wrapper is in place.
7. `cargo build && cargo test --lib`.
8. Commit.

### Task 3: Migrate `bos::session` v1 + `bos::template` v1 (3 methods total — pure file relocations)

**Why combined?** v1 is csm-rs-only (not in the spec). Both v1 wrappers are pure raw-reqwest file relocations with no generated client to consult and no per-method routing decisions. Combining them into one task keeps the v1 work tight (one commit instead of two).

**Files:**
- Create: `src/bos/wrapper/v1/session.rs`
- Create: `src/bos/wrapper/v1/template.rs`
- Modify: `src/bos/wrapper/v1/mod.rs` (add `mod session; mod template;`)
- Modify: `src/bos/session/http_client/mod.rs` (drop the `pub mod v1;` line; keep `pub use v1::types::*;` if present)
- Modify: `src/bos/template/http_client/mod.rs` (same)
- Delete: `src/bos/session/http_client/v1/mod.rs`
- Delete: `src/bos/template/http_client/v1/mod.rs`

#### Step 1: Inventory the existing v1 methods

```bash
grep -nE "pub async fn bos_session_v1_|pub async fn bos_template_v1_" \
    src/bos/session/http_client/v1/mod.rs \
    src/bos/template/http_client/v1/mod.rs
```
Expected 3 entries:
```
bos_session_v1_post
bos_template_v1_get
bos_template_v1_post
```

#### Step 2: Create the session v1 wrapper file

Copy the existing implementation of `bos_session_v1_post` from `src/bos/session/http_client/v1/mod.rs` into a new file `src/bos/wrapper/v1/session.rs`. Update the imports as needed (the file is now under `crate::bos::wrapper::v1::session`, not `crate::bos::session::http_client::v1`).

Module docstring should explain:
- v1 has no spec coverage; pure raw-reqwest file relocation.
- Lists the methods present (just `bos_session_v1_post`).

#### Step 3: Create the template v1 wrapper file

Same pattern. Copy both `bos_template_v1_get` and `bos_template_v1_post` from `src/bos/template/http_client/v1/mod.rs` into `src/bos/wrapper/v1/template.rs`.

#### Step 4: Decide types.rs strategy for v1

v1 has no `dispatcher_conv.rs`. The hand-written types in `src/bos/template/http_client/v1/types.rs` are small. They can either:
- Stay as-is (just adjust `src/bos/template/http_client/mod.rs` to keep the `pub mod v1 { pub mod types; }` declaration), or
- Move to a flatter location under the wrapper layer (`src/bos/wrapper/v1/template_types.rs`).

Easier and aligned with the CFS precedent: keep types where they are; just adjust how the parent `http_client/mod.rs` declares them.

#### Step 5: Register the wrapper files

In `src/bos/wrapper/v1/mod.rs`, replace the commented-out placeholders with:
```rust
mod session;
mod template;
```

#### Step 6: Update parent `http_client/mod.rs` declarations

In `src/bos/session/http_client/mod.rs`: drop the `pub mod v1;` line OR change it to a parent-declares-children form keeping only the surviving sub-items (`pub(crate) mod v1 { pub(crate) mod types; }`). If there are no `types.rs` files under v1 for session (likely — v1 session has no types.rs per the inventory), just delete `pub mod v1;` outright.

In `src/bos/template/http_client/mod.rs`: same, but template v1 DOES have `types.rs` — so the form is `pub(crate) mod v1 { pub(crate) mod types; }`.

#### Step 7: Delete the old http client files

```bash
git rm src/bos/session/http_client/v1/mod.rs
git rm src/bos/template/http_client/v1/mod.rs
```

#### Step 8: Fix collateral

Run `cargo build 2>&1` and fix every error. Likely sites:
- `src/bos/session/mod.rs` / `src/bos/template/mod.rs` — re-exports of the v1 surface.
- `src/backend_connector/bos.rs` (if it exists) — consumer through the dispatcher.
- Anywhere the v1 types were imported.

#### Step 9: Build and test

```bash
cargo build 2>&1 | tail -5
cargo test --lib 2>&1 | tail -5
cargo test --test shasta_client_hsm 2>&1 | tail -5
cargo test --test backend_connector 2>&1 | tail -5
cargo test --test shasta_client_cfs 2>&1 | tail -5
cargo test --test shasta_client_misc 2>&1 | tail -5
(cd ../manta && cargo check 2>&1 | tail -5) 2>/dev/null || echo "no ../manta"
```
Baselines (post-BSS migration): 157 lib / 12 shasta_client_hsm / 32 backend_connector / 12 shasta_client_cfs / 7 misc. Confirm no regressions.

#### Step 10: Commit

```bash
git add src/bos/wrapper/v1/session.rs src/bos/wrapper/v1/template.rs \
        src/bos/wrapper/v1/mod.rs \
        src/bos/session/http_client/mod.rs src/bos/template/http_client/mod.rs \
        # plus any v1/types.rs files left in place but moved under inline declarations
git commit -m "$(printf 'refactor(bos): relocate v1 session+template into wrapper layer (raw reqwest)\n\nThe upstream BOS spec is v2-only; v1 is csm-rs-specific and cannot be\nrouted through progenitor. These migrate as pure file relocations.')"
```

---

### Task 4: Migrate `bos::session` v2 (3 methods)

**Files:**
- Create: `src/bos/wrapper/v2/session.rs`
- Modify: `src/bos/wrapper/v2/mod.rs` (add `mod session;`)
- Modify: `src/bos/session/http_client/v2/types.rs` (likely UNCHANGED — keep hand-written)
- Modify: `src/bos/session/http_client/mod.rs` (drop the `pub mod v2;` line and replace with `pub(crate) mod v2 { pub(crate) mod types; #[cfg(feature = "manta-dispatcher")] mod dispatcher_conv; }`)
- Delete: `src/bos/session/http_client/v2/mod.rs`

#### Step 1: Inventory the existing v2 session methods

```bash
grep -nE "pub async fn bos_session_v2_" src/bos/session/http_client/v2/mod.rs
```
Expected 3 entries:
```
bos_session_v2_post
bos_session_v2_get
bos_session_v2_delete
```

For each, read the actual signature.

#### Step 2: Check the dispatcher_conv coupling

```bash
wc -l src/bos/session/http_client/v2/dispatcher_conv.rs
```
Expected ~106 lines (per the pre-survey). CFS/BSS precedent applies: keep hand-written types in `v2/types.rs`, route methods through progenitor where the contracts fit, document why each non-wrap stays on raw.

#### Step 3: Decide per-method routing

For each `bos_session_v2_*` method, look at the generated equivalent (Section B of the reference doc):
- Does the generated method's signature match the historical csm-rs signature?
- Are there off-spec query params, tolerant body shapes, status-code expectations the generated client doesn't match?

#### Step 4: Create the wrapper file

Create `src/bos/wrapper/v2/session.rs`. Module docstring lists per-method routing rationale (concrete reasons, not vague). For raw-reqwest methods, copy the existing implementation verbatim from `src/bos/session/http_client/v2/mod.rs` and update imports.

Use the `crate::bos::wrapper::run` adapter for data-returning methods routed through progenitor.

#### Step 5: types.rs strategy

Keep hand-written (dispatcher_conv coupling justifies it, per CFS/BSS precedent).

#### Step 6: Register the wrapper file

In `src/bos/wrapper/v2/mod.rs`, replace the commented-out placeholder with:
```rust
mod session;
```

#### Step 7: Update `src/bos/session/http_client/mod.rs`

Drop the `pub mod v2;` line. Replace with:
```rust
pub(crate) mod v2 {
    pub(crate) mod types;
    #[cfg(feature = "manta-dispatcher")]
    mod dispatcher_conv;
}
```

(Same pattern Task 3 of the CFS plan established.)

#### Step 8: Delete the old http client file

```bash
git rm src/bos/session/http_client/v2/mod.rs
```

#### Step 9: Fix collateral

Run `cargo build 2>&1` and address every error.

#### Step 10: Build and test

```bash
cargo build 2>&1 | tail -5
cargo test --lib 2>&1 | tail -5
cargo test --test shasta_client_hsm 2>&1 | tail -5
cargo test --test backend_connector 2>&1 | tail -5
cargo test --test shasta_client_cfs 2>&1 | tail -5
cargo test --test shasta_client_misc 2>&1 | tail -5
(cd ../manta && cargo check 2>&1 | tail -5) 2>/dev/null || echo "no ../manta"
```

#### Step 11: Commit

```bash
git add src/bos/wrapper/v2/session.rs src/bos/wrapper/v2/mod.rs \
        src/bos/session/http_client/v2/types.rs \
        src/bos/session/http_client/mod.rs
git commit -m "$(printf 'refactor(bos): generate /v2/sessions wrapper from progenitor\n\nFully-progenitor-routed methods: <list>.\nKept on raw reqwest with documented rationale: <list>.')"
```

---

### Task 5: Migrate `bos::template` v2 (4 methods)

**Files:**
- Create: `src/bos/wrapper/v2/template.rs`
- Modify: `src/bos/wrapper/v2/mod.rs` (add `mod template;`)
- Modify: `src/bos/template/http_client/v2/types.rs` (likely UNCHANGED)
- Modify: `src/bos/template/http_client/mod.rs` (`pub(crate) mod v2 { pub(crate) mod types; #[cfg(feature = "manta-dispatcher")] mod dispatcher_conv; }`)
- Delete: `src/bos/template/http_client/v2/mod.rs`
- Possibly: update `src/bos/template/utils.rs` if it imports v2 items by path.

#### Step 1: Inventory the existing v2 template methods

```bash
grep -nE "pub async fn bos_template_v2_" src/bos/template/http_client/v2/mod.rs
```
Expected 4 entries:
```
bos_template_v2_get
bos_template_v2_get_all
bos_template_v2_put
bos_template_v2_delete
```

#### Step 2: Check dispatcher_conv coupling

```bash
wc -l src/bos/template/http_client/v2/dispatcher_conv.rs
```
Expected ~120 lines.

#### Step 3: Decide per-method routing

For each `bos_template_v2_*` method, evaluate the contract match against Section B of the reference doc. Likely outcomes:
- `bos_template_v2_get` (single ID) — clean candidate IF return type aligns.
- `bos_template_v2_get_all` — list endpoint; check filter parameters and return shape.
- `bos_template_v2_put` — body shape sensitivity.
- `bos_template_v2_delete` — typically `Result<(), Error>`; cleanest candidate.

#### Step 4: Create the wrapper file

Create `src/bos/wrapper/v2/template.rs`. Module docstring lists per-method routing rationale. For raw-reqwest methods, copy the existing implementation verbatim.

#### Step 5: types.rs strategy

Keep hand-written (dispatcher_conv ~120 lines justifies it).

#### Step 6: Register the wrapper file

In `src/bos/wrapper/v2/mod.rs`: add `mod template;`

#### Step 7: Update `src/bos/template/http_client/mod.rs`

Drop the `pub mod v2;` line and replace with the inline `pub(crate) mod v2 { pub(crate) mod types; #[cfg(feature = "manta-dispatcher")] mod dispatcher_conv; }` form.

#### Step 8: Delete the old http client file

```bash
git rm src/bos/template/http_client/v2/mod.rs
```

#### Step 9: Fix collateral (especially `src/bos/template/utils.rs`)

Run `cargo build 2>&1` and address every error.

#### Step 10: Build and test

Same battery as Task 4.

#### Step 11: Commit

```bash
git add src/bos/wrapper/v2/template.rs src/bos/wrapper/v2/mod.rs \
        src/bos/template/http_client/v2/types.rs \
        src/bos/template/http_client/mod.rs \
        # plus utils.rs if changed
git commit -m "$(printf 'refactor(bos): generate /v2/sessiontemplates wrapper from progenitor\n\nFully-progenitor-routed methods: <list>.\nKept on raw reqwest with documented rationale: <list>.')"
```

---

### Task 6: Migrate `bos_health_check` (1 method)

**Files:**
- Modify: `src/bos/wrapper/health_check.rs` (replace placeholder with full implementation)
- Delete: `src/bos/health_check.rs`
- Modify: `src/bos/mod.rs` (drop `pub mod health_check;` if present)

#### Step 1: Inventory the existing method

```bash
grep -nE "pub async fn bos_health_check" src/bos/health_check.rs
```
Expected 1 entry: `bos_health_check`. Read the signature.

#### Step 2: Decide routing

The generated `get_healthz_v2` (or whatever Section B says the name is) returns whatever the spec defines as the health-check response. If it matches the existing csm-rs return type, route via progenitor. If not, copy the existing implementation verbatim.

The existing `bos_health_check` returns `Result<serde_json::Value, Error>` per the pre-survey (line 15 of the file). If the generated method returns a typed struct, choose:
- **Route + convert at boundary**: route through `run` and then `serde_json::to_value(rv).map_err(Error::SerdeJsonError)?` on the way out so the public signature stays `Result<Value, Error>`.
- **Stay raw**: copy verbatim if the typed return is awkward.

#### Step 3: Write `src/bos/wrapper/health_check.rs`

Replace the placeholder file from Task 2 with the actual implementation. Module docstring documents the routing decision.

Example (adjust generated method name from Section B):
```rust
//! Wrapper for `GET /v2/healthz`. Replaces `src/bos/health_check.rs`.
//!
//! Routing: <progenitor via run() OR raw reqwest>, see decision below.

use crate::{ShastaClient, error::Error};

use super::run;

impl ShastaClient {
    /// `GET /apis/bos/v2/healthz` — BOS service health check.
    pub async fn bos_health_check(
        &self,
        token: &str,
    ) -> Result<serde_json::Value, Error> {
        // Routing-A: through progenitor + boundary conversion.
        let typed = run(self, token, |c| async move { c.get_healthz_v2().await }).await?;
        serde_json::to_value(typed).map_err(Error::SerdeJsonError)
        // OR Routing-B: copy the existing implementation here.
    }
}
```

#### Step 4: Delete `src/bos/health_check.rs`

```bash
git rm src/bos/health_check.rs
```

#### Step 5: Update `src/bos/mod.rs`

Drop the `pub mod health_check;` line. The wrapper takes over.

#### Step 6: Build and test

Same battery as Task 4.

#### Step 7: Commit

```bash
git add src/bos/wrapper/health_check.rs src/bos/mod.rs
git commit -m "refactor(bos): relocate bos_health_check into wrapper layer"
```

---

## Phase 3: Final cleanup and verification

### Task 7: End-to-end verification + module docs

**Files:**
- Modify: `src/bos/mod.rs` (append the "How this module is built" doc section)

- [ ] **Step 1: Confirm there are no remaining `*/http_client/v{1,2}/mod.rs` files**

```bash
find src/bos -path '*/http_client/v*/mod.rs' -print
```
Expected: empty (each migrated). If anything is left, an earlier task missed it.

- [ ] **Step 2: Full build + test + clippy sweep**

```bash
cargo build 2>&1 | tail -5
cargo test --lib 2>&1 | tail -5
cargo test --test shasta_client_hsm 2>&1 | tail -5
cargo test --test backend_connector 2>&1 | tail -5
cargo test --test shasta_client_cfs 2>&1 | tail -5
cargo test --test shasta_client_misc 2>&1 | tail -5
cargo clippy --lib 2>&1 | grep -c "warning:" || true
```

Baselines (post-BSS migration): 157 lib / 12 shasta_client_hsm / 32 backend_connector / 12 shasta_client_cfs / 7 misc. Confirm no regressions. New clippy warnings from `src/bos/generated.rs` are silenced by `#![allow(...)]`.

- [ ] **Step 3: Append the codegen pipeline note to `src/bos/mod.rs`**

After the existing module-level `//!` doc, append:

```rust
//!
//! ## How this module is built
//!
//! Wire-format types and the underlying HTTP client surface are
//! generated from `src/bos/csm_api_docs.yaml` (OpenAPI 3.0.3). Mirrors
//! the HSM pipeline documented in [`crate::hsm`]; the only structural
//! difference is that the BOS spec is OpenAPI 3.x natively (no
//! Swagger 2.0 conversion needed), so there is no `make convert-spec`
//! step for BOS.
//!
//! 1. `build.rs` runs `progenitor` on the YAML and writes the
//!    generated client to `$OUT_DIR/bos_generated.rs`.
//! 2. `src/bos/generated.rs` `include!`s the file as a `pub(crate)`
//!    module — only the wrapper layer and `types.rs` re-export aliases
//!    are allowed to touch it.
//! 3. `src/bos/wrapper/` glues the generated client (and where the
//!    spec/contract drifts, raw `reqwest` calls) to the public
//!    `ShastaClient::bos_*` API. Per-method routing decisions
//!    (progenitor vs raw `reqwest`) are documented in each per-resource
//!    file's module docstring. The wrapper is split into `v1/` (raw
//!    reqwest only — the spec is v2-only) and `v2/` subfolders so the
//!    API-version boundary is visible in the directory tree.
//!
//! Per-resource `types.rs` files are hand-rolled (not pure re-exports
//! of generated types) where a full swap would cascade through
//! `dispatcher_conv` bridges (`session/http_client/v2/dispatcher_conv.rs`
//! and `template/http_client/v2/dispatcher_conv.rs`). The generated
//! client is wired up and ready, but per-method progenitor routing is
//! deferred for the methods where the cost-of-swap outweighs the
//! benefit (same pattern as the CFS and BSS migrations).
```

- [ ] **Step 4: Commit**

```bash
git add src/bos/mod.rs
git commit -m "docs(bos): describe the progenitor-driven codegen pipeline in module docs"
```

- [ ] **Step 5: Verify final git history matches the migration order**

```bash
git log --oneline | head -12
```
Expected: BOS Task 0 → Task 7 commits in order.

- [ ] **Step 6: Downstream check**

```bash
(cd ../manta && cargo check 2>&1 | tail -5) 2>/dev/null || echo "no ../manta — skip"
```

---

## Self-review notes (kept for the executor)

- **Spec coverage**: The plan migrates the 11 existing `bos_*` methods (3 v1 + 8 v2). The spec defines additional v2 endpoints csm-rs does NOT expose (`/v2/components`, `/v2/applystaged`, `/v2/options`, `/v2/sessiontemplatetemplate`, `/v2/sessiontemplatesvalid/{id}`, `/v2/sessions/{id}/status`, `/v2/version`). These are out of scope; the generated client has methods for them after Task 1, and wrapping them is a follow-up.

- **No placeholders**: The Task 0 reference doc has fill-in markers — same pattern as HSM/CFS/BSS Task 0. Subsequent tasks reference the existing source as authoritative naming, with the plan's verbatim names as starting points.

- **Type consistency**: All public method names match historical `bos_*` naming. The v1/v2 split in csm-rs is preserved by the `wrapper/v1/`/`wrapper/v2/` directory structure.

- **Why does v1 need migration if it can't be progenitor-routed?**: Same goal as the rest of the pattern — consolidate the wrapper layer so the public `ShastaClient::bos_*` methods all live under `src/bos/wrapper/` instead of `src/bos/<resource>/http_client/`. The v1 wrappers are file moves, not progenitor adoption.

- **Why combine v1 session + template into one task but split v2 into two tasks?**: v1 has 3 methods total with no per-method routing decisions to make (none can route). v2 has 7 methods with real per-method decisions on each. One commit per v2 resource keeps the review surface manageable.

- **Why `health_check` is its own task**: it's not bound to `session` or `template`; it's a singleton method that needs a placeholder slot in Task 2 (`src/bos/wrapper/health_check.rs`) and a real implementation in Task 6. Keeping it separate avoids confusion about the v1/v2 split (it's a `/v2/healthz` route, but it doesn't fit the `wrapper/v2/<resource>.rs` pattern).

- **Out of scope for this plan**: extending `bos::wrapper` to expose the new endpoints (`components`, `applystaged`, `options`, etc.). Generated methods for them exist after Task 1 — wrapping them is a follow-up.
