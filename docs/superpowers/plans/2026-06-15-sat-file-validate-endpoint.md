# SAT-file Validate Endpoint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `POST /api/v1/sat-file/validate` to manta-server (delegating to a new public `csm-rs::validate_sat_file` via a new `SatTrait::validate_sat_file`), and wire `manta apply sat-file` to call it as a pre-flight check between plan-build and pre-hook.

**Architecture:** Four crates touched, mirroring the existing `apply_sat_file` layering. csm-rs gets a new public wrapper around its existing private `gather_sat_apply_data` + `validate_sat_file_sections`. manta-backend-dispatcher extends `SatTrait` with a `validate_sat_file` method (default `Err`). manta-server impls the trait, exposes the handler, and registers it in `routes.rs` + utoipa. manta-cli regenerates its openapi client and inserts the pre-flight call.

**Tech Stack:** Rust 2024 edition, axum 0.8, utoipa, progenitor (build-time codegen), tokio, wiremock, serde_json, serde_yaml.

**Branches (already created):**
- `csm-rs`: `feat/sat-file-validate-endpoint` (currently has the design spec commit `f4d33f7`)
- `manta-backend-dispatcher`: `feat/sat-file-validate-endpoint`
- `manta`: `feat/sat-file-validate-endpoint`

**Source of truth:** `docs/superpowers/specs/2026-06-15-sat-file-validate-endpoint-design.md` in csm-rs.

---

## Task 1: csm-rs — add `ValidateSatFileParams` + `pub validate_sat_file`

**Files:**
- Modify: `/Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs/src/commands/i_apply_sat_file/command.rs`

- [ ] **Step 1: Add `ValidateSatFileParams` struct and `pub async fn validate_sat_file` at the end of the file**

After the existing `validate_sat_file_sections` function (around line 350+, after the closing brace), append:

```rust
/// Parameters for [`validate_sat_file`].
///
/// Subset of the inputs `apply_sat_file::exec` accepts — only the
/// fields the gather + validate pipeline actually reads. The fields
/// the `process_*` phases need (gitea_*, ansible_*, reboot,
/// watch_logs, timestamps, debug_on_failure, overwrite, dry_run) are
/// filled with defaults inside the wrapper so callers don't have to
/// supply junk values.
pub struct ValidateSatFileParams<'a> {
  pub shasta_token: &'a str,
  pub shasta_base_url: &'a str,
  pub shasta_root_cert: &'a [u8],
  pub socks5_proxy: Option<&'a str>,
  pub vault_base_url: &'a str,
  pub site_name: &'a str,
  pub k8s_api_url: &'a str,
  pub hsm_group_available_vec: &'a [String],
  pub sat_template_file_yaml: serde_yaml::Value,
}

/// Validate a SAT file against the live CSM state without mutating
/// anything.
///
/// Public entry point that wraps the private `gather_sat_apply_data`
/// + `validate_sat_file_sections` pair: fetches the k8s
/// `cray-product-catalog` ConfigMap and the current CFS / IMS /
/// recipe lists, parses the SAT YAML, and runs the same per-section
/// validators the apply pipeline runs.
///
/// Returns `Ok(())` if the SAT file would apply cleanly given the
/// current CSM state; returns the first validation [`Error`]
/// encountered otherwise (fail-fast — see the design doc).
///
/// `shasta_k8s_secrets` is the Vault-fetched k8s credential blob;
/// taken as a separate argument to mirror `apply_sat_file::exec`'s
/// signature and avoid coupling csm-rs to a Vault client.
pub async fn validate_sat_file(
  params: ValidateSatFileParams<'_>,
  shasta_k8s_secrets: serde_json::Value,
) -> Result<(), Error> {
  // Reuse the existing context struct. Fields not read by the
  // gather + validate path get empty defaults; the validator never
  // reaches the apply phase so these stay inert.
  let ctx = SatApplyContext {
    shasta_token: params.shasta_token,
    shasta_base_url: params.shasta_base_url,
    shasta_root_cert: params.shasta_root_cert,
    socks5_proxy: params.socks5_proxy,
    vault_base_url: params.vault_base_url,
    site_name: params.site_name,
    k8s_api_url: params.k8s_api_url,
    gitea_base_url: "",
    gitea_token: "",
    hsm_group_available_vec: params.hsm_group_available_vec,
    ansible_verbosity: None,
    ansible_passthrough: None,
    reboot: false,
    watch_logs: false,
    timestamps: false,
    debug_on_failure: false,
    overwrite: false,
    dry_run: true,
  };

  let (sat_file, cray_product_catalog, configuration_vec, image_vec, ims_recipe_vec) =
    gather_sat_apply_data(
      &ctx,
      shasta_k8s_secrets,
      &params.sat_template_file_yaml,
    )
    .await?;

  validate_sat_file_sections(
    &ctx,
    &sat_file,
    &cray_product_catalog,
    image_vec,
    configuration_vec,
    ims_recipe_vec,
  )
  .await
}
```

