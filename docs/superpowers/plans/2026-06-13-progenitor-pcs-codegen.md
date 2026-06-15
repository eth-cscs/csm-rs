# progenitor-based PCS client codegen — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hand-written `src/pcs/` HTTP client and wire-format types with code generated from `src/pcs/csm_api_docs.yaml` (OpenAPI 3.0.0), preserving the existing public `ShastaClient::pcs_*` API.

**Architecture:** Single-step pipeline (no Swagger 2.0 conversion needed — PCS spec is already OpenAPI 3.0.0). `build.rs` runs `progenitor::Generator` on `csm_api_docs.yaml` directly and writes `$OUT_DIR/pcs_generated.rs` alongside the existing HSM/CFS/BSS/BOS outputs. Output is `include!`-d into a single `pub(crate)` `src/pcs/generated.rs`. A thin wrapper layer at `src/pcs/wrapper/` maps the 10 existing `pcs_*` methods on `ShastaClient` to generated client calls or to verbatim raw-reqwest re-locations. Because PCS has a single API version (no v1/v2 split), the wrapper directly contains per-resource files (`power_cap.rs`, `power_status.rs`, `transitions.rs`) — no subfolders.

**Tech Stack:** Rust (edition 2021), `progenitor ~ 0.8` (already pulled in), `openapiv3 ~ 2` (already pulled in), `serde_yaml 0.9` (already pulled in by the CFS work), `reqwest 0.12`, `serde 1`, `tokio 1.45`.

**Reference documents:**
- Source design spec: `docs/superpowers/specs/2026-06-13-progenitor-hsm-codegen-design.md`.
- HSM precedent plan (Swagger 2.0): `docs/superpowers/plans/2026-06-13-progenitor-hsm-codegen.md`.
- CFS precedent plan (OpenAPI 3 native, multi-version): `docs/superpowers/plans/2026-06-13-progenitor-cfs-codegen.md`.
- BSS precedent plan (single resource): `docs/superpowers/plans/2026-06-13-progenitor-bss-codegen.md`.
- BOS precedent plan (OpenAPI 3 native, multi-resource, v1/v2 split): `docs/superpowers/plans/2026-06-13-progenitor-bos-codegen.md` — **structurally closest, except PCS has no version split**.
- Output reference docs as templates: `2026-06-13-progenitor-output-reference.md` (HSM), `2026-06-13-progenitor-cfs-output-reference.md`, `2026-06-13-progenitor-bss-output-reference.md`, `2026-06-13-progenitor-bos-output-reference.md`.
- Memory: `[Partial progenitor migration is OK]` — methods can stay on raw reqwest with documented rationale.

**PCS-specific deltas from HSM / CFS / BSS / BOS:**
- **Spec is OpenAPI 3.0.0 natively** (`openapi: "3.0.0"` at line 1). No `swagger2openapi` conversion needed.
- **No version split in the public API**: all paths are at the root (`/transitions`, `/power-status`, `/power-cap`, `/health`, etc.) — no `/v1` or `/v2` prefix. The wrapper layer needs NO `v1/` or `v2/` subfolders; resources live directly under `src/pcs/wrapper/`.
- **3 resources, 10 methods total**: `power_cap` (4 methods), `power_status` (1 method), `transitions` (5 methods). Health endpoints (`/liveness`, `/readiness`, `/health`) exist in the spec but are NOT in csm-rs's current public API — out of scope for this plan.
- **`power_cap` has NO `dispatcher_conv.rs`** — unique among the 3 PCS resources. This is the cleanest candidate for actual progenitor adoption (BOS-health_check pattern). Tasks should evaluate it accordingly.
- **`power_status` (96-line dispatcher_conv) and `transitions` (182-line dispatcher_conv)** have the standard CFS/BSS/BOS-style hand-written-types-with-coupling shape; expect the "move-the-file, keep types hand-written" outcome unless contracts unexpectedly align.
- **`transitions/http_client/` is a directory** (not a flat `http_client.rs` file) — wraps a subdirectory layout. Verify before touching.
- **Mid-sized scale**: 944-line spec (smallest yet), 32 schemas, ~9 operations, 10 public csm-rs methods. Smallest of the 5 migrations so far.
- **dispatcher_conv coupling**: 278 total lines across 2 of 3 resources. `power_cap` has none.

