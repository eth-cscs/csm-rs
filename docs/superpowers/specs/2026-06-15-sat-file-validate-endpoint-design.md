# SAT-file validation endpoint

**Status**: Design — awaiting review
**Date**: 2026-06-15
**Branch**: `feat/sat-file-validate-endpoint` (csm-rs, manta-backend-dispatcher, manta)
**Scope**: csm-rs, manta-backend-dispatcher, manta-server, manta-cli

## Problem

`manta apply sat-file` today builds an execution plan client-side and
then dispatches the plan element-by-element through the per-element
SAT endpoints in manta-server (`/sat-file/configurations`,
`/sat-file/images/cfs-session`, `/sat-file/images/stamp`,
`/sat-file/session-templates`). The CLI does some up-front client-side
checks while building the plan — cross-reference resolution between
`configurations`, `images`, and `session_templates` entries — but it
cannot validate the SAT file against **live CSM state**: whether the
referenced product layers exist in the `cray-product-catalog` k8s
ConfigMap, whether the referenced base images exist in IMS, whether
the referenced configurations already exist in CFS, etc.

The first time this kind of mistake is caught today is mid-apply, when
csm-rs's apply pipeline runs `validate_sat_file_sections` and returns
`Err(...)`. By then the operator has already confirmed the rendered
SAT preview, accepted the reboot prompt, and run their pre-hook.
Discovery is late and the failure surfaces partway through dispatch.

We want a pre-flight check that fails fast — before any state-changing
work is committed — and lets the operator see and fix CSM-state
mismatches before they trigger a partial apply.

## Goal

Expose csm-rs's existing `validate_sat_file_sections` as an HTTP
endpoint on manta-server and have `manta apply sat-file` call it
between plan-build and pre-hook. On failure the CLI aborts before
pre-hook runs and before any per-element dispatch begins.

## Non-goals

- Rewriting `validate_sat_file_sections` to accumulate issues instead
  of failing on the first one. The endpoint surfaces whatever the
  existing validator returns; richer reporting can come later.
- A `/sat-file/{configurations,images,session-templates}/validate`
  per-section split. The underlying csm-rs validator is whole-file;
  per-section endpoints would require a non-trivial csm-rs refactor.
- A standalone "lint the YAML" mode that doesn't need CSM
  connectivity. The validator inherently requires live CFS, IMS and
  k8s product-catalog reads to do its job.
- An ochami-backend implementation of the new trait method. The
  default `Err("not implemented")` is fine until ochami needs apply
  support, at which point validate is part of the same scope.

## Architecture

Four crates, mirroring the existing `apply_sat_file` layering:

```
csm-rs                      manta-backend-dispatcher       manta-server                       manta-cli
─────────                   ────────────────────────       ─────────────                      ─────────
pub async fn                pub struct                     impl SatTrait::validate_sat_file   apply::sat_file::exec.rs:
  validate_sat_file(          ValidateSatFileParams<'a>      for StaticBackendDispatcher      new step 6a between
    params,                                                  └─ dispatch! to csm-rs           plan-build and pre-hook:
  ) -> Result<(), Error>    SatTrait::validate_sat_file                                         client.post_sat_validate(..)
                              (default = "not implemented") POST /api/v1/sat-file/validate      .await?
                                                            ├─ handler: post_sat_validate      (mandatory; no skip flag)
                                                            ├─ route in routes.rs
                                                            └─ utoipa entry in api_doc.rs
```

### Path

`POST /api/v1/sat-file/validate`

### Auth & scoping

Matches the existing `apply_sat_file` endpoints exactly:

- Bearer token required (read by the existing auth extractor).
- HSM-group access enforced: a SAT file referencing a group outside
  the caller's `hsm_group_available_vec` is rejected with 403, using
  the same `validate_user_group_vec_access` check the existing sat
  handlers run. Prevents the "can validate what I can't apply"
  footgun.

### Why a wrapper in csm-rs

`validate_sat_file_sections` is private and takes a private
`SatApplyContext` plus pre-fetched live-CSM snapshots (CFS configs,
IMS images, IMS recipes) and the k8s `cray-product-catalog` ConfigMap.
Exposing the function alone would force every caller to reproduce the
fetching orchestration that `gather_sat_apply_data` already does
inside `command.rs`, and to construct a `SatApplyContext` whose 5+
unused fields (`gitea_*`, `ansible_*`, `reboot`, `watch_logs`,
`timestamps`, `debug_on_failure`, `overwrite`, `dry_run`) would have
to be filled in with junk.