- [ ] **Step 2: Confirm csm-rs builds**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
cargo build
```

Expected: clean build, no errors. Warnings about `validate_sat_file` being unused are OK at this point — it's consumed externally.

- [ ] **Step 3: Confirm existing csm-rs tests still pass**

```bash
cargo test
```

Expected: 244 passed, 0 failed, 2 ignored (the pre-existing baseline from the prior merge).

- [ ] **Step 4: Commit**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
git add src/commands/i_apply_sat_file/command.rs
git commit -m "$(cat <<'EOF'
feat(sat): add pub validate_sat_file wrapper

New public entry point that runs the existing gather +
validate_sat_file_sections pipeline without any subsequent apply
work. Lets callers (e.g. manta-server's new /sat-file/validate
endpoint) check a SAT file against live CSM state without
constructing the private SatApplyContext or reproducing the gather
orchestration themselves.

Spec: docs/superpowers/specs/2026-06-15-sat-file-validate-endpoint-design.md
EOF
)"
```

---

## Task 2: manta-backend-dispatcher — add `ValidateSatFileParams` + `SatTrait::validate_sat_file`

**Files:**
- Modify: `/Users/masber/Documents/DATA_REPOS/CODING/rust/manta-backend-dispatcher/src/interfaces/apply_sat_file.rs`

- [ ] **Step 1: Append `ValidateSatFileParams` after the existing param structs (after `ApplySessionTemplateParams`, before `pub trait SatTrait`)**

```rust
/// Parameters for [`SatTrait::validate_sat_file`].
///
/// Subset of [`ApplySatFileParams`] — only the fields the validator
/// reads. The SAT content travels as a structured
/// `serde_json::Value` end-to-end, exactly like
/// [`ApplySatFileParams::sat_file`].
pub struct ValidateSatFileParams<'a> {
  /// Bearer token authenticating the caller against the backend
  /// (and against Vault when the backend fetches k8s creds).
  pub shasta_token: &'a str,
  /// Vault base URL — used to fetch the k8s secrets needed to read
  /// the `cray-product-catalog` ConfigMap.
  pub vault_base_url: &'a str,
  /// Site identifier used to namespace the Vault path
  /// (`manta/data/<site_name>/k8s`).
  pub site_name: &'a str,
  /// Kubernetes API URL — used to talk to the in-cluster product
  /// catalog after the k8s creds have been fetched from Vault.
  pub k8s_api_url: &'a str,
  /// Parsed SAT file as a structured value (identical shape to
  /// [`ApplySatFileParams::sat_file`]).
  pub sat_file: serde_json::Value,
  /// HSM groups the caller is allowed to target; SAT files
  /// referencing groups outside this slice are rejected.
  pub hsm_group_available_vec: &'a [String],
}
```

- [ ] **Step 2: Add the trait method to `SatTrait` (just below `apply_sat_file`, before `apply_configuration`)**

```rust
  /// Validate a SAT file against the backend's live state without
  /// mutating anything.
  ///
  /// Pre-flight check: returns `Ok(())` if the SAT file would
  /// apply cleanly given the current state, or `Err(...)` with the
  /// first detected mismatch (fail-fast). See the
  /// `2026-06-15-sat-file-validate-endpoint` design doc for the
  /// shape of the validator's checks.
  ///
  /// The default implementation returns
  /// [`Error::Message`](crate::error::Error::Message) so backends
  /// that don't support SAT validation can be plugged in without
  /// implementing the method.
  fn validate_sat_file(
    &self,
    _params: ValidateSatFileParams<'_>,
  ) -> impl Future<Output = Result<(), Error>> + Send {
    async {
      Err(Error::Message(
        "Validate SAT file command not implemented for this backend".to_string(),
      ))
    }
  }
```