**Type/method name reference (filled in by Task 0):** `docs/superpowers/plans/2026-06-13-progenitor-pcs-output-reference.md`. Subsequent migration tasks consult this file.

---

## Phase 0: Feasibility verification

### Task 0: Run progenitor end-to-end against the PCS spec; capture the name mapping

**Why this exists:** Smoke-test progenitor against the PCS YAML; capture name mapping so subsequent tasks can substitute real names.

**Files:**
- Create: `docs/superpowers/plans/2026-06-13-progenitor-pcs-output-reference.md` (committed)
- Possibly modify: `src/pcs/csm_api_docs.yaml` (post-conversion patches if progenitor refuses)
- Temporary: `/tmp/progenitor-pcs-smoke/` (scratch crate; not committed)

- [ ] **Step 1: Validate the spec parses as OpenAPI 3.0**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
python3 -c "import yaml; d=yaml.safe_load(open('src/pcs/csm_api_docs.yaml')); print(d['openapi']); print('paths:', len(d['paths'])); print('schemas:', len(d['components']['schemas']))"
```
Expected: `3.0.0`, `paths: 9`, `schemas: 32`.

- [ ] **Step 2: Scaffold a throwaway crate**

```bash
mkdir -p /tmp/progenitor-pcs-smoke && cd /tmp/progenitor-pcs-smoke && cargo init --lib --name progenitor_pcs_smoke
cp /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs/src/pcs/csm_api_docs.yaml ./spec.yaml
```

Set `/tmp/progenitor-pcs-smoke/Cargo.toml`:
```toml
[package]
name = "progenitor_pcs_smoke"
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

Create `/tmp/progenitor-pcs-smoke/build.rs`:
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

Replace `/tmp/progenitor-pcs-smoke/src/lib.rs`:
```rust
#![allow(dead_code, clippy::all)]
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
```

- [ ] **Step 4: Run the smoke build**

```bash
cd /tmp/progenitor-pcs-smoke && cargo build 2>&1 | tail -60
```
Expected: succeeds. Known failure modes (same as HSM/CFS/BSS/BOS Task 0):
- `progenitor::Generator::generate_tokens` panics → patch the YAML (commit patches in same Task 0 commit).
- `syn::parse2` fails → same.

If the build fails after up to a day of patching, STOP and report BLOCKED.

- [ ] **Step 5: Inspect generated code and capture the reference doc**

Locate:
```bash
find /tmp/progenitor-pcs-smoke/target -name generated.rs -path '*build*' | head -1
```

Create `docs/superpowers/plans/2026-06-13-progenitor-pcs-output-reference.md` modelled on the existing reference docs (HSM, CFS, BSS, BOS). The BOS reference doc is the closest structural template.

Required sections:

**A. Generated type names (by YAML schema name)** — 32 schemas total. Look up each in the generated file. Plan-relevant schemas (verify against actual generated names):
```
Transition
TransitionRequest
TransitionStatus
PowerStatus
PowerStatusResponse
PowerCap
PowerCapTask
PowerCapTaskSnapshot
HealthCheck
Liveness
Readiness
Error
```
List others as encountered.

**A.1. Schemas referenced by the plan but NOT generated** (empty list if none).

**B. Generated method names (by operationId)** — grep `operationId:` in `src/pcs/csm_api_docs.yaml`. For each one, find the corresponding generated Rust method name. Focus on the 10 operations csm-rs uses today (look up exact operationIds in the spec):
```
transitions list (GET /transitions)
transition get (GET /transitions/{transitionID})
transition post (POST /transitions)
power-status (POST /power-status with body? — verify, since csm-rs's only method is `pcs_power_status_post`)
power-cap list (GET /power-cap)
power-cap get (GET /power-cap/{taskID})
power-cap snapshot post (POST /power-cap/snapshot)
power-cap patch (PATCH /power-cap)
```
List the others (health/liveness/readiness) under "not in csm-rs public API" subsection.

**C.** `progenitor_client::Error<E>` variants (same as HSM/CFS/BSS/BOS).

**D.** `Client::new` / `Client::new_with_client` signatures.

**E. basePath behaviour** — PCS spec uses `servers:` (line 82 of YAML). Look up the actual server URL. csm-rs's `base_url` includes `/apis`. Wrapper override should be `format!("{}/power-control/v1", client.base_url())` (verify what the existing csm-rs code uses by grepping `pcs/` for `format!`-based URL construction). Verify with wiremock test in Step 6.

