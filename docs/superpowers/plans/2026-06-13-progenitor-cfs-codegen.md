# progenitor-based CFS client codegen — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hand-written `src/cfs/` HTTP client and wire-format types with code generated from `src/cfs/csm_api_docs.yaml` (OpenAPI 3.0.2), preserving the existing public `ShastaClient` API and the v2/v3 dual surface.

**Architecture:** Single-step pipeline (no Swagger 2.0 conversion needed — CFS spec is already OpenAPI 3.0.2). `build.rs` runs `progenitor::Generator` on `csm_api_docs.yaml` directly and writes `$OUT_DIR/cfs_generated.rs` next to the existing `hsm_generated.rs`. Output is `include!`-d into a single `pub(crate)` `src/cfs/generated.rs` module that contains BOTH v2 and v3 surfaces. A thin wrapper layer at `src/cfs/wrapper/` maps the existing `cfs_*` methods on `ShastaClient` to generated client calls, organised by API version (`wrapper/v2/` and `wrapper/v3/`) so the version boundary is visible in the directory tree. Methods that don't fit cleanly stay on raw `reqwest` with documented rationale (per the established partial-migration policy from the HSM migration).

**Tech Stack:** Rust (edition 2021), `progenitor ~ 0.8` (already pulled in by the HSM work), `openapiv3 ~ 2` (already in build-dependencies), `reqwest 0.12`, `serde 1`, `tokio 1.45`.

**Reference documents:**
- Source design spec: `docs/superpowers/specs/2026-06-13-progenitor-hsm-codegen-design.md` — the CFS migration applies the same architectural decisions (types vs wrapper layering, partial-migration policy, build.rs + include!, public-API preservation).
- HSM precedent plan: `docs/superpowers/plans/2026-06-13-progenitor-hsm-codegen.md` — the patterns used here are direct adaptations.
- HSM output reference: `docs/superpowers/plans/2026-06-13-progenitor-output-reference.md` — the format Task 0 below replicates for CFS.
- Memory: `[Partial progenitor migration is OK]` — methods can stay on raw reqwest with documented rationale where generated contracts conflict with real CSM. Same policy applies here.

**CFS-specific deltas from the HSM migration:**
- **No Swagger 2.0 conversion step**, because the CFS spec is already OpenAPI 3.0.2 (`openapi: "3.0.2"` at line 24). The `Makefile` `convert-spec` target is HSM-only; CFS does not need one. The YAML feeds progenitor directly via `build.rs`.
- **Dual v2 + v3 surface**: every resource (component, configuration, session) exposes both versions on different URL prefixes. The spec puts each version under its own path tree (`/v2/sessions`, `/v3/sessions`, …). progenitor generates one Client with methods named from operationIds, so v2 and v3 methods coexist on a single client — no special handling needed.
- **Smaller scale**: 36 public methods, 96 schemas, vs 50/210 for HSM. Migration tasks are correspondingly tighter.
- **`dispatcher_conv` coupling in every CFS submodule**: same precedent as HSM Tasks 9/10/11 — defer the wholesale type swap when a large `dispatcher_conv.rs` would cascade. Pattern: keep hand-written types, route methods through progenitor where the contracts fit, keep on raw reqwest otherwise.

**Type/method name reference (filled in by Task 0):** `docs/superpowers/plans/2026-06-13-progenitor-cfs-output-reference.md`. Subsequent migration tasks consult this file because progenitor's exact mangled type/method names are not knowable until the generator runs on the converted spec.

---

## Phase 0: Feasibility verification (one task, gates everything else)

### Task 0: Run progenitor end-to-end against the CFS spec; capture the name mapping

**Why this exists:** progenitor accepts OpenAPI 3.x natively, so no conversion is required, but the spec may still contain quirks progenitor refuses (the HSM spec needed three patches). The exact Rust type/method names progenitor emits are determined by its mangling rules + the spec's `operationId` and `components.schemas` keys. Nothing in Phase 1+ can be concrete until we have inspected the generated `.rs` file.

**Files:**
- Create: `docs/superpowers/plans/2026-06-13-progenitor-cfs-output-reference.md` (committed)
- Temporary: `/tmp/progenitor-cfs-smoke/` (scratch crate; not committed)

- [ ] **Step 1: Validate the spec parses as OpenAPI 3.0**

Run:
```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
python3 -c "import yaml; d=yaml.safe_load(open('src/cfs/csm_api_docs.yaml')); print(d['openapi']); print('paths:', len(d['paths'])); print('schemas:', len(d['components']['schemas']))"
```
Expected output: `3.0.2`, `paths: ~20`, `schemas: ~96`. If the OpenAPI version is missing or the YAML doesn't parse, STOP — the spec needs investigation before proceeding.

- [ ] **Step 2: Scaffold a throwaway crate to test progenitor against the CFS spec**

```bash
mkdir -p /tmp/progenitor-cfs-smoke && cd /tmp/progenitor-cfs-smoke && cargo init --lib --name progenitor_cfs_smoke
cp /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs/src/cfs/csm_api_docs.yaml ./spec.yaml
```

Set `/tmp/progenitor-cfs-smoke/Cargo.toml` to:
```toml
[package]
name = "progenitor_cfs_smoke"
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

(`serde_yaml` is new for build-deps because the CFS spec is YAML not JSON — progenitor's `Generator::generate_tokens` takes `&openapiv3::OpenAPI`, which we'll build via `serde_yaml::from_str`.)

- [ ] **Step 3: Add a minimal build.rs for the smoke crate**

Create `/tmp/progenitor-cfs-smoke/build.rs`:
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

Replace `/tmp/progenitor-cfs-smoke/src/lib.rs`:
```rust
#![allow(dead_code, clippy::all)]
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
```

- [ ] **Step 4: Run the smoke build**

```bash
cd /tmp/progenitor-cfs-smoke && cargo build 2>&1 | tail -60
```
Expected: `cargo build` succeeds. Two known failure modes (same as HSM Task 0):
  - `progenitor::Generator::generate_tokens` panics on the spec → patch the YAML (commit the patches alongside the spec, document in Section F of the reference doc).
  - `syn::parse2` fails on the generated tokens → same; spec issue surfacing through codegen.

If the build fails after up to a day of patching, STOP this plan and report BLOCKED.

- [ ] **Step 5: Inspect the generated code and capture the type/method name reference**

Locate the generated file:
```bash
find /tmp/progenitor-cfs-smoke/target -name generated.rs -path '*build*' | head -1
```

Open it. Catalogue the type and method names progenitor emitted. Create `docs/superpowers/plans/2026-06-13-progenitor-cfs-output-reference.md` with this structure (model on the HSM reference doc):

```markdown
# progenitor output reference for CFS