- [ ] **Step 3: Confirm manta-backend-dispatcher builds and tests pass**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta-backend-dispatcher
cargo build
cargo test
```

Expected: clean build, all existing tests pass.

- [ ] **Step 4: Commit**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta-backend-dispatcher
git add src/interfaces/apply_sat_file.rs
git commit -m "$(cat <<'EOF'
feat(sat): add SatTrait::validate_sat_file

Default impl returns `Err("not implemented")` so existing backends
that don't yet support validation can still satisfy the trait.
CSM backend implements it in manta-server; ochami impl deferred
until ochami needs apply support.

Companion to csm-rs's new pub validate_sat_file wrapper.
EOF
)"
```

---

## Task 3: manta-shared — add `PostSatValidateRequest` wire type

**Files:**
- Modify: `/Users/masber/Documents/DATA_REPOS/CODING/rust/manta/crates/manta-shared/src/types/api/sat_file.rs`

- [ ] **Step 1: Find the location for the new struct**

Look at the existing types in the file:

```bash
grep -n "pub struct Post\|pub struct Create\|pub struct Stamp" /Users/masber/Documents/DATA_REPOS/CODING/rust/manta/crates/manta-shared/src/types/api/sat_file.rs
```

Add the new struct following the same convention — `#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]`.

- [ ] **Step 2: Add the struct**

Append after the last existing `pub struct PostSat...` block in the file:

```rust
/// Body for `POST /api/v1/sat-file/validate`.
///
/// The CLI parses the SAT YAML once into a `serde_json::Value` and
/// forwards it verbatim; the server hands it through to csm-rs's
/// validator without re-parsing.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostSatValidateRequest {
  /// Parsed SAT file content. Same shape as
  /// `PostSatConfigurationRequest::configuration` aggregated into
  /// the full SAT document (`{ configurations, images,
  /// session_templates, hardware }`).
  pub sat_file: serde_json::Value,
}
```

- [ ] **Step 3: Build manta-shared**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta
cargo build -p manta-shared
```

Expected: clean build.

- [ ] **Step 4: Commit**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta
git add crates/manta-shared/src/types/api/sat_file.rs
git commit -m "feat(sat): add PostSatValidateRequest wire type"
```

---

## Task 4: manta-server — add `extract_all_target_groups` helper

The handler needs to combine HSM groups across `images[]` and `session_templates[]` entries to feed `validate_user_group_vec_access`. Use TDD.

**Files:**
- Modify: `/Users/masber/Documents/DATA_REPOS/CODING/rust/manta/crates/manta-server/src/service/sat_groups.rs`

- [ ] **Step 1: Write failing tests**

At the end of the existing `mod tests` block in `sat_groups.rs` (find it with `grep -n "mod tests" .../sat_groups.rs`), add:

```rust
  #[test]
  fn extract_all_target_groups_empty_sat_file_returns_empty() {
    let sat = json!({});
    assert!(super::extract_all_target_groups(&sat).is_empty());
  }

  #[test]
  fn extract_all_target_groups_collects_from_images_and_templates() {
    let sat = json!({
      "images": [
        { "name": "img-1", "configuration_group_names": ["compute", "uan"] },
        { "name": "img-2", "configuration_group_names": ["compute"] },
      ],
      "session_templates": [
        {
          "name": "st-1",
          "bos_parameters": {
            "boot_sets": {
              "compute": { "node_groups": ["compute"] },
              "uan":     { "node_groups": ["uan", "admin"] },
            }
          }
        }
      ]
    });
    let mut got = super::extract_all_target_groups(&sat);
    got.sort();
    assert_eq!(got, vec!["admin", "compute", "uan"]);
  }

  #[test]
  fn extract_all_target_groups_handles_missing_sections() {
    let sat = json!({ "images": [ { "name": "img", "configuration_group_names": ["g1"] } ] });
    let got = super::extract_all_target_groups(&sat);
    assert_eq!(got, vec!["g1"]);
  }
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta
cargo test -p manta-server extract_all_target_groups
```

Expected: FAIL — `cannot find function 'extract_all_target_groups' in module 'super'`.

- [ ] **Step 3: Implement `extract_all_target_groups`**

Add to `sat_groups.rs` between `extract_session_template_groups` and the `#[cfg(test)] mod tests` block:

```rust
/// Read every HSM group name referenced anywhere in a SAT file —
/// across all `images[]` and `session_templates[]` entries —
/// deduplicated.
///
/// Returns an empty `Vec` for a SAT file with no groups (or no
/// images / session_templates sections at all).
///
/// Used by [`crate::server::handlers::sat_file::post_sat_validate`]
/// to enforce HSM-group access before delegating to the backend.
pub fn extract_all_target_groups(sat_file: &Value) -> Vec<String> {
  let mut groups: Vec<String> = Vec::new();

  if let Some(images) = sat_file.get("images").and_then(Value::as_array) {
    for image in images {
      groups.extend(extract_image_groups(image));
    }
  }

  if let Some(templates) =
    sat_file.get("session_templates").and_then(Value::as_array)
  {
    for tpl in templates {
      groups.extend(extract_session_template_groups(tpl));
    }
  }

  groups.sort();
  groups.dedup();
  groups
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -p manta-server extract_all_target_groups
```

Expected: all three new tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/manta-server/src/service/sat_groups.rs
git commit -m "feat(sat): extract_all_target_groups across whole sat file"
```

---

## Task 5: csm-rs — impl `SatTrait::validate_sat_file` for `ShastaClient`

The CSM-backend impl lives in csm-rs (not manta-server) — see existing
`impl SatTrait for ShastaClient` at `src/backend_connector/sat.rs:63`. The
module is gated behind the `manta-dispatcher` feature, which manta enables
via path override.

**Depends on:** Tasks 1 + 2 (the new csm-rs `validate_sat_file` and the new
manta-backend-dispatcher trait method).

**Files:**
- Modify: `/Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs/src/backend_connector/sat.rs`

- [ ] **Step 1: Extend the trait-method import block**

Find the existing import (line 39-43):

```rust
    apply_sat_file::{
      ApplyConfigurationParams, ApplyImageCreateSessionParams,
      ApplyImageParams, ApplyImageStampParams, ApplySatFileParams,
      ApplySessionTemplateParams, SatTrait,
    },
```

Add `ValidateSatFileParams`:

```rust
    apply_sat_file::{
      ApplyConfigurationParams, ApplyImageCreateSessionParams,
      ApplyImageParams, ApplyImageStampParams, ApplySatFileParams,
      ApplySessionTemplateParams, SatTrait, ValidateSatFileParams,
    },