**F. Spec patches applied** — describe any YAML edits. If none, write "None required — the OpenAPI 3.0.0 spec was accepted as-is."

**G. Generated artefact stats** — total lines, struct count, enum count, method count.

**H. Runtime crate deps** — verify whether PCS-generated code uses any deps not already in `Cargo.toml`. After HSM/CFS/BSS/BOS migrations csm-rs has `progenitor-client = "0.8"`, `regress = "0.10"`, `chrono` with `serde`, `uuid` with `serde`. Grep generated file. Likely no new deps.

- [ ] **Step 6: URL/basePath sanity check**

Add a wiremock test to `/tmp/progenitor-pcs-smoke/src/lib.rs` hitting a small operation (e.g. `GET /liveness`). Use actual generated method name from Section B.

```rust
#[cfg(test)]
mod sanity {
    use super::*;

    #[tokio::test]
    async fn liveness_uses_overridden_baseurl() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/liveness"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let http = reqwest::Client::new();
        let client = Client::new_with_client(&server.uri(), http);
        // Replace `get_liveness` with the actual generated name from the reference doc.
        let _ = client.get_liveness().await.expect("call");
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}
```

Run:
```bash
cd /tmp/progenitor-pcs-smoke && cargo test sanity -- --nocapture
```
Expected: `1 passed`.

Capture finding in Section E of the reference doc.

- [ ] **Step 7: Commit the reference doc IN THE CSM-RS REPO**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
git add docs/superpowers/plans/2026-06-13-progenitor-pcs-output-reference.md
# If you had to apply YAML patches, also stage src/pcs/csm_api_docs.yaml.
git commit -m "chore(pcs): commit progenitor output reference for PCS spec"
```

---

## Phase 1: Build infrastructure

### Task 1: Extend build.rs to also generate the PCS client

**Files:** Modify `build.rs`.

The existing `build.rs` runs `generate_one` for HSM (JSON), CFS (YAML), BSS (JSON), and BOS (YAML). Extend it to ALSO call `generate_one` for PCS (YAML).

Task 0's Section H should confirm BOS-PCS-generated code uses no new deps. Skip Cargo.toml changes unless H says otherwise.

- [ ] **Step 1: Read the current build.rs**

```bash
cat build.rs
```

- [ ] **Step 2: Add a fifth `generate_one` invocation for PCS**

Edit `build.rs`. After the BOS `generate_one` call, add:
```rust
    // PCS: OpenAPI 3.0.0 YAML (upstream-tracked directly).
    generate_one(
        &manifest_dir.join("src/pcs/csm_api_docs.yaml"),
        &out_dir.join("pcs_generated.rs"),
        SpecFormat::Yaml,
    );