The new `validate_sat_file` wraps gather + validate into one public
entry point, so manta-server (and any future caller) hands in a small
params struct of "things the validator actually needs" and gets back
a single `Result<(), Error>`. No existing public surface changes.

## Components & contracts

### csm-rs

```rust
// src/commands/i_apply_sat_file/command.rs (added alongside apply_sat_file)
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

pub async fn validate_sat_file(
  params: ValidateSatFileParams<'_>,
  shasta_k8s_secrets: serde_json::Value,
) -> Result<(), Error>
```

Internally builds a private `SatApplyContext` (filling the
validator-irrelevant fields with empty `""` / `&[]`) → calls the
existing private `gather_sat_apply_data` → calls the existing private
`validate_sat_file_sections`. Neither private item becomes public.

### manta-backend-dispatcher

```rust
// src/interfaces/apply_sat_file.rs (added below ApplySatFileParams)
pub struct ValidateSatFileParams<'a> {
  pub shasta_token: &'a str,
  pub vault_base_url: &'a str,
  pub site_name: &'a str,
  pub k8s_api_url: &'a str,
  pub sat_file: serde_json::Value,    // wire-format JSON, identical to ApplySatFileParams.sat_file
  pub hsm_group_available_vec: &'a [String],
}

// In trait SatTrait { … }
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

Default `Err("not implemented")` matches every other method on
`SatTrait`. Only `StaticBackendDispatcher`'s CSM impl overrides it for
now.

### manta-server

`src/backend_dispatcher/sat.rs` — one-line trait impl:

```rust
async fn validate_sat_file(
  &self,
  params: ValidateSatFileParams<'_>,
) -> Result<(), Error> {
  dispatch!(self, validate_sat_file, params)
}
```

`src/server/handlers/sat_file.rs` — new handler alongside existing
`post_sat_*`:

```rust
#[derive(Deserialize, ToSchema)]
pub struct PostSatValidateRequest {
  pub sat_file: serde_json::Value,
}