```

- [ ] **Step 2: Add the trait-method impl just after `apply_sat_file`**

Mirrors the existing `apply_sat_file` body (Vault fetch + YAML transcode +
delegate to a function in `commands::i_apply_sat_file::command`), but
delegates to the new public `validate_sat_file` and returns
`Result<(), Error>`. Insert after the closing brace of `apply_sat_file`
(around line 140), before `apply_configuration`:

```rust
  async fn validate_sat_file(
    &self,
    params: ValidateSatFileParams<'_>,
  ) -> Result<(), Error> {
    let ValidateSatFileParams {
      shasta_token,
      vault_base_url,
      site_name,
      k8s_api_url,
      sat_file,
      hsm_group_available_vec,
    } = params;

    // Same shape-transcode the apply path uses: trait carries the
    // SAT file as serde_json::Value; csm-rs's command takes
    // serde_yaml::Value. JSON ⊂ YAML, so this is lossless.
    let sat_template_file_yaml: serde_yaml::Value =
      serde_json::from_value(sat_file).map_err(|e| {
        Error::Message(format!(
          "SAT file value is not a valid YAML mapping: {e}"
        ))
      })?;

    let socks5_proxy = self.socks5_proxy.as_deref();
    let shasta_k8s_secrets = fetch_shasta_k8s_secrets_from_vault(
      vault_base_url,
      shasta_token,
      site_name,
      socks5_proxy,
    )
    .await
    .map_err(Error::from)?;

    crate::commands::i_apply_sat_file::command::validate_sat_file(
      crate::commands::i_apply_sat_file::command::ValidateSatFileParams {
        shasta_token,
        shasta_base_url: &self.base_url,
        shasta_root_cert: &self.root_cert,
        socks5_proxy,
        vault_base_url,
        site_name,
        k8s_api_url,
        hsm_group_available_vec,
        sat_template_file_yaml,
      },
      shasta_k8s_secrets,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }
```

- [ ] **Step 3: Build csm-rs with default features (which include `manta-dispatcher`)**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
cargo build
```

Expected: clean build. (This will only resolve if the local
manta-backend-dispatcher checkout — which is on the same feature
branch — exposes the new trait method. If you get
"`validate_sat_file` is not a member of trait `SatTrait`", verify that
Task 2 was committed and that any cargo path override or
`[patch.crates-io]` resolution points at the local checkout.)

- [ ] **Step 4: Confirm csm-rs tests still pass**

```bash
cargo test
```

Expected: 244 passed, 0 failed, 2 ignored.

- [ ] **Step 5: Commit**

```bash
git add src/backend_connector/sat.rs
git commit -m "feat(sat): impl SatTrait::validate_sat_file for ShastaClient"
```

---

## Task 6: manta-server — impl `SatTrait::validate_sat_file` on `StaticBackendDispatcher`

This is the dispatcher shim that fans the call out to whichever backend
(currently only `ShastaClient`) is active. The actual work lives in csm-rs
(Task 5).

**Files:**
- Modify: `/Users/masber/Documents/DATA_REPOS/CODING/rust/manta/crates/manta-server/src/backend_dispatcher/sat.rs`

- [ ] **Step 1: Confirm `ValidateSatFileParams` reaches scope**

The file does `use super::*;` and `super` is `backend_dispatcher/mod.rs`,
which re-exports the trait param types. Check that `mod.rs` already imports
`ValidateSatFileParams`:

```bash
grep -n "ValidateSatFileParams\|ApplySatFileParams" \
  /Users/masber/Documents/DATA_REPOS/CODING/rust/manta/crates/manta-server/src/backend_dispatcher/mod.rs
```

If `ApplySatFileParams` appears but `ValidateSatFileParams` does not, add it to
the same `use manta_backend_dispatcher::interfaces::apply_sat_file::{...}`
import in `mod.rs`.

- [ ] **Step 2: Add the trait method to `impl SatTrait for StaticBackendDispatcher`**

Insert after the existing `apply_sat_file` (around line 18, before
`apply_configuration`):

```rust
  async fn validate_sat_file(
    &self,
    params: ValidateSatFileParams<'_>,
  ) -> Result<(), Error> {
    dispatch!(self, validate_sat_file, params)
  }
```

- [ ] **Step 3: Build manta-server**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta
cargo build -p manta-server
```

Expected: clean build. If you get
"no method `validate_sat_file` on `ShastaClient`", Task 5 (csm-rs
impl) is incomplete or not picked up by the path override.

- [ ] **Step 4: Commit**

```bash
git add crates/manta-server/src/backend_dispatcher/sat.rs \
        crates/manta-server/src/backend_dispatcher/mod.rs
git commit -m "feat(sat): dispatcher impl of SatTrait::validate_sat_file"
```

---

## Task 7: manta-server — add `post_sat_validate` handler

**Files:**
- Modify: `/Users/masber/Documents/DATA_REPOS/CODING/rust/manta/crates/manta-server/src/server/handlers/sat_file.rs`

- [ ] **Step 1: Extend the wire-type re-export**

Find the `pub use manta_shared::types::api::sat_file::{...};` block (around line 66) and add `PostSatValidateRequest`:

```rust
pub use manta_shared::types::api::sat_file::{
  CreateImageCfsSessionRequest, PostSatConfigurationRequest,
  PostSatSessionTemplateRequest, PostSatSessionTemplateResponse,
  PostSatValidateRequest,
  StampImageFromSessionRequest,
};
```

- [ ] **Step 2: Add the backend-params import**

Extend the `use manta_backend_dispatcher::interfaces::apply_sat_file::{...}` block at the top of the file with `ValidateSatFileParams as BackendValidateSatFileParams`.

- [ ] **Step 3: Add the handler at the end of the file (before the `#[cfg(test)] mod tests` block)**

```rust
// ---------------------------------------------------------------------------
// POST /api/v1/sat-file/validate — Pre-flight validation of a whole SAT file
//   against live CSM state. Returns 204 on success, 400 on validation
//   failure. Read-only; safe to call before any state-changing apply work.
// ---------------------------------------------------------------------------

#[utoipa::path(post, path = "/sat-file/validate", tag = "sat-file",
  params(SiteHeader),
  request_body = PostSatValidateRequest,
  security(("bearerAuth" = [])),
  responses(
    (status = 204, description = "SAT file is valid"),
    (status = 400, description = "SAT validation failed",       body = ErrorResponse),
    (status = 401, description = "Unauthorized",                body = ErrorResponse),
    (status = 403, description = "Caller cannot target referenced HSM groups", body = ErrorResponse),
    (status = 501, description = "Vault or k8s not configured", body = ErrorResponse),
  )
)]
/// `POST /api/v1/sat-file/validate` — validate a SAT file against
/// live CSM state without mutating anything. Used by
/// `manta apply sat-file` as a pre-flight check.
#[tracing::instrument(skip_all)]
pub async fn post_sat_validate(
  ctx: RequestCtx,
  Json(body): Json<PostSatValidateRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
  tracing::info!("post_sat_validate");
  let infra = ctx.infra();

  let vault_base_url = require_vault(infra.vault_base_url)?;
  let k8s_api_url = require_k8s_url(infra.k8s_api_url)?;

  let target_groups =
    crate::service::sat_groups::extract_all_target_groups(&body.sat_file);

  validate_user_group_vec_access(&infra, &ctx.token, &target_groups)
    .await
    .map_err(to_handler_error)?;

  // Caller's HSM-group scope — same source used by
  // post_sat_session_template at handlers/sat_file.rs:291.
  let hsm_group_available_vec = infra
    .backend
    .get_group_name_available(&ctx.token)
    .await
    .map_err(to_handler_error)?;

  infra
    .backend
    .validate_sat_file(BackendValidateSatFileParams {
      shasta_token: &ctx.token,
      vault_base_url,
      site_name: infra.site_name,
      k8s_api_url,
      sat_file: body.sat_file,
      hsm_group_available_vec: &hsm_group_available_vec,
    })
    .await
    .map_err(to_handler_error)?;

  Ok(StatusCode::NO_CONTENT)
}
```

- [ ] **Step 4: Build manta-server**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta
cargo build -p manta-server
```

Expected: clean build.

- [ ] **Step 5: Commit (route + utoipa registration come in the next task — handler alone is unwired but compiles)**

```bash
git add crates/manta-server/src/server/handlers/sat_file.rs
git commit -m "feat(sat): add post_sat_validate handler"
```

---

## Task 8: manta-server — register route + OpenAPI entry

**Files:**
- Modify: `/Users/masber/Documents/DATA_REPOS/CODING/rust/manta/crates/manta-server/src/server/routes.rs`
- Modify: `/Users/masber/Documents/DATA_REPOS/CODING/rust/manta/crates/manta-server/src/server/api_doc.rs`

- [ ] **Step 1: Register the route**

In `routes.rs`, find the `/sat-file/session-templates` route registration (around line 146-149) and add the new line right after it:

```rust
    .route(
      "/sat-file/validate",
      post(handlers::post_sat_validate),
    )
```

- [ ] **Step 2: Add to OpenAPI `paths(...)`**

In `api_doc.rs`, find the `paths(...)` macro args and add `handlers::post_sat_validate` next to the other `post_sat_*` entries.

- [ ] **Step 3: Add `PostSatValidateRequest` to `components(...)` schemas**

In the same file, find the `components(schemas(...))` block and add `PostSatValidateRequest` next to the other `PostSat*Request` entries.

- [ ] **Step 4: Build and run --emit-openapi to verify the new path appears**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta
cargo build -p manta-server
cargo run -p manta-server -- --emit-openapi | grep -A2 "sat-file/validate"
```

Expected: the JSON output includes the new path with operationId `post_sat_validate` and the expected response codes.

- [ ] **Step 5: Commit**

```bash
git add crates/manta-server/src/server/routes.rs crates/manta-server/src/server/api_doc.rs
git commit -m "feat(sat): wire POST /sat-file/validate route + utoipa entry"
```

---

## Task 9: manta-server — wire-format-lock test for `PostSatValidateRequest`

**Files:**
- Modify: `/Users/masber/Documents/DATA_REPOS/CODING/rust/manta/crates/manta-server/src/server/handlers/sat_file.rs`

- [ ] **Step 1: Add the test to the existing `mod tests` block**

Find the existing test imports near line 320:

```rust
  use super::{
    CreateImageCfsSessionRequest, PostSatConfigurationRequest,
    PostSatSessionTemplateRequest, PostSatSessionTemplateResponse,
    StampImageFromSessionRequest,
  };
```

Extend to include `PostSatValidateRequest`:

```rust
  use super::{
    CreateImageCfsSessionRequest, PostSatConfigurationRequest,
    PostSatSessionTemplateRequest, PostSatSessionTemplateResponse,
    PostSatValidateRequest,
    StampImageFromSessionRequest,
  };
```

Then add a wire-lock test at the end of the `mod tests` block:

```rust
  /// Lock the shape of the CLI's POST /sat-file/validate body.
  /// Catches renames on either side of the wire.
  #[test]
  fn cli_validate_body_deserialises() {
    let cli_body = serde_json::json!({
      "sat_file": {
        "configurations": [{ "name": "cfg-v1" }],
        "images": [],
        "session_templates": [],
      }
    });
    let req: PostSatValidateRequest =
      serde_json::from_value(cli_body).unwrap();
    assert!(req.sat_file.get("configurations").is_some());
  }
```

- [ ] **Step 2: Run tests**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta
cargo test -p manta-server cli_validate_body_deserialises
```

Expected: passes.

- [ ] **Step 3: Confirm the full manta-server suite stays green**

```bash
cargo test -p manta-server
```

Expected: no regressions.

- [ ] **Step 4: Commit**

```bash
git add crates/manta-server/src/server/handlers/sat_file.rs
git commit -m "test(sat): wire-format lock for PostSatValidateRequest"
```

---

## Task 10: manta-cli — regenerate `openapi.json` and verify codegen

**Files:**
- Modify: `/Users/masber/Documents/DATA_REPOS/CODING/rust/manta/crates/manta-cli/openapi.json` (regenerated)

- [ ] **Step 1: Regenerate the spec**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta
cargo run -p manta-server -- --emit-openapi > crates/manta-cli/openapi.json
```

- [ ] **Step 2: Verify the diff includes only the new endpoint**

```bash
git diff -- crates/manta-cli/openapi.json | grep -E "^[-+]" | head -40
```

Expected: only additions related to `post_sat_validate` and `PostSatValidateRequest`. If you see unrelated diffs (renamed fields, reordered keys), investigate — they may signal an unintended schema change elsewhere.

- [ ] **Step 3: Build manta-cli — build.rs auto-regenerates the openapi client**

```bash
cargo build -p manta-cli
```

Expected: clean build. Confirm `post_sat_validate` is now generated:

```bash
grep -n "post_sat_validate" target/debug/build/manta-cli-*/out/openapi_client.rs | head -3
```

Expected: at least one match (the generated client method).

- [ ] **Step 4: Commit**

```bash
git add crates/manta-cli/openapi.json
git commit -m "build(cli): regenerate openapi.json for /sat-file/validate"
```

---

## Task 11: manta-cli — add pre-flight call in `exec.rs`

**Files:**
- Modify: `/Users/masber/Documents/DATA_REPOS/CODING/rust/manta/crates/manta-cli/src/dispatch/apply/sat_file/exec.rs`

- [ ] **Step 1: Add the import for the new request type**

Find the existing `use crate::openapi_client::types::{...}` import (or wherever `PostSatConfigurationRequest` is imported in `dispatch.rs`) and verify that `PostSatValidateRequest` will be available. Add to the imports at the top of `exec.rs`:

```rust
use crate::openapi_client::types::PostSatValidateRequest;
use crate::http_client::OpenApiResultExt;
```

- [ ] **Step 2: Move `MantaClient` construction before step 7 and add the pre-flight call**

Find lines 153-154 (the current `MantaClient::from_app_ctx` line). Move that construction up to just after step 6 (the plan-shape log), and insert the pre-flight call. Replace the section from line ~146 to line ~154 with:

```rust
  // 6a. Pre-flight: server-side validation against live CSM state.
  //     Built first so the same client is reused for dispatch below.
  //     Failing here aborts before the pre-hook fires.
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .post_sat_validate(
      client.site_name(),
      &PostSatValidateRequest { sat_file: sat_file.clone() },
    )
    .await
    .into_anyhow()
    .context("Server-side SAT validation failed")?;
  tracing::info!("SAT file validated server-side");

  // 7. Pre-hook -> server call -> post-hook.
  crate::common::hooks::run_hook_if_present(opts.prehook_opt, "pre")?;

  // 7a. Dispatch the plan element-by-element. The CLI accumulates
  //     `ref_name → image_id` across calls and builds the same
  //     four-list response the legacy endpoint used to return.
  let result = dispatch::dispatch_plan(ctx, &client, plan, opts).await?;
```

(Confirm `Context` is in scope from the existing `use anyhow::{Context, ...}` at the top of the file — line 41.)

- [ ] **Step 3: Build manta-cli**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta
cargo build -p manta-cli
```

Expected: clean build.

- [ ] **Step 4: Confirm existing CLI surface tests still pass**

```bash
cargo test -p manta-cli
```

Expected: no regressions (existing `cli_tests.rs` is `--help` / `--version` smoke coverage; the apply flow is not exercised in unit tests).

- [ ] **Step 5: Commit**

```bash
git add crates/manta-cli/src/dispatch/apply/sat_file/exec.rs
git commit -m "$(cat <<'EOF'
feat(cli): pre-flight SAT validation in apply sat-file

Calls the new POST /sat-file/validate endpoint between plan-build
and pre-hook. Surfaces live-CSM-state mismatches before any
state-changing work begins, and avoids firing the operator's
pre-hook on a SAT file that the server will reject.

MantaClient construction moves up from step 7a to step 6a so the
same client is used for validation and dispatch.
EOF
)"
```

---

## Task 12: end-to-end build sweep + review handoff

Verifies all four crates build and test together with the path overrides active.

- [ ] **Step 1: csm-rs**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/csm-rs
cargo build && cargo test
```