```

- [ ] **Step 3: Build to exercise all five spec codegens**

```bash
cargo build 2>&1 | tail -10
```
Expected: `Finished`.

- [ ] **Step 4: Verify all five generated files exist**

```bash
find target -name 'hsm_generated.rs' -path '*out*' | head -1
find target -name 'cfs_generated.rs' -path '*out*' | head -1
find target -name 'bss_generated.rs' -path '*out*' | head -1
find target -name 'bos_generated.rs' -path '*out*' | head -1
find target -name 'pcs_generated.rs' -path '*out*' | head -1
```

- [ ] **Step 5: Commit**

```bash
git add build.rs
git commit -m "feat(pcs): extend build.rs to generate PCS client from OpenAPI 3.0 YAML"
```

---

### Task 2: Create `src/pcs/generated.rs` and the PCS wrapper skeleton

**Files:**
- Create: `src/pcs/generated.rs`
- Create: `src/pcs/wrapper/mod.rs` — shared `gen_client` / `map_err` / `run` + 3 `mod <resource>;` declarations
- Create: `src/pcs/wrapper/power_cap.rs` — placeholder; Task 4 fills it in
- Create: `src/pcs/wrapper/power_status.rs` — placeholder; Task 3 fills it in
- Create: `src/pcs/wrapper/transitions.rs` — placeholder; Task 5 fills it in
- Modify: `src/pcs/mod.rs`

Because PCS has no version split, the wrapper layer is flat — per-resource files directly under `src/pcs/wrapper/`. Same pattern BSS uses (single file with all bootparameters methods) but extended for multiple resources.

- [ ] **Step 1: Create `src/pcs/generated.rs`**

```rust
//! progenitor-generated PCS client. Output of `build.rs`, included verbatim.
//!
//! `pub(crate)` because only the wrapper layer in `crate::pcs::wrapper`
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
include!(concat!(env!("OUT_DIR"), "/pcs_generated.rs"));
```

- [ ] **Step 2: Create `src/pcs/wrapper/mod.rs`**

Open `docs/superpowers/plans/2026-06-13-progenitor-pcs-output-reference.md`:
- Section D — verify `Client::new_with_client` signature matches HSM/CFS/BSS/BOS.
- Section C — verify `progenitor_client::Error<E>` variants (same 8).
- Section E — verify baseurl formula (likely `format!("{}/power-control/v1", client.base_url())`; check existing PCS code for the historical URL convention).

Write `src/pcs/wrapper/mod.rs` modelled on `src/bos/wrapper/mod.rs`. Adjustments:
- The `generated` module is `crate::pcs::generated`.
- The baseurl formula is whatever Section E confirmed.
- Error messages mention "PCS".
- Sub-modules are `mod power_cap; mod power_status; mod transitions;` instead of BOS's `mod v1; mod v2; mod health_check;`.

```rust
//! Thin wrapper bridging the generated PCS client to the public
//! `ShastaClient` API. Mirrors `crate::hsm::wrapper` — see its
//! module-level docs for the design rationale.
//!
//! Responsibilities:
//!  - construct a per-call generated `Client` with bearer auth baked in;
//!  - override the spec's basePath with the URL shape csm-rs has always used;
//!  - map `progenitor_client::Error<T>` into `crate::error::Error`.
//!
//! Per-resource wrapper files (`power_cap.rs`, `power_status.rs`,
//! `transitions.rs`) hold `impl ShastaClient { pub async fn pcs_*() }`
//! blocks that delegate to the generated client via the `run` adapter.
//! No version split: PCS exposes a single API version.

use crate::{ShastaClient, pcs::generated, error::Error};

pub(crate) fn gen_client(
    client: &ShastaClient,
    token: &str,
) -> Result<generated::Client, Error> {
    let inner = crate::common::http::build_client_with_auth(
        client.root_cert(),
        client.socks5_proxy(),
        Some(token),
    )?;
    // Baseurl: verify against Section E of the reference doc + grep
    // existing `format!("...")` patterns in `src/pcs/` for the historical
    // URL convention. Likely: `{base_url}/power-control/v1`.
    let baseurl = format!("{}/power-control/v1", client.base_url());
    Ok(generated::Client::new_with_client(&baseurl, inner))
}