Captured by Task 0 of the implementation plan. Updated when the YAML changes.

## A. Generated type names (by YAML schema name)

The table below lists the schemas referenced by Tasks 3–8 of the implementation plan. The full inventory of generated types is in the generated file; grep `pub (struct|enum) ` to enumerate them.

| YAML schema | Generated Rust type (`generated::types::…`) |
|---|---|
| V2SessionCreateRequest | …fill in… |
| V3SessionCreateRequest | …fill in… |
| V2SessionDetails | …fill in… |
| V3SessionData | …fill in… |
| V2SessionDetailsArray | …fill in… |
| V3SessionDataCollection | …fill in… |
| V3SessionIdCollection | …fill in… |
| V2ComponentsCreateRequest | …fill in… |
| V3ComponentsCreateRequest | …fill in… |
| V2ComponentsUpdateRequest | …fill in… |
| V3ComponentsUpdateRequest | …fill in… |
| V2ComponentUpdateRequest | …fill in… |
| V3ComponentUpdateRequest | …fill in… |
| V2ComponentDetails | …fill in… |
| V3ComponentData | …fill in… |
| V2ComponentDetailsArray | …fill in… |
| V3ComponentDataCollection | …fill in… |
| V3ComponentIdCollection | …fill in… |
| V2ConfigurationUpdateRequest | …fill in… |
| V3ConfigurationUpdateRequest | …fill in… |
| V2Options | …fill in… |
| V3Options | …fill in… |
| V2OptionsUpdateRequest | …fill in… |
| V3OptionsUpdateRequest | …fill in… |
| V3SourceCreateRequest | …fill in… |
| V3SourceUpdateRequest | …fill in… |
| V3SourceRestoreRequest | …fill in… |
| Version | …fill in… |
| Healthz | …fill in… |

## A.1. Schemas referenced by the plan that are NOT generated (or appear under a different name)

…fill in any cases where the plan's name isn't what progenitor emitted — same role as Section A.1 of the HSM reference doc. Most likely: any schema referenced by the plan that's actually inline-defined in the YAML (and thus emitted under an auto-generated nested name).

## B. Generated method names (by operationId)

Section B lists every operationId referenced by Tasks 3–8 of the implementation plan. For the full inventory, grep `pub async fn ` in the generated file or `operationId:` in `src/cfs/csm_api_docs.yaml`.

| YAML operationId | Generated method (`generated::Client::…`) | HTTP verb + path |
|---|---|---|
| (v2 sessions) | …fill in 6+ rows… | GET/POST/DELETE /v2/sessions /v2/sessions/{name} |
| (v3 sessions) | …fill in 6+ rows… | GET/POST/DELETE /v3/sessions /v3/sessions/{name} |
| (v2 components) | …fill in 4+ rows… | GET/PUT/DELETE /v2/components /v2/components/{id} |
| (v3 components) | …fill in 6+ rows… | GET/PUT/PATCH/DELETE /v3/components /v3/components/{id} |
| (v2 configurations) | …fill in 3+ rows… | GET/PUT/DELETE /v2/configurations /v2/configurations/{id} |
| (v3 configurations) | …fill in 3+ rows… | GET/PUT/DELETE /v3/configurations /v3/configurations/{id} |
| (v3 sources) | …fill in 3+ rows… | GET/PUT/DELETE /v3/sources /v3/sources/{id} |
| (v2 options / v3 options) | …fill in… | GET/PATCH /v2/options /v3/options |
| (root) | …fill in… | GET /versions /healthz /v2 /v3 |

Look up the actual operationIds by grepping `operationId:` in the YAML — there are too many for the plan to enumerate by hand and the YAML is the canonical source.

## C. progenitor `Error` enum variant list

…copy from `/tmp/progenitor-cfs-smoke/target/debug/build/.../out/generated.rs` (or from progenitor-client 0.8 source if identical to HSM)…

## D. Client constructor signature

…copy from the generated file…

## E. basePath behaviour

The CFS spec declares `servers: - url: https://api-gw-service-nmn.local/apis/cfs`. ShastaClient's `base_url` already includes `/apis` (e.g. `https://api.cmn.alps.cscs.ch/apis`). So:
- Override the spec's server URL at client construction.
- The wrapper's `gen_client` builds the generated `Client` with `format!("{}/cfs", client.base_url())`. v2/v3 path prefixes come from the operation paths themselves.
- Verify with a wiremock test (Step 6 below).

## F. Spec patches applied

…describe any patches made to make progenitor accept the spec, with counts and re-run commands. Same format as HSM Section F. If no patches were needed, write "None required — the OpenAPI 3.0.2 spec was accepted as-is."

## G. Generated artefact stats

Total generated file: …KB / …lines / …structs / …enums / …methods. Fill in.

## H. Runtime crate dependencies