Expected: 244 passed, 0 failed, 2 ignored.

- [ ] **Step 2: manta-backend-dispatcher**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta-backend-dispatcher
cargo build && cargo test
```

Expected: clean build, all existing tests pass.

- [ ] **Step 3: manta workspace (server + cli + shared)**

```bash
cd /Users/masber/Documents/DATA_REPOS/CODING/rust/manta
cargo build --workspace
cargo test --workspace
```

Expected: all green, including the new wire-lock test and the existing CLI surface tests.

- [ ] **Step 4: Verify the new endpoint shape end-to-end via emitted spec**

```bash
cargo run -p manta-server -- --emit-openapi | jq '.paths."/sat-file/validate"' | head -30
```

Expected: JSON object describing the post operation with the 204/400/401/403/501 responses.

- [ ] **Step 5: Stop here for review**

Do NOT push or open PRs in this task. The implementation is locally complete; pushing and PR creation is a separate decision the user owns. Report:

- Branch name: `feat/sat-file-validate-endpoint` in csm-rs, manta-backend-dispatcher, manta.
- New endpoint: `POST /api/v1/sat-file/validate` returns 204 on success, 400 on validation failure.
- New CLI behaviour: `manta apply sat-file` runs the pre-flight before the pre-hook.
- Commits per repo and any noteworthy diffs.

Ask the user whether to push to remotes / open PRs.

---

## Notes for the executing agent

- The path-override at `manta/.cargo/config.toml` (already committed, in `.gitignore`) makes manta-server / manta-cli pick up local csm-rs and manta-backend-dispatcher changes automatically. No version bumps or `[patch.crates-io]` edits are needed during this work.
- The current branch state for csm-rs already has the design spec commit `f4d33f7` and the prior merge commit `6986541` from the `feat/progenitor-hsm-codegen` merge. Don't rebase / squash these.
- If any task fails its expected build/test, fix in place — do not paper over with `#[allow(dead_code)]` or comment-out. A real failure indicates either a wrong signature or a missing piece in a previous task; check the previous task's expected output before moving on.
- The plan deliberately stops short of pushing branches or opening PRs — the user controls that decision.