pub async fn post_sat_validate(
  ctx: RequestCtx,
  SiteHeader(site_name): SiteHeader,
  Json(req): Json<PostSatValidateRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
  let vault_base_url = require_vault(&ctx)?;
  let k8s_api_url = require_k8s_url(&ctx)?;

  validate_user_group_vec_access(&req.sat_file, &ctx.hsm_group_available_vec)
    .map_err(to_handler_error(StatusCode::FORBIDDEN))?;

  ctx.infra
    .validate_sat_file(BackendValidateSatFileParams {
      shasta_token: &ctx.token,
      vault_base_url,
      site_name: &site_name,
      k8s_api_url,
      sat_file: req.sat_file,
      hsm_group_available_vec: &ctx.hsm_group_available_vec,
    })
    .await
    .map_err(to_handler_error(StatusCode::BAD_REQUEST))?;

  Ok(StatusCode::NO_CONTENT)
}
```

(Exact extractor names follow whatever the existing sat handlers use;
the snippet shows shape and intent.)

`src/server/routes.rs` — new line next to the other `/sat-file/*`
routes:

```rust
.route("/sat-file/validate", post(handlers::post_sat_validate))
```

`src/server/api_doc.rs` — append `handlers::post_sat_validate` to the
`paths(…)` list and `PostSatValidateRequest` to the `components(…)`
schemas list.

### manta-cli

`src/dispatch/apply/sat_file/exec.rs` — new step 6a, inserted between
the existing step 6 (plan logged) and step 7 (pre-hook):

```rust
// 6a. Pre-flight: server-side validation against live CSM state.
//     Fails fast before the pre-hook fires.
let client = MantaClient::from_app_ctx(ctx, Some(token))?;
client.openapi
  .post_sat_validate(
    client.site_name(),
    &PostSatValidateRequest { sat_file: sat_file.clone() },
  )
  .await
  .into_anyhow_with_context("Server-side SAT validation failed")?;
tracing::info!("SAT file validated server-side");
```

The `MantaClient::from_app_ctx` construction moves from current step
7a up to step 6a so the same client is used for both validation and
dispatch.

The pre-flight is mandatory — no `--skip-validate` flag. The
client-side cross-reference checks in `plan::build_plan` are
unaffected and still run before the server is contacted; they catch
issues that don't need a round trip.

## Data flow

```
manta-cli exec.rs                       manta-server                           csm-rs
  step 6a (after plan build):           POST /api/v1/sat-file/validate         validate_sat_file(params)
  ─────────────────────────────         ──────────────────────────             ───────────────────────
  PostSatValidateRequest {                                                      1. gather_sat_apply_data
    sat_file: sat_file.clone(),         1. extract token, site, vault,             ├─ k8s client (via Vault)
  }                                        k8s url, hsm_group_access              ├─ try_get_configmap
  client.openapi                        2. validate_user_group_vec_access         │   (cray-product-catalog)
    .post_sat_validate(site, &body)        on req.sat_file                        └─ tokio::try_join!
    .await?                             3. infra.validate_sat_file(                   cfs_configuration_v2_get_all
                                           BackendValidateSatFileParams { … })       ims_image_get_all
                                        4. map csm-rs Error → 400                    ims_recipe_get
                                           on success → 204 No Content            2. validate_sat_file_sections
                                                                                     ├─ validate_configurations_section
                                                                                     ├─ validate_images_section
                                                                                     └─ validate_session_templates_section
                                                                                  3. Ok(()) | Err(…)
```

## Error mapping

| Failure | HTTP | Response body |
|---|---|---|
| Missing/invalid bearer token | 401 | `{ "error": "..." }` (existing auth middleware) |
| SAT references HSM group outside `hsm_group_available_vec` | 403 | `{ "error": "..." }` |
| Malformed JSON body / missing `sat_file` field | 400 | axum-default `Json` rejection |
| Upstream CSM/k8s/Vault unreachable | 502 | `{ "error": "..." }` (via existing `to_handler_error`) |
| SAT validation failure (csm-rs returns `Err`) | **400** | `{ "error": "<csm-rs Error to_string()>" }` |
| Success | **204** | empty body |

Only the last two rows are new behaviour; the rest reuses existing
handler infrastructure. csm-rs's `Error` enum is `.to_string()`'d
into the existing `ErrorResponse` struct — no new error type is
introduced.

### CLI behaviour on 400

`exec.rs` step 6a `bail!`s with `"Server-side SAT validation failed: <message>"`.
Pre-hook never runs. The existing client-side cross-reference checks
in `plan::build_plan` remain in place — they catch issues without a
round trip; server-side validation catches what the client cannot see
(live CSM state, k8s product catalog).

## Testing

**csm-rs**

- New `validate_sat_file` is thin glue (gather + validate). Existing
  unit coverage in `src/commands/i_apply_sat_file/tests.rs` already
  exercises `validate_sat_file_sections` directly; one new
  happy-path integration-style test that mocks CSM + the k8s
  ConfigMap is sufficient to lock the wrapper's wiring.

**manta-backend-dispatcher**

- No tests. The trait default returns `Err("not implemented")`; the
  CSM impl is tested in manta-server.

**manta-server** — one handler test per outcome:

1. csm-rs returns `Ok(())` → handler returns 204.
2. csm-rs returns `Err(...)` → 400 with `{ "error": "..." }`.
3. SAT references a group not in `hsm_group_available_vec` → 403
   before the trait method is called.
4. Missing required header (`Vault`, `K8s-Url`) → existing rejection
   400.

Reuses the wiremock + axum test harness the other sat handlers use,
plus the OpenAPI wire-format-lock pattern documented in
`handlers/sat_file.rs`.

**manta-cli** — one integration-style test:

1. Server returns 204 → exec proceeds to dispatch.
2. Server returns 400 → exec `bail!`s before pre-hook fires.

## Out of scope / future work

- Accumulating-issues validator (`{ valid, issues: [...] }` response).
  Requires reworking the three `validate_*_section` helpers to
  collect rather than short-circuit.
- ochami-backend implementation of `SatTrait::validate_sat_file`.
- Per-section endpoints (`/sat-file/configurations/validate`, etc.).
- A `--skip-validate` flag on `manta apply sat-file`. Adding one
  requires a clear use case; deferred until asked.