The HSM work added `regress = "0.10"` and `progenitor-client = "0.8"` to runtime deps because the generated code references them (typify's `pattern` validators use `regress::Regex`, all generated methods use `progenitor_client::Error`). CFS may add no new deps (both already present) but verify.
```

If a schema or operationId from the table is missing from the generated file, write "NOT GENERATED" in the right column.

- [ ] **Step 6: URL/basePath sanity check**

Add a wiremock-based test to `/tmp/progenitor-cfs-smoke/src/lib.rs` that confirms the generated client honours an overridden basePath. Pick a small operation (e.g. `GET /healthz` or `GET /versions`) — look up the actual generated method name from the reference doc you just wrote.

```rust
#[cfg(test)]
mod sanity {
    use super::*;

    #[tokio::test]
    async fn healthz_uses_overridden_baseurl() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/healthz"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let http = reqwest::Client::new();
        let client = Client::new_with_client(&server.uri(), http);
        // Replace `do_healthz_get` with the actual generated name from the reference doc.
        let _ = client.do_healthz_get().await.expect("call");
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}
```

Run:
```bash
cd /tmp/progenitor-cfs-smoke && cargo test sanity -- --nocapture
```
Expected: `1 passed`. Capture the basePath finding (does progenitor prepend the spec's `servers[0].url` to operation paths, or only what's passed to `new_with_client`?) in Section E of the reference doc.

- [ ] **Step 7: Commit the reference doc**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
git add docs/superpowers/plans/2026-06-13-progenitor-cfs-output-reference.md
git commit -m "chore(cfs): commit progenitor output reference for CFS spec"
```

(The YAML is already committed; no further spec artefact to add unless Step 4 required YAML patches, in which case stage those too.)

---

## Phase 1: Build infrastructure

### Task 1: Extend build.rs to also generate the CFS client

**Files:**
- Modify: `build.rs`

The existing `build.rs` (from the HSM work) reads `src/hsm/csm_api_docs.openapi3.json` and writes `$OUT_DIR/hsm_generated.rs`. Extend it to ALSO read `src/cfs/csm_api_docs.yaml` and write `$OUT_DIR/cfs_generated.rs`.

- [ ] **Step 1: Read the current build.rs**

```bash
cat build.rs
```

Note the existing pattern: reads JSON, parses to `openapiv3::OpenAPI`, runs `progenitor::Generator`, writes the output to `$OUT_DIR/hsm_generated.rs`. The CFS extension follows the same shape but reads YAML and writes a different filename.

- [ ] **Step 2: Replace the body with a two-spec generator**

Replace the entire body of `build.rs` with:
```rust
//! Build-time codegen of the HSM and CFS HTTP clients from their
//! respective OpenAPI specs.
//!
//! Each call to `generate_one` reads one spec file, runs progenitor on
//! it, pretty-prints the result, and writes it under `$OUT_DIR`. The
//! `src/<module>/generated.rs` files `include!` the corresponding
//! output. Re-runs when either spec changes.

use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // HSM: OpenAPI 3.0 JSON (converted from the upstream Swagger 2.0).
    generate_one(
        &manifest_dir.join("src/hsm/csm_api_docs.openapi3.json"),
        &out_dir.join("hsm_generated.rs"),
        SpecFormat::Json,
    );

    // CFS: OpenAPI 3.0.2 YAML (upstream-tracked directly).
    generate_one(
        &manifest_dir.join("src/cfs/csm_api_docs.yaml"),
        &out_dir.join("cfs_generated.rs"),
        SpecFormat::Yaml,
    );
}

enum SpecFormat {
    Json,
    Yaml,
}

fn generate_one(spec_path: &PathBuf, out_path: &PathBuf, format: SpecFormat) {
    println!("cargo:rerun-if-changed={}", spec_path.display());

    let src = fs::read_to_string(spec_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", spec_path.display()));

    let spec: openapiv3::OpenAPI = match format {
        SpecFormat::Json => serde_json::from_str(&src)
            .unwrap_or_else(|e| panic!("parse {} as JSON: {e}", spec_path.display())),
        SpecFormat::Yaml => serde_yaml::from_str(&src)
            .unwrap_or_else(|e| panic!("parse {} as YAML: {e}", spec_path.display())),
    };

    let mut generator = progenitor::Generator::default();
    let tokens = generator
        .generate_tokens(&spec)
        .unwrap_or_else(|e| {
            panic!(
                "progenitor codegen failed for {}: {e}. \
                 Check the spec is valid OpenAPI 3.0 and contains no \
                 progenitor-unsupported constructs.",
                spec_path.display()
            )
        });
    let ast: syn::File =
        syn::parse2(tokens).expect("generated tokens do not parse");
    let pretty = prettyplease::unparse(&ast);

    fs::write(out_path, pretty)
        .unwrap_or_else(|e| panic!("write {}: {e}", out_path.display()));
}
```

- [ ] **Step 3: Add serde_yaml to build-dependencies**

Open `Cargo.toml`. Find the `[build-dependencies]` block. Add a single line:
```toml
serde_yaml = "0.9"
```

The full block should now read:
```toml
[build-dependencies]
progenitor = "0.8"
openapiv3 = "2"
serde_json = "1"
serde_yaml = "0.9"
prettyplease = "0.2"
syn = "2"
```

- [ ] **Step 4: Build to exercise both spec codegens**

```bash
cargo build 2>&1 | tail -10
```
Expected: `Compiling csm-rs (build script)` then `Compiling csm-rs` then `Finished`. The build script runs both `generate_one` calls; only `hsm_generated.rs` is currently `include!`-d so `cfs_generated.rs` is produced but unused (this is fine — Task 2 wires it in).

If progenitor panics on the CFS spec at this step, the panic message will say which `generate_one` invocation failed. Re-check Task 0's findings.

- [ ] **Step 5: Verify both generated files exist**

```bash
find target -name 'hsm_generated.rs' -path '*out*' | head -1
find target -name 'cfs_generated.rs' -path '*out*' | head -1
```
Expected: one line each. Both files must be present.

- [ ] **Step 6: Commit**

```bash
git add build.rs Cargo.toml
git commit -m "feat(cfs): extend build.rs to generate CFS client from OpenAPI 3.0 YAML"
```

---

### Task 2: Create `src/cfs/generated.rs` and the CFS wrapper module skeleton

**Files:**
- Create: `src/cfs/generated.rs`
- Create: `src/cfs/wrapper/mod.rs` — shared `gen_client` / `map_err` / `run` and the `mod v2; mod v3;` declarations.
- Create: `src/cfs/wrapper/v2/mod.rs` — empty placeholder; per-resource `mod component; mod configuration; mod session;` lines are added by Tasks 3, 5, 7.
- Create: `src/cfs/wrapper/v3/mod.rs` — empty placeholder; per-resource `mod component; mod configuration; mod session;` lines are added by Tasks 4, 6, 8.
- Modify: `src/cfs/mod.rs`

This task creates the CFS-side analogues of `src/hsm/generated.rs` and `src/hsm/wrapper/mod.rs`. The structure is intentionally symmetric — anyone who learned the HSM layout will recognise the CFS one.

- [ ] **Step 1: Create `src/cfs/generated.rs`**

```rust
//! progenitor-generated CFS client. Output of `build.rs`, included verbatim.
//!
//! `pub(crate)` because only the wrapper layer in `crate::cfs::wrapper`
//! and per-resource `types.rs` re-export aliases are allowed to touch
//! the generated symbols. Public consumers go through `ShastaClient`.
#![allow(dead_code, clippy::all, missing_docs, non_camel_case_types, non_snake_case, unused_imports)]
include!(concat!(env!("OUT_DIR"), "/cfs_generated.rs"));
```

- [ ] **Step 2: Create `src/cfs/wrapper/mod.rs`**

Open `docs/superpowers/plans/2026-06-13-progenitor-cfs-output-reference.md` and look up:
  (a) Section D — the actual `Client::new_with_client` signature in the CFS generated file. It is almost certainly the same as the HSM one (progenitor 0.8 emits a consistent signature), but verify.
  (b) Section C — the variant list of `progenitor_client::Error<E>`. Should match HSM's; verify.

Write `src/cfs/wrapper/mod.rs` modelled on `src/hsm/wrapper/mod.rs` (which already exists and is the proven precedent). The only differences from the HSM version are:
- The `generated` module path is `crate::cfs::generated` (not `crate::hsm::generated`).
- The baseurl formula is `format!("{}/cfs", client.base_url())` (not `format!("{}/smd/hsm/v2", ...)`); v2 vs v3 path prefixes are determined by the operation methods themselves.

```rust
//! Thin wrapper bridging the generated CFS client to the public
//! `ShastaClient` API. Mirrors `crate::hsm::wrapper` — see its
//! module-level docs for the design rationale.
//!
//! Responsibilities:
//!  - construct a per-call generated `Client` with bearer auth baked in;
//!  - override the spec's basePath with the URL shape csm-rs has always
//!    used (`{base_url}/cfs` — v2 and v3 prefixes come from the
//!    operation paths);
//!  - map `progenitor_client::Error<T>` into `crate::error::Error` (async,
//!    reads the body for UnexpectedResponse/ErrorResponse — same idiom
//!    as `crate::hsm::wrapper::map_err`).
//!
//! Per-resource wrapper files (`component.rs`, `configuration.rs`,
//! `session.rs`) hold `impl ShastaClient { pub async fn cfs_*() }`
//! blocks that delegate to the generated client via the `run` adapter.

use crate::{ShastaClient, error::Error, cfs::generated};

/// Build a generated CFS `Client` bound to the caller's token. Re-uses
/// the shared `http::build_client_with_auth` helper so timeout / TLS /
/// proxy config stays consistent with the rest of csm-rs.
pub(crate) fn gen_client(
    client: &ShastaClient,
    token: &str,
) -> Result<generated::Client, Error> {
    let inner = crate::common::http::build_client_with_auth(
        client.root_cert(),
        client.socks5_proxy(),
        Some(token),
    )?;
    // CFS basePath: csm-rs's `base_url` already ends in `/apis`; CFS
    // operations live under `/cfs/...` (v2 and v3 prefixes are part of
    // the operation paths).
    let baseurl = format!("{}/cfs", client.base_url());
    Ok(generated::Client::new_with_client(&baseurl, inner))
}

/// Map a generated `Error` into the crate's `Error` enum. Async because
/// `UnexpectedResponse` and `ErrorResponse` carry a `reqwest::Response`
/// whose body must be read to produce a useful diagnostic. Mirrors
/// `crate::hsm::wrapper::map_err`.
pub(crate) async fn map_err<E: std::fmt::Debug>(
    err: progenitor_client::Error<E>,
) -> Error {
    use progenitor_client::Error::*;
    match err {
        InvalidRequest(s) => Error::Message(format!("CFS invalid request: {s}")),
        CommunicationError(e) => Error::NetError(e),
        InvalidUpgrade(e) => Error::NetError(e),
        ErrorResponse(rv) => {
            let status = rv.status();
            Error::Message(format!("CFS error response: status={status} body={:?}", rv.into_inner()))
        }
        ResponseBodyError(e) => Error::NetError(e),
        InvalidResponsePayload(_, e) => Error::SerdeJsonError(e),
        UnexpectedResponse(resp) => {
            let status = resp.status();
            let url = resp.url().clone();
            let body = resp.text().await.unwrap_or_else(|e| format!("<body read failed: {e}>"));
            Error::Message(format!("CFS unexpected response: status={status} url={url} body={body}"))
        }
        PreHookError(s) => Error::Message(format!("CFS pre-hook error: {s}")),
    }
}

/// Adapter so per-resource wrappers can write:
/// `let rv = run(self, token, |c| c.do_something()).await?;`
/// Mirrors `crate::hsm::wrapper::run`.
pub(crate) async fn run<F, Fut, T, E>(
    client: &ShastaClient,
    token: &str,
    op: F,
) -> Result<T, Error>
where
    F: FnOnce(generated::Client) -> Fut,
    Fut: std::future::Future<Output = Result<progenitor_client::ResponseValue<T>, progenitor_client::Error<E>>>,
    E: std::fmt::Debug,
{
    let gc = gen_client(client, token)?;
    match op(gc).await {
        Ok(rv) => Ok(rv.into_inner()),
        Err(e) => Err(map_err(e).await),
    }
}
```

If the `progenitor_client::Error<E>` enum variant names don't match, the compiler will tell you exactly which arms are wrong (Section C of the reference doc has the truth).

- [ ] **Step 3: Create the v2 and v3 wrapper submodule shells**

The v2/v3 subdirectories exist so per-resource wrappers (Tasks 3-8) land in
`src/cfs/wrapper/v2/component.rs`, `src/cfs/wrapper/v3/component.rs`, etc.
Create the two `mod.rs` placeholders so the compiler sees the modules now,
even though they're empty:

Create `src/cfs/wrapper/v2/mod.rs`:
```rust
//! `manta`-facing CFS v2 wrapper methods. Per-resource sub-modules
//! (`component`, `configuration`, `session`) attach
//! `impl ShastaClient { pub async fn cfs_<resource>_v2_*() }` blocks
//! to the public client. Each sub-module's docstring records the
//! per-method routing decision (generated client vs raw reqwest).
//!
//! See `crate::cfs::wrapper` for the shared `gen_client` / `map_err`
//! / `run` helpers — they're version-agnostic and serve both v2 and v3.

// Per-resource modules are added by Tasks 3, 5, 7:
//   mod component;
//   mod configuration;
//   mod session;
```

Create `src/cfs/wrapper/v3/mod.rs`:
```rust
//! `manta`-facing CFS v3 wrapper methods. Per-resource sub-modules
//! (`component`, `configuration`, `session`) attach
//! `impl ShastaClient { pub async fn cfs_<resource>_v3_*() }` blocks
//! to the public client. Each sub-module's docstring records the
//! per-method routing decision (generated client vs raw reqwest).
//!
//! See `crate::cfs::wrapper` for the shared `gen_client` / `map_err`
//! / `run` helpers — they're version-agnostic and serve both v2 and v3.

// Per-resource modules are added by Tasks 4, 6, 8:
//   mod component;
//   mod configuration;
//   mod session;
```

At the bottom of `src/cfs/wrapper/mod.rs` (the file from Step 2), add:
```rust
mod v2;
mod v3;
```

- [ ] **Step 4: Wire the new modules into `src/cfs/mod.rs`**

Open `src/cfs/mod.rs` and read it first to understand the existing layout:
```bash
cat src/cfs/mod.rs
```

Add these two module declarations near the other `pub mod` lines (the exact position matters less than the declarations existing):
```rust
pub(crate) mod generated;
mod wrapper;
```

- [ ] **Step 5: Verify it compiles**

```bash
cargo build 2>&1 | tail -10
```
Expected: `Finished` with dead-code warnings for `cfs::wrapper::gen_client`, `cfs::wrapper::map_err`, `cfs::wrapper::run`, plus rust-analyzer "module empty" hints for `wrapper/v2/mod.rs` and `wrapper/v3/mod.rs` (expected — Tasks 3-8 fill them in). If `progenitor_client::Error` variant names don't match, the compiler will tell you exactly which arms are wrong — update them against Section C of the reference doc.

- [ ] **Step 6: Commit**

```bash
git add src/cfs/generated.rs src/cfs/wrapper/ src/cfs/mod.rs
git commit -m "feat(cfs): scaffold generated module + wrapper skeleton (v2/v3 split)"
```

---

## Phase 2: Per-resource migrations

For every migration task below, the same rhythm applies:
1. Read the existing `*/http_client/v{N}/mod.rs` to capture each `pub async fn` signature byte-for-byte — these are the canonical public API names. The plan's verbatim names below are sketches; the existing source wins on conflict.
2. Look up each generated method/type name in `docs/superpowers/plans/2026-06-13-progenitor-cfs-output-reference.md`.
3. Decide for each method: route through progenitor's generated client (preferred) OR keep on raw `reqwest` with documented rationale in the wrapper module docstring. The partial-migration policy from `[Partial progenitor migration is OK]` applies — both choices are acceptable, but every non-wrap needs a concrete WHY.
4. Replace `*/http_client/v{N}/types.rs` with pure `pub use` re-exports of generated types, UNLESS doing so would cascade through `dispatcher_conv.rs`. If it would, keep hand-written types (Task 9/10/11 precedent from the HSM work).
5. Delete the old `http_client/v{N}/mod.rs` after the wrapper is in place.
6. `cargo build && cargo test --lib`.
7. Commit.

### Task 3: Migrate `cfs::component` v2 (9 methods)

**Files:**
- Create: `src/cfs/wrapper/v2/component.rs`
- Modify: `src/cfs/wrapper/v2/mod.rs` (add `mod component;`)
- Modify: `src/cfs/component/http_client/v2/types.rs`
- Modify: `src/cfs/component/http_client/mod.rs` (drop the `pub mod v2;` line for the bits we replace; keep its `pub use` re-exports)
- Delete: `src/cfs/component/http_client/v2/mod.rs`

- [ ] **Step 1: Inventory the existing v2 public API**

```bash
grep -nE "pub async fn cfs_component_v2_" src/cfs/component/http_client/v2/mod.rs
```
Expected 9 entries (from the pre-survey):
```
cfs_component_v2_get
cfs_component_v2_get_all
cfs_component_v2_get_single_component
cfs_component_v2_get_multiple
cfs_component_v2_get_parallel
cfs_component_v2_get_query
cfs_component_v2_put_component
cfs_component_v2_put_component_list
cfs_component_v2_delete_single_component
```

Each signature must be preserved byte-for-byte by the wrapper file you write in Step 3. Read the file to capture exact parameter types and return types.

- [ ] **Step 2: Check the dispatcher_conv coupling**

```bash
wc -l src/cfs/component/http_client/v2/dispatcher_conv.rs
```

If the file exists and is large (>100 lines), Task 9/10/11 HSM precedent applies: keep hand-written types in `v2/types.rs`, route methods through progenitor where the contracts fit, and skip the types.rs re-export step. If the file is small or absent, do the full re-export swap.

- [ ] **Step 3: Create the wrapper file**

Create `src/cfs/wrapper/v2/component.rs`. Module docstring should document each method's routing decision (progenitor vs raw reqwest). Template (the example shows ONE method routed through `run`; replicate the pattern for each `cfs_component_v2_*` method, looking up the generated name in Section B of the reference doc):

```rust
//! Wrapper for `/cfs/v2/components`. Replaces
//! `src/cfs/component/http_client/v2/mod.rs`.
//!
//! Per-method routing decisions:
//! - `cfs_component_v2_get_all` → progenitor `do_<operationId>` (clean signature match).
//! - `cfs_component_v2_get_query` → progenitor.
//! - `cfs_component_v2_get_multiple` → raw reqwest because <REASON: contract mismatch
//!   the implementer discovered while writing this file — e.g. accepts a comma-joined
//!   ids string that the generated method exposes as Option<&str>, but the actual
//!   wire shape needs <reason>>.
//! - `cfs_component_v2_get_parallel` → raw reqwest; it's a convenience wrapper over
//!   `cfs_component_v2_get_multiple`, not an endpoint binding of its own.
//! - (etc, one line per method.)

use crate::{
    ShastaClient,
    error::Error,
    cfs::component::http_client::v2::types::Component,
};

use super::run;

impl ShastaClient {
    /// `GET /cfs/v2/components`.
    pub async fn cfs_component_v2_get_all(
        &self,
        token: &str,
    ) -> Result<Vec<Component>, Error> {
        // Replace `do_<operationId>` with the actual generated name from Section B.
        run(self, token, |c| async move { c.do_v2_components_get(/* args from generated signature */).await }).await
    }

    // … one impl block per existing v2 method.
}
```

For methods that stay on raw reqwest, copy the original implementation verbatim from `src/cfs/component/http_client/v2/mod.rs` into the wrapper file. The point of the migration is to consolidate where the method lives (one place per public method, all under `src/cfs/wrapper/`), not to force every method through progenitor.

- [ ] **Step 4: Decide types.rs strategy**

Based on Step 2's `dispatcher_conv` size:

**If dispatcher_conv is small/absent**, replace `src/cfs/component/http_client/v2/types.rs` with pure re-exports:
```rust
//! Re-exports of the progenitor-generated CFS v2 component schemas.

pub use crate::cfs::generated::types::{
    V2ComponentDetails           as Component,         // verify actual name in Section A
    V2ComponentDetailsArray      as ComponentArray,    // verify
    V2ComponentsCreateRequest    as ComponentsCreateRequest,  // verify
    V2ComponentsUpdateRequest    as ComponentsUpdateRequest,  // verify
    V2ComponentUpdateRequest     as ComponentUpdateRequest,   // verify
};
```

**If dispatcher_conv is large**, leave `types.rs` hand-written and import the generated types only inside the wrapper file. The behaviour-delta cost of a wholesale type swap (cascade through `dispatcher_conv.rs`) outweighs the cleanup benefit at this stage.

- [ ] **Step 5: Register the wrapper file**

In `src/cfs/wrapper/v2/mod.rs`, replace the `// mod component;` placeholder comment with:
```rust
mod component;
```

- [ ] **Step 6: Update `src/cfs/component/http_client/mod.rs`**

Open it and drop the `pub mod v2;` line (the wrapper now owns the methods; the v2 sub-module's `mod.rs` will be deleted). Keep any `pub use v2::types::*;` re-export — `types.rs` itself still exists.

- [ ] **Step 7: Delete the old http client file**

```bash
git rm src/cfs/component/http_client/v2/mod.rs
```
(Keep `v2/types.rs` and `v2/dispatcher_conv.rs` — `types.rs` is now a re-export shim or unchanged, depending on Step 4; `dispatcher_conv.rs` is preserved for the manta-dispatcher bridge.)

- [ ] **Step 8: Build and test**

```bash
cargo build 2>&1 | tail -10
cargo test --lib 2>&1 | tail -5
```
Expected: Finished, all baseline tests pass.

- [ ] **Step 9: Commit**

```bash
git add src/cfs/wrapper/v2/component.rs src/cfs/wrapper/v2/mod.rs \
        src/cfs/component/http_client/v2/types.rs \
        src/cfs/component/http_client/mod.rs
git commit -m "refactor(cfs): generate /cfs/v2/components wrapper from progenitor"
```

---

### Task 4: Migrate `cfs::component` v3 (10 methods)

**Files:**
- Create: `src/cfs/wrapper/v3/component.rs`
- Modify: `src/cfs/wrapper/v3/mod.rs` (add `mod component;`)
- Modify: `src/cfs/component/http_client/v3/types.rs`
- Modify: `src/cfs/component/http_client/mod.rs` (drop the `pub mod v3;` line)
- Delete: `src/cfs/component/http_client/v3/mod.rs`

- [ ] **Step 1: Inventory existing v3 methods**

```bash
grep -nE "pub async fn cfs_component_v3_" src/cfs/component/http_client/v3/mod.rs
```
Expected 10 entries:
```
cfs_component_v3_get_options
cfs_component_v3_get
cfs_component_v3_get_single_by_id
cfs_component_v3_get_query_batch
cfs_component_v3_get_query
cfs_component_v3_patch_component
cfs_component_v3_patch_component_list
cfs_component_v3_put_component
cfs_component_v3_put_component_list
cfs_component_v3_delete_single_component
```

- [ ] **Step 2: Check dispatcher_conv coupling**

```bash
wc -l src/cfs/component/http_client/v3/dispatcher_conv.rs
```
Same decision rule as Task 3 Step 2.

- [ ] **Step 3: Create the wrapper file**

Create `src/cfs/wrapper/v3/component.rs` following the same pattern as Task 3 Step 3. Per-method routing decisions documented in the module docstring. Pay particular attention to `cfs_component_v3_get_query_batch` — its existing implementation in `v3/mod.rs:119` uses the `parallel_batch` chunking helper from `common/http.rs`. Likely candidates to keep on raw reqwest: the parallel/batched methods (they aren't single-endpoint operations) and any method that returns a `reqwest::Response`. The single-endpoint typed methods (`do_v3_component_get`, `do_v3_component_put`, `do_v3_component_patch`, `do_v3_component_delete`) are the cleanest candidates for progenitor routing.

- [ ] **Step 4: Decide types.rs strategy**

Same rule as Task 3 Step 4. Re-export or keep hand-written based on `v3/dispatcher_conv.rs` size.

- [ ] **Step 5: Register the wrapper file**

In `src/cfs/wrapper/v3/mod.rs`:
```rust
mod component;
```

- [ ] **Step 6: Update `src/cfs/component/http_client/mod.rs`**

Drop the `pub mod v3;` line. Keep any `pub use v3::types::*;` re-export.

- [ ] **Step 7: Delete the old http client file**

```bash
git rm src/cfs/component/http_client/v3/mod.rs
```

- [ ] **Step 8: Build and test**

```bash
cargo build 2>&1 | tail -10
cargo test --lib 2>&1 | tail -5
```

- [ ] **Step 9: Commit**

```bash
git add src/cfs/wrapper/v3/component.rs src/cfs/wrapper/v3/mod.rs \
        src/cfs/component/http_client/v3/types.rs \
        src/cfs/component/http_client/mod.rs
git commit -m "refactor(cfs): generate /cfs/v3/components wrapper from progenitor"
```

---

### Task 5: Migrate `cfs::configuration` v2 (4 methods)

**Files:**
- Create: `src/cfs/wrapper/v2/configuration.rs`
- Modify: `src/cfs/wrapper/v2/mod.rs` (add `mod configuration;`)
- Modify: `src/cfs/configuration/http_client/v2/types/mod.rs` and its sub-files
- Modify: `src/cfs/configuration/http_client/mod.rs`
- Delete: `src/cfs/configuration/http_client/v2/mod.rs`

- [ ] **Step 1: Inventory existing v2 methods**

```bash
grep -nE "pub async fn cfs_configuration_v2_" src/cfs/configuration/http_client/v2/mod.rs
```
Expected 4 entries:
```
cfs_configuration_v2_get
cfs_configuration_v2_get_all
cfs_configuration_v2_put
cfs_configuration_v2_delete
```

(There's also a `from_sat_file_serde_yaml` function in `v2/types/cfs_configuration_request.rs` — that's a constructor on the request type, not an HTTP method. Leave it in place.)

- [ ] **Step 2: Check dispatcher_conv coupling**

```bash
ls src/cfs/configuration/http_client/v2/types/
wc -l src/cfs/configuration/http_client/v2/types/dispatcher_conv.rs
```
The configuration submodule has a more elaborate types structure (separate `_request.rs`, `_response.rs`, `dispatcher_conv.rs`) than component/session. Plan adjustment: if `dispatcher_conv.rs` is large, keep all the hand-written request/response types and route only the methods through progenitor (where contracts fit).

- [ ] **Step 3: Create the wrapper file**

Create `src/cfs/wrapper/v2/configuration.rs` following the same pattern. Look up the generated names for `V2ConfigurationUpdateRequest` and whatever the response type is named in Section A.

- [ ] **Step 4: Decide types strategy**

Same rule as Tasks 3 and 4. The configuration v2 module has more hand-rolled type structure than the component/session ones, so keeping hand-written types is the more likely outcome. The request type's `from_sat_file_serde_yaml` constructor is csm-rs-specific (not in the spec) — it must stay hand-rolled regardless.

- [ ] **Step 5: Register the wrapper file**

In `src/cfs/wrapper/v2/mod.rs`:
```rust
mod configuration;
```

- [ ] **Step 6: Update `src/cfs/configuration/http_client/mod.rs`**

Drop the `pub mod v2;` line.

- [ ] **Step 7: Delete the old http client file**

```bash
git rm src/cfs/configuration/http_client/v2/mod.rs
```

- [ ] **Step 8: Build and test**

```bash
cargo build 2>&1 | tail -10
cargo test --lib 2>&1 | tail -5
```

- [ ] **Step 9: Commit**

```bash
git add src/cfs/wrapper/v2/configuration.rs src/cfs/wrapper/v2/mod.rs \
        src/cfs/configuration/http_client/v2/types/ \
        src/cfs/configuration/http_client/mod.rs
git commit -m "refactor(cfs): generate /cfs/v2/configurations wrapper from progenitor"
```

---

### Task 6: Migrate `cfs::configuration` v3 (3 methods)

**Files:**
- Create: `src/cfs/wrapper/v3/configuration.rs`
- Modify: `src/cfs/wrapper/v3/mod.rs` (add `mod configuration;`)
- Modify: `src/cfs/configuration/http_client/v3/types/mod.rs` and its sub-files
- Modify: `src/cfs/configuration/http_client/mod.rs`
- Delete: `src/cfs/configuration/http_client/v3/mod.rs`

- [ ] **Step 1: Inventory existing v3 methods**

```bash
grep -nE "pub async fn cfs_configuration_v3_" src/cfs/configuration/http_client/v3/mod.rs
```
Expected 3 entries:
```
cfs_configuration_v3_get
cfs_configuration_v3_put
cfs_configuration_v3_delete
```

(Plus `from_sat_file_serde_yaml` and `create_from_repos` in the types sub-files — both stay; they're constructors not HTTP methods.)

- [ ] **Step 2: Check dispatcher_conv coupling**

```bash
ls src/cfs/configuration/http_client/v3/types/
wc -l src/cfs/configuration/http_client/v3/types/dispatcher_conv.rs
```

- [ ] **Step 3: Create the wrapper file**

Create `src/cfs/wrapper/v3/configuration.rs` following the established pattern. The v3 surface adds a `/v3/sources` endpoint family that's NOT covered by the existing `cfs_configuration_v3_*` methods — leave those out of scope here; if needed they're a follow-up. The plan covers what exists in `src/cfs/configuration/http_client/v3/mod.rs` today.

- [ ] **Step 4: Decide types strategy**

Same rule. Likely outcome: keep hand-written types because of the request/response/dispatcher_conv split.

- [ ] **Step 5: Register the wrapper file**

In `src/cfs/wrapper/v3/mod.rs`:
```rust
mod configuration;
```

- [ ] **Step 6: Update `src/cfs/configuration/http_client/mod.rs`**

Drop the `pub mod v3;` line.

- [ ] **Step 7: Delete the old http client file**

```bash
git rm src/cfs/configuration/http_client/v3/mod.rs
```

- [ ] **Step 8: Build and test**

```bash
cargo build 2>&1 | tail -10
cargo test --lib 2>&1 | tail -5
```

- [ ] **Step 9: Commit**

```bash
git add src/cfs/wrapper/v3/configuration.rs src/cfs/wrapper/v3/mod.rs \
        src/cfs/configuration/http_client/v3/types/ \
        src/cfs/configuration/http_client/mod.rs
git commit -m "refactor(cfs): generate /cfs/v3/configurations wrapper from progenitor"
```

---

### Task 7: Migrate `cfs::session` v2 (5+ methods)

**Files:**
- Create: `src/cfs/wrapper/v2/session.rs`
- Modify: `src/cfs/wrapper/v2/mod.rs` (add `mod session;`)
- Modify: `src/cfs/session/http_client/v2/types.rs`
- Modify: `src/cfs/session/http_client/mod.rs`
- Delete: `src/cfs/session/http_client/v2/mod.rs`

- [ ] **Step 1: Inventory existing v2 methods**

```bash
grep -nE "pub async fn cfs_session_v2_" src/cfs/session/http_client/v2/mod.rs
```
Capture all of them. (The pre-survey hit at least `cfs_session_v2_get` at line 31; there are more — read the file.)

- [ ] **Step 2: Check dispatcher_conv coupling**

```bash
wc -l src/cfs/session/http_client/v2/dispatcher_conv.rs
```

- [ ] **Step 3: Create the wrapper file**

Create `src/cfs/wrapper/v2/session.rs`. Same pattern; per-method docstring rationale.

- [ ] **Step 4: Decide types.rs strategy**

Same rule as Task 3 Step 4.

- [ ] **Step 5: Register the wrapper file**

In `src/cfs/wrapper/v2/mod.rs`:
```rust
mod session;
```

- [ ] **Step 6: Update `src/cfs/session/http_client/mod.rs`**

Drop the `pub mod v2;` line.

- [ ] **Step 7: Delete the old http client file**

```bash
git rm src/cfs/session/http_client/v2/mod.rs
```

- [ ] **Step 8: Build and test**

```bash
cargo build 2>&1 | tail -10
cargo test --lib 2>&1 | tail -5
```

- [ ] **Step 9: Commit**

```bash
git add src/cfs/wrapper/v2/session.rs src/cfs/wrapper/v2/mod.rs \
        src/cfs/session/http_client/v2/types.rs \
        src/cfs/session/http_client/mod.rs
git commit -m "refactor(cfs): generate /cfs/v2/sessions wrapper from progenitor"
```

---

### Task 8: Migrate `cfs::session` v3 (5+ methods)

**Files:**
- Create: `src/cfs/wrapper/v3/session.rs`
- Modify: `src/cfs/wrapper/v3/mod.rs` (add `mod session;`)
- Modify: `src/cfs/session/http_client/v3/types.rs`
- Modify: `src/cfs/session/http_client/mod.rs`
- Delete: `src/cfs/session/http_client/v3/mod.rs`

- [ ] **Step 1: Inventory existing v3 methods**

```bash
grep -nE "pub async fn cfs_session_v3_" src/cfs/session/http_client/v3/mod.rs
```

- [ ] **Step 2: Check dispatcher_conv coupling**

```bash
wc -l src/cfs/session/http_client/v3/dispatcher_conv.rs
```

- [ ] **Step 3: Create the wrapper file**

Create `src/cfs/wrapper/v3/session.rs` following the established pattern.

- [ ] **Step 4: Decide types.rs strategy**

Same rule.

- [ ] **Step 5: Register the wrapper file**

In `src/cfs/wrapper/v3/mod.rs`:
```rust
mod session;
```

- [ ] **Step 6: Update `src/cfs/session/http_client/mod.rs`**

Drop the `pub mod v3;` line.

- [ ] **Step 7: Delete the old http client file**

```bash
git rm src/cfs/session/http_client/v3/mod.rs
```

- [ ] **Step 8: Build and test**

```bash
cargo build 2>&1 | tail -10
cargo test --lib 2>&1 | tail -5
```

- [ ] **Step 9: Commit**

```bash
git add src/cfs/wrapper/v3/session.rs src/cfs/wrapper/v3/mod.rs \
        src/cfs/session/http_client/v3/types.rs \
        src/cfs/session/http_client/mod.rs
git commit -m "refactor(cfs): generate /cfs/v3/sessions wrapper from progenitor"
```

---

## Phase 3: Final cleanup and verification

### Task 9: End-to-end verification + module docs

**Files:**
- Modify: `src/cfs/mod.rs` (append the "How this module is built" doc section)

- [ ] **Step 1: Confirm there are no remaining `*/http_client/v{2,3}/mod.rs` files in `src/cfs/`**

```bash
find src/cfs -path '*/http_client/v*/mod.rs' -print
```
Expected: empty. If anything is left, an earlier migration task missed it.

- [ ] **Step 2: Full build + test + clippy sweep**

```bash
cargo build 2>&1 | tail -5
cargo test --lib 2>&1 | tail -5
cargo test --test shasta_client_hsm 2>&1 | tail -5
cargo test --test backend_connector 2>&1 | tail -5
cargo clippy --lib 2>&1 | grep -c "warning:" || true
```
Baselines (the HSM work left them at): 157 lib / 12 shasta_client_hsm / 32 backend_connector. Confirm no regressions. New clippy warnings from `src/cfs/generated.rs` are silenced by its `#![allow(...)]` block; any other module's new warnings must be addressed.

- [ ] **Step 3: Append the codegen pipeline note to `src/cfs/mod.rs`**

Open `src/cfs/mod.rs`. After the existing module-level `//!` doc, append:

```rust
//!
//! ## How this module is built
//!
//! Wire-format types and the underlying HTTP client surface are
//! generated from `src/cfs/csm_api_docs.yaml` (OpenAPI 3.0.2). Mirrors
//! the HSM pipeline documented in [`crate::hsm`]; the only structural
//! difference is that the CFS spec is OpenAPI 3.x natively (no
//! Swagger 2.0 conversion needed), so there is no `make convert-spec`
//! step for CFS.
//!
//! 1. `build.rs` runs `progenitor` on the YAML and writes the
//!    generated client to `$OUT_DIR/cfs_generated.rs`.
//! 2. `src/cfs/generated.rs` `include!`s the file as a `pub(crate)`
//!    module — only the wrapper layer and `types.rs` re-export aliases
//!    are allowed to touch it.
//! 3. `src/cfs/wrapper/` glues the generated client (and where the
//!    spec/contract drifts, raw `reqwest` calls) to the public
//!    `ShastaClient::cfs_*` API. Per-method routing decisions
//!    (progenitor vs raw `reqwest`) are documented in each per-resource
//!    file's module docstring.
//!
//! Per-resource `types.rs` files are either pure re-exports of
//! generated types, or hand-rolled wire types where a full swap to
//! generated types would cascade through `dispatcher_conv` bridges
//! (configuration v2/v3, possibly others). See the file-level
//! docstrings for the call.
```

- [ ] **Step 4: Commit**

```bash
git add src/cfs/mod.rs
git commit -m "docs(cfs): describe the progenitor-driven codegen pipeline in module docs"
```

- [ ] **Step 5: Verify final git history matches the migration order**

```bash
git log --oneline | head -15
```
Expected: a sequence of commits corresponding to Tasks 0 → 9 in order.

- [ ] **Step 6: Downstream check**

```bash
test -d ../manta && (cd ../manta && cargo check 2>&1 | tail -5) || echo "no ../manta — skip"
```
Manta's `.cargo/config.toml` path-overrides csm-rs to the local checkout; this exercises the new generated code from the consumer side. (Note: `manta-server` running on `:8080` won't pick up the change until restarted.)

---

## Self-review notes (kept for the executor)

- **Spec coverage**: Every section of the CFS YAML (paths under `/v2`, `/v3`, root) maps to a task. v2/v3 components → Tasks 3/4. v2/v3 configurations → Tasks 5/6. v2/v3 sessions → Tasks 7/8. Root operations (`/versions`, `/healthz`, `/v2`, `/v3`) are exercised in Task 0's wiremock smoke test but not migrated because csm-rs's existing public API doesn't expose them. If a future need arises, add them as a follow-up. The `/v3/sources` endpoints are not in csm-rs's existing public API either; same disposition — out of scope for this plan.
- **No placeholders**: Every "fill in" marker is inside the Task 0 reference document, which is the artefact the engineer populates by inspecting the generated file. Subsequent tasks use the existing source as the authoritative naming reference, with the plan's verbatim names as starting points.
- **Type consistency**: All wrapper method names match the historical `cfs_*` naming. The CFS reference doc (Section A) is the single source of truth for generated type names; tasks reference it consistently.
- **Why no migration of `cfs::cleanup.rs`, `cfs::cleanup_session.rs`, `cfs::common.rs`, `cfs::health.rs`**: these are csm-rs-specific orchestration code (multi-call workflows, polling, version negotiation), not direct endpoint bindings. They consume the `cfs_*` methods we're migrating, so they don't need their own wrapper files. They may need import-path adjustments once Tasks 3–8 land — fix them as collateral edits in whichever task surfaces the breakage.