#[allow(clippy::enum_glob_use, clippy::match_same_arms)]
pub(crate) async fn map_err<E: std::fmt::Debug>(
    err: progenitor_client::Error<E>,
) -> Error {
    use progenitor_client::Error::*;
    match err {
        InvalidRequest(s) => Error::Message(format!("PCS invalid request: {s}")),
        CommunicationError(e) => Error::NetError(e),
        InvalidUpgrade(e) => Error::NetError(e),
        ErrorResponse(rv) => {
            let status = rv.status();
            Error::Message(format!(
                "PCS error response: status={status} body={:?}",
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
                "PCS unexpected response: status={status} url={url} body={body}"
            ))
        }
        PreHookError(s) => Error::Message(format!("PCS pre-hook error: {s}")),
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

mod power_cap;
mod power_status;
mod transitions;
```

- [ ] **Step 3: Create `src/pcs/wrapper/power_cap.rs` placeholder**

```rust
//! Wrapper for PCS `power-cap` endpoints. Replaces
//! `src/pcs/power_cap/http_client.rs`.
//!
//! Task 4 inserts the `impl ShastaClient` block here.
```

- [ ] **Step 4: Create `src/pcs/wrapper/power_status.rs` placeholder**

```rust
//! Wrapper for PCS `power-status` endpoint. Replaces
//! `src/pcs/power_status/http_client.rs`.
//!
//! Task 3 inserts the `impl ShastaClient` block here.
```

- [ ] **Step 5: Create `src/pcs/wrapper/transitions.rs` placeholder**

```rust
//! Wrapper for PCS `transitions` endpoints. Replaces
//! `src/pcs/transitions/http_client/`.
//!
//! Task 5 inserts the `impl ShastaClient` block here.
```

- [ ] **Step 6: Wire the new modules into `src/pcs/mod.rs`**

Open `src/pcs/mod.rs` and read it. Add (near other `pub mod` lines):
```rust
pub(crate) mod generated;
mod wrapper;
```

- [ ] **Step 7: Verify it compiles**

```bash
cargo build 2>&1 | tail -10
```
Expected: `Finished`. Dead-code warnings expected on `gen_client`, `map_err`, `run`.

- [ ] **Step 8: Commit**

```bash
git add src/pcs/generated.rs src/pcs/wrapper/ src/pcs/mod.rs
git commit -m "feat(pcs): scaffold generated module + wrapper skeleton"
```

---

## Phase 2: Per-resource migrations (smallest first)

### Task 3: Migrate `pcs::power_status` (1 method)

**Files:**
- Modify: `src/pcs/wrapper/power_status.rs` (replace placeholder with full implementation)
- Modify: `src/pcs/power_status/mod.rs` (drop `pub mod http_client;` if present; keep types/dispatcher_conv references)
- Delete: `src/pcs/power_status/http_client.rs`

- [ ] **Step 1: Inventory the existing method**

```bash
grep -nE "pub async fn pcs_power_status_" src/pcs/power_status/http_client.rs
```
Expected 1 entry: `pcs_power_status_post`. Read the signature.

- [ ] **Step 2: Check dispatcher_conv coupling**

```bash
wc -l src/pcs/power_status/dispatcher_conv.rs
```
Expected ~96 lines.

- [ ] **Step 3: Decide routing**

Look at the generated equivalent. The existing `pcs_power_status_post` likely takes a hand-written `PowerStatusRequest` and returns a hand-written `PowerStatusResponse`. Compare to generated `PowerStatus` / `PowerStatusResponse` shapes.

Likely outcome (per CFS/BSS/BOS precedent): stays raw because of dispatcher_conv coupling. Document the why concretely.

- [ ] **Step 4: Replace the wrapper file**

Replace `src/pcs/wrapper/power_status.rs` content with the actual implementation. Module docstring documents the routing decision with a concrete reason. For raw-reqwest, copy the existing implementation from `src/pcs/power_status/http_client.rs` verbatim and update imports.

- [ ] **Step 5: types.rs strategy**

Keep hand-written (dispatcher_conv coupling justifies it). Adopt the `pub(crate) mod http_client { pub(crate) mod types; #[cfg(feature = "manta-dispatcher")] mod dispatcher_conv; }` form in `src/pcs/power_status/mod.rs` if the existing structure declared things differently.

Actually — since `power_status/http_client.rs` is a FLAT file (not a directory like transitions), the types live at `src/pcs/power_status/types.rs` and dispatcher_conv at `src/pcs/power_status/dispatcher_conv.rs`. The mod.rs probably already declares these alongside `pub mod http_client;`. Just drop the `pub mod http_client;` line — types and dispatcher_conv stay declared as-is.

- [ ] **Step 6: Update `src/pcs/power_status/mod.rs`**

Open the file. Find the `pub mod http_client;` line and delete it. Confirm that `pub mod types;` and (gated) `pub mod dispatcher_conv;` declarations stay.

- [ ] **Step 7: Delete the old http client file**

```bash
git rm src/pcs/power_status/http_client.rs
```

- [ ] **Step 8: Fix collateral**

Run `cargo build 2>&1` and address every error.

- [ ] **Step 9: Build and test**

```bash
cargo build 2>&1 | tail -5
cargo test --lib 2>&1 | tail -5
cargo test --test shasta_client_hsm 2>&1 | tail -5
cargo test --test backend_connector 2>&1 | tail -5
cargo test --test shasta_client_cfs 2>&1 | tail -5
cargo test --test shasta_client_misc 2>&1 | tail -5
cargo test --test shasta_client_bos 2>&1 | tail -5
(cd ../manta && cargo check 2>&1 | tail -5) 2>/dev/null || echo "no ../manta"
```
Baselines: 157 lib / 12 shasta_client_hsm / 32 backend_connector / 12 shasta_client_cfs / 7 misc / 9 shasta_client_bos.

If `tests/shasta_client_pcs.rs` exists, also run it. Otherwise it just doesn't exist.

- [ ] **Step 10: Commit**

```bash
git add src/pcs/wrapper/power_status.rs src/pcs/power_status/mod.rs
git commit -m "$(printf 'refactor(pcs): relocate /power-status wrapper into wrapper layer\n\nRouting: <progenitor OR raw with reason>.')"
```

---

### Task 4: Migrate `pcs::power_cap` (4 methods)

**Files:**
- Modify: `src/pcs/wrapper/power_cap.rs` (replace placeholder)
- Modify: `src/pcs/power_cap/mod.rs` (drop `pub mod http_client;`)
- Delete: `src/pcs/power_cap/http_client.rs`

**This task has the cleanest chance for progenitor adoption**: `power_cap` has NO `dispatcher_conv.rs`. The hand-written types live at `src/pcs/power_cap/types.rs` (6 types). Without the dispatcher coupling, swapping to generated types is potentially feasible.

- [ ] **Step 1: Inventory the existing 4 methods**

```bash
grep -nE "pub async fn pcs_power_cap_" src/pcs/power_cap/http_client.rs
```
Expected 4 entries:
```
pcs_power_cap_get
pcs_power_cap_get_task_id
pcs_power_cap_post_snapshot
pcs_power_cap_patch
```

- [ ] **Step 2: Confirm no dispatcher_conv**

```bash
ls src/pcs/power_cap/dispatcher_conv.rs 2>&1
```
Expected: "No such file" — confirming the unique-clean status.

- [ ] **Step 3: Compare hand-written types to generated types**

Read `src/pcs/power_cap/types.rs` from line 1 onwards. Note every type, every field. Then look at the generated equivalents in `target/debug/build/csm-rs-*/out/pcs_generated.rs` (find with `find target -name pcs_generated.rs -path '*out*' | head -1`). Look up names from Section A of the reference doc.

Compare:
- Field names: do they line up?
- Field types: stricter (typed enums, newtypes, `chrono::DateTime`)? `Option<T>` where hand-written used `T`?
- Field count: divergence?

- [ ] **Step 4: DECISION — full type swap (Option A) vs hand-written retention (Option B)**

**Choose A — Full type swap (recommended if shapes are close)**:
- Replace `src/pcs/power_cap/types.rs` content with `pub use` aliases to the generated types.
- Update `src/pcs/power_cap/mod.rs` re-exports as needed.
- This is the migration outcome the PCS plan exists to attempt — `power_cap` is the cleanest candidate.

**Choose B — Keep hand-written types** (only if A would force a breaking public API change OR cascade significantly):
- Leave `src/pcs/power_cap/types.rs` unchanged.
- Document the choice in the wrapper docstring with the specific reason.
- Use generated types only inside the wrapper file.

Document the choice in the commit message either way.

- [ ] **Step 5: Decide per-method routing**

For each of the 4 methods, look at the generated equivalent:
- Signature match?
- Off-spec query params?
- Tolerant body shape requirements?

If Step 4 chose A: all 4 methods are likely good candidates for progenitor routing (no dispatcher_conv blocker). If Step 4 chose B: methods may still route by adopting generated types at the wrapper boundary and converting to hand-written for the public return.

- [ ] **Step 6: Replace the wrapper file**

Replace `src/pcs/wrapper/power_cap.rs` content. Module docstring documents the type-strategy decision AND each method's routing decision with concrete reasons. Use `crate::pcs::wrapper::run` adapter for data-returning methods routed through progenitor.

- [ ] **Step 7: Update `src/pcs/power_cap/mod.rs`**

Drop `pub mod http_client;`.

- [ ] **Step 8: Delete the old http client file**

```bash
git rm src/pcs/power_cap/http_client.rs
```

- [ ] **Step 9: Fix collateral**

Run `cargo build 2>&1` and address every error. If Step 4 chose A and the field shapes change, fix every consumer the compiler points at.

- [ ] **Step 10: Build, test, downstream check**

Same battery as Task 3 Step 9.

- [ ] **Step 11: Commit**

```bash
git add src/pcs/wrapper/power_cap.rs src/pcs/power_cap/mod.rs src/pcs/power_cap/types.rs
git commit -m "$(printf 'refactor(pcs): generate /power-cap wrapper from progenitor\n\nType strategy: <A: swapped to generated via pub use aliases OR\nB: kept hand-written, conversion at wrapper boundary>.\n\nFully-progenitor-routed methods: <list>.\nKept on raw reqwest with documented rationale: <list>.\n\nBehaviour delta: <if any field-shape changes from the type swap>.')"
```

---

### Task 5: Migrate `pcs::transitions` (5 methods)

**Files:**
- Modify: `src/pcs/wrapper/transitions.rs` (replace placeholder)
- Modify: `src/pcs/transitions/mod.rs` (drop the `pub mod http_client;` declaration; `transitions/http_client/` is a DIRECTORY containing `mod.rs`)
- Delete: `src/pcs/transitions/http_client/mod.rs`
- Possibly delete: `src/pcs/transitions/http_client/` directory if empty after the deletion

- [ ] **Step 1: Inventory the existing 5 methods**

```bash
grep -nE "pub async fn pcs_transitions_" src/pcs/transitions/http_client/mod.rs
```
Expected 5 entries:
```
pcs_transitions_get
pcs_transitions_get_by_id
pcs_transitions_post
pcs_transitions_post_block
pcs_transitions_wait_to_complete
```

- [ ] **Step 2: Check dispatcher_conv coupling**

```bash
wc -l src/pcs/transitions/dispatcher_conv.rs
```
Expected ~182 lines. Largest of the 3 PCS resources — expect "stay raw, keep hand-written" outcome.

- [ ] **Step 3: Decide per-method routing**

For each `pcs_transitions_*` method, evaluate against generated equivalents (Section B of the reference doc). Pay particular attention to:
- `pcs_transitions_post_block` — likely a convenience wrapper that calls `pcs_transitions_post` and then polls. Probably stays raw.
- `pcs_transitions_wait_to_complete` — definitely a polling wrapper, not a single endpoint binding. Stays raw.
- `pcs_transitions_get`, `_get_by_id`, `_post` — single-endpoint methods; check signatures.

- [ ] **Step 4: Replace the wrapper file**

Replace `src/pcs/wrapper/transitions.rs` content. Module docstring lists per-method routing rationale.

- [ ] **Step 5: types.rs strategy**

Keep hand-written (dispatcher_conv 182 lines — largest of the PCS resources).

- [ ] **Step 6: Check what's in `transitions/http_client/`**

```bash
ls src/pcs/transitions/http_client/
```
If there are files OTHER than `mod.rs` (sub-files, types, etc.), be careful not to delete them. The plan assumes only `mod.rs` lives there based on the inventory; verify before deleting.

- [ ] **Step 7: Update `src/pcs/transitions/mod.rs`**

Drop the `pub mod http_client;` line.

- [ ] **Step 8: Delete the old http client**

```bash
git rm src/pcs/transitions/http_client/mod.rs
```

If the `http_client/` directory is now empty:
```bash
# Verify empty:
ls src/pcs/transitions/http_client/ 2>&1
# If empty, remove:
rmdir src/pcs/transitions/http_client/
```

- [ ] **Step 9: Fix collateral**

Run `cargo build 2>&1` and address every error.

- [ ] **Step 10: Build, test, downstream check**

Same battery as Task 3 Step 9.

- [ ] **Step 11: Commit**

```bash
git add src/pcs/wrapper/transitions.rs src/pcs/transitions/mod.rs
git commit -m "$(printf 'refactor(pcs): generate /transitions wrapper from progenitor\n\nFully-progenitor-routed methods: <list>.\nKept on raw reqwest with documented rationale: <list>.')"
```

---

## Phase 3: Final cleanup and verification

### Task 6: End-to-end verification + module docs

**Files:** Modify `src/pcs/mod.rs` (append "How this module is built" doc section).

- [ ] **Step 1: Confirm no remaining `http_client*` files in `src/pcs/`**

```bash
find src/pcs -name 'http_client*' -print
```
Expected: empty.

- [ ] **Step 2: Full build + test + clippy sweep**

```bash
cargo build 2>&1 | tail -5
cargo test --lib 2>&1 | tail -5
cargo test --test shasta_client_hsm 2>&1 | tail -5
cargo test --test backend_connector 2>&1 | tail -5
cargo test --test shasta_client_cfs 2>&1 | tail -5
cargo test --test shasta_client_misc 2>&1 | tail -5
cargo test --test shasta_client_bos 2>&1 | tail -5
cargo clippy --lib 2>&1 | grep -c "warning:" || true
```

Baselines (post-BOS migration): 157 lib / 12 shasta_client_hsm / 32 backend_connector / 12 shasta_client_cfs / 7 misc / 9 shasta_client_bos.

- [ ] **Step 3: Append the codegen pipeline note to `src/pcs/mod.rs`**

After the existing module-level `//!` doc, append:

```rust
//!
//! ## How this module is built
//!
//! Wire-format types and the underlying HTTP client surface are
//! generated from `src/pcs/csm_api_docs.yaml` (OpenAPI 3.0.0). Mirrors
//! the HSM pipeline documented in [`crate::hsm`]; the only structural
//! difference is that the PCS spec is OpenAPI 3.x natively (no
//! Swagger 2.0 conversion needed), so there is no `make convert-spec`
//! step for PCS.
//!
//! 1. `build.rs` runs `progenitor` on the YAML and writes the
//!    generated client to `$OUT_DIR/pcs_generated.rs`.
//! 2. `src/pcs/generated.rs` `include!`s the file as a `pub(crate)`
//!    module — only the wrapper layer and `types.rs` re-export aliases
//!    are allowed to touch it.
//! 3. `src/pcs/wrapper/` glues the generated client (and where the
//!    spec/contract drifts, raw `reqwest` calls) to the public
//!    `ShastaClient::pcs_*` API. Per-resource files (`power_cap.rs`,
//!    `power_status.rs`, `transitions.rs`) host the routing decisions,
//!    documented in each file's module docstring. There is no version
//!    split — PCS exposes a single API version.
//!
//! Per-resource `types.rs` files are hand-rolled (not pure re-exports
//! of generated types) where a full swap would cascade through
//! `dispatcher_conv` bridges (`power_status/dispatcher_conv.rs` and
//! `transitions/dispatcher_conv.rs`). `power_cap` has no
//! `dispatcher_conv` — see commit history for whether its types were
//! swapped to the generated equivalents or kept hand-written.
```

- [ ] **Step 4: Commit**

```bash
git add src/pcs/mod.rs
git commit -m "docs(pcs): describe the progenitor-driven codegen pipeline in module docs"
```

- [ ] **Step 5: Verify final git history**

```bash
git log --oneline | head -12
```
Expected: PCS Task 0 → Task 6 commits in order.

- [ ] **Step 6: Downstream check**

```bash
(cd ../manta && cargo check 2>&1 | tail -5) 2>/dev/null || echo "no ../manta — skip"
```

---

## Self-review notes (kept for the executor)

- **Spec coverage**: The plan migrates the 10 existing `pcs_*` methods. The spec defines health endpoints (`/liveness`, `/readiness`, `/health`) that csm-rs does NOT expose; out of scope. Generated methods for them exist after Task 1; wrapping them is a follow-up.

- **No placeholders**: The Task 0 reference doc has fill-in markers — same pattern as HSM/CFS/BSS/BOS Task 0. Subsequent tasks reference the existing source as authoritative naming, with the plan's verbatim names as starting points.

- **Type consistency**: All public method names match historical `pcs_*` naming. No v1/v2 split to track.

- **Why no version subfolders?**: PCS exposes a single API version; the spec uses root paths (no `/v1` or `/v2` prefix). The wrapper directly contains per-resource files, no extra nesting.

- **Why `power_cap` could be the first full-progenitor adoption in this migration train**: it's the only PCS resource without a `dispatcher_conv.rs`. The HSM/CFS/BSS/BOS pattern has been "keep hand-written types, route methods where contracts match" — almost always landing 0/N because dispatcher_conv coupling blocks the type swap. `power_cap` removes that blocker. Task 4 has an explicit decision step (Step 4) where the implementer evaluates the actual shape comparison and picks A (swap) or B (keep).

- **Why combine all of v1 — wait, there is no v1**: PCS doesn't split. Task 3 (power_status), Task 4 (power_cap), Task 5 (transitions) — one per resource, in size order.

- **Out of scope for this plan**: extending the wrapper to expose `/liveness`, `/readiness`, `/health` endpoints. Generated methods for them exist after Task 1; wrapping is a follow-up.
