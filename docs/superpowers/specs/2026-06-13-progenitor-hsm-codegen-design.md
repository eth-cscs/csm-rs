# Generating the HSM HTTP client with progenitor

**Status**: Design — awaiting review
**Date**: 2026-06-13
**Scope**: `src/hsm/` module of csm-rs

## Problem

The HSM (Hardware State Manager) HTTP client in `src/hsm/` is hand-written:
~18 endpoint wrappers across 8 resource modules, plus ~30 hand-rolled
wire-format types in `src/hsm/*/types.rs` that mirror the CSM HSM
OpenAPI schemas. Today the spec lives in-tree at
`src/hsm/csm_api_docs.yaml` (Swagger 2.0, ~12k lines, ~70 endpoints, 210
schemas) and the Rust code is maintained by hand against it.

Three concrete problems:

1. **Drift risk**. The spec and the code are kept in sync by review
   discipline, not by tooling. Two latent bugs were found while
   designing this work:
   - `RedfishEndpoint::rediscover_on_update` is `#[serde(rename =
     "RediscoveryOnUpdate")]` — spec says `RediscoverOnUpdate` (no
     "y"). Field is silently dropped on the wire today.
   - `EthernetInterface` (`src/hsm/hw_inventory/ethernet_interfaces/types.rs:61`)
     has snake_case field names with no `#[serde(rename)]`. The spec
     uses PascalCase (`MACAddress`, `IPAddresses`, etc.) and an array
     of `IPAddressMapping` for IPs, not a single `Option<String>`.
     The struct does not deserialize the wire format correctly.
2. **Coverage gap**. csm-rs uses only 18 of ~70 documented endpoints.
   Adding new ones requires hand-writing each, which is why the gap
   has been stable.
3. **Type debt**. ~2500 lines of hand-rolled schema types in
   `src/hsm/*/types.rs`. ~90% of them are 1:1 mirrors of YAML
   definitions; the remaining 10% are genuine csm-rs-specific
   projections (`NodeSummary`, `ArtifactSummary`).

## Decision summary

| Decision | Choice |
|---|---|
| Codegen tool | `progenitor` (Oxide) |
| Spec format conversion | Swagger 2.0 → OpenAPI 3.0 via `swagger2openapi` (committed JSON artifact, NOT in build.rs) |
| Integration mode | `build.rs` + `include!(OUT_DIR/...)` |
| Public API stability | Preserve `client.hsm_*(...)` method shape via thin wrapper layer |
| Type strategy | **Replace** hand-rolled wire types with generated ones; retain `NodeSummary`/`ArtifactSummary`/`ArtifactType` as projection types |
| Inherent methods on wire types | Move to extension traits (`GroupExt`, etc.) |
| Scope | HSM/SMD module only — CFS and other modules untouched |

## Why progenitor, not openapi-generator or utoipa

- **utoipa generates OpenAPI specs from Rust code** — the reverse
  direction. Not applicable.
- **openapi-generator** is Java-based; adopting it adds a JDK
  dependency to every contributor and CI runner. Its Rust template
  emits a less idiomatic shape (`Configuration` struct + free
  functions in `*Api` modules) that's farther from current
  `ShastaClient` method-on-struct style.
- **progenitor** runs inside `cargo build` (pure Rust toolchain) and
  emits `reqwest` + `serde` + `async/await` code that closely matches
  the existing hand-written style.

The one risk is strict spec parsing: progenitor will refuse a spec
with structural issues that openapi-generator might paper over. This
is mitigated by phase 0 (see Execution).

## Architecture

```
src/hsm/csm_api_docs.yaml              (Swagger 2.0 — upstream artifact)
                │
                ▼ (developer step: `make convert-spec`, NOT cargo)
src/hsm/csm_api_docs.openapi3.json     (OpenAPI 3.0 — committed alongside)
                │
                ▼ (build.rs)
$OUT_DIR/hsm_generated.rs              (progenitor output, never committed)
                │
                ▼ (include!)
src/hsm/generated.rs                   (pub(crate) — wrapper-only access)
                │
                ▼
src/hsm/wrapper/{component,group,...}  (thin glue, ~18 methods total)
                │
                ▼
impl ShastaClient { pub async fn hsm_*(...) }   (PUBLIC API — unchanged shape)
                │
                ▼
downstream callers (manta, etc.)        (recompile-clean)
```

### File tree changes

**Added**:
- `csm-rs/build.rs` — reads `src/hsm/csm_api_docs.openapi3.json`, runs
  `progenitor::Generator`, writes `$OUT_DIR/hsm_generated.rs`.
- `src/hsm/generated.rs` — one-liner
  `include!(concat!(env!("OUT_DIR"), "/hsm_generated.rs"));` made
  `pub(crate)`. Generated types are then re-exported with stable names
  from per-resource `types.rs` files.
- `src/hsm/wrapper/mod.rs` — `gen_client(&self, token)` helper and the
  `map_err` function that maps `progenitor_client::Error<T>` →
  `crate::error::Error`.
- `src/hsm/wrapper/{component,component_status,group,memberships,
  hw_component,redfish_endpoint,ethernet_interfaces,service_values}.rs`
  — one file per resource holding `impl ShastaClient { pub async fn
  hsm_*() }` blocks.
- `src/hsm/wrapper/hw_component_types.rs` — projection types
  (`NodeSummary`, `ArtifactSummary`, `ArtifactType`) and their
  `try_from_generated` impls. Public types, re-exported through
  `hsm/hw_inventory/hw_component/mod.rs` to preserve the existing
  public path.
- `src/hsm/csm_api_docs.openapi3.json` — converted spec, committed.
- `Makefile` (or `xtask`) target `convert-spec` invoking
  `npx swagger2openapi` — developer step, not invoked by `cargo build`.
- Extension traits: `src/hsm/group/ext.rs` (and any other resource
  that had inherent methods on wire types) defining `GroupExt` etc.

**Removed**:
- `src/hsm/component/http_client.rs`
- `src/hsm/component_status/http_client/mod.rs`
- `src/hsm/group/http_client.rs`
- `src/hsm/memberships/http_client.rs`
- `src/hsm/hw_inventory/hw_component/http_client.rs`
- `src/hsm/hw_inventory/redfish_endpoint/http_client.rs`
- `src/hsm/hw_inventory/ethernet_interfaces/http_client.rs`
- `src/hsm/service/values/role/http_client.rs`

**Modified**:
- `src/hsm/*/types.rs` files reduce to **pure** `pub use` re-export
  blocks pointing at generated types. No hand-rolled structs survive
  in `types.rs`. Projection types and module-private helpers move
  into the wrapper layer (see Type strategy).

## Components

### Spec conversion step

Swagger 2.0 is not supported by progenitor. `swagger2openapi` (npm,
MIT) converts to OpenAPI 3.0. The conversion runs as a developer step,
NOT inside `build.rs`, so a `cargo build` does not require Node.

```bash
# Convention: re-run when csm_api_docs.yaml changes.
make convert-spec
# expands to:
npx swagger2openapi src/hsm/csm_api_docs.yaml \
    -o src/hsm/csm_api_docs.openapi3.json
```

The JSON is committed and is the input progenitor reads. The YAML
stays as the upstream-tracked artifact for reference and provenance.

### build.rs

Reads `src/hsm/csm_api_docs.openapi3.json`, runs
`progenitor::Generator::new()` with default settings (tuned in phase
0), writes the result to `$OUT_DIR/hsm_generated.rs`. Adds a
`cargo:rerun-if-changed=src/hsm/csm_api_docs.openapi3.json` directive
so changes to the spec trigger regeneration.

### generated.rs

```rust
// src/hsm/generated.rs
include!(concat!(env!("OUT_DIR"), "/hsm_generated.rs"));
```

The entire module is `pub(crate)` from `src/hsm/mod.rs`. No code
outside `src/hsm/wrapper/` and `src/hsm/*/types.rs` touches it.

### Wrapper layer

`src/hsm/wrapper/mod.rs` provides:

```rust
pub(crate) fn gen_client(client: &ShastaClient, token: &str)
    -> crate::hsm::generated::Client { ... }

fn map_err<E: std::fmt::Debug>(e: progenitor_client::Error<E>)
    -> crate::error::Error { ... }
```

**Auth strategy** (per-call client construction): csm-rs's API is
per-call token (`async fn hsm_groups_get_all(&self, token: &str)`),
but progenitor's `Client` bakes auth into the inner reqwest client at
construction. Resolution: build a fresh `generated::Client` per call.
`reqwest::Client` clones are `Arc`-cheap, so per-call construction is
essentially free.

**Server URL override**: The spec's `basePath: /apis/smd/hsm/v2` is
double-counted against current callers. Confirmed: csm-rs's
`base_url` is e.g. `https://api.cmn.alps.cscs.ch/apis` (already
includes `/apis`). `gen_client` constructs progenitor's `Client` with
`format!("{}/smd/hsm/v2", client.base_url())`, ignoring the spec's
declared basePath.

**Per-resource wrapper files** hold the public surface:

```rust
// src/hsm/wrapper/group.rs
impl ShastaClient {
    pub async fn hsm_groups_get_all(&self, token: &str)
        -> Result<Vec<Group>, Error>
    {
        gen_client(self, token).do_groups_get_v2(/* query */)
            .await.map_err(map_err)
            .map(|rv| rv.into_inner())
    }
    // ... 4 more methods for groups
}
```

Most wrappers are 3–5 lines. The exceptions:
- `hsm_component_status_get` retains its chunking loop (the inner GET
  delegates to the generated client; the loop is unchanged).
- `hsm_hw_inventory_get` performs a projection via
  `NodeSummary::try_from_generated(...)`.

### Type strategy

Each `src/hsm/*/types.rs` becomes a re-export block:

```rust
// src/hsm/group/types.rs (after)
pub use crate::hsm::generated::types::Group_1_0_0 as Group;
pub use crate::hsm::generated::types::Members_1_0_0 as Members;

// `Member` (the singular helper) has no schema counterpart — stays hand-rolled.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Member {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
```

The exact mangled type names progenitor emits (e.g. `Group_1_0_0` vs
`Group100`) get locked down in phase 0 by inspecting one generated
file.

**Projection types live in the wrapper layer**, not in
`hw_component/types.rs`. They have no spec equivalent and are an
implementation detail of how the wrapper translates generated wire
types into the csm-rs public API:

- `NodeSummary` (public — returned from `hsm_hw_inventory_get`)
- `ArtifactSummary` (public — embedded in `NodeSummary`)
- `ArtifactType` enum (public — embedded in `ArtifactSummary`)

These move to `src/hsm/wrapper/hw_component_types.rs` (or inline at
the top of `src/hsm/wrapper/hw_component.rs` if small). The wrapper's
`hsm_hw_inventory_get` method imports them and returns
`NodeSummary` as the public return type. `src/hsm/hw_inventory/hw_component/mod.rs`
re-exports them so the existing public path
`csm_rs::hsm::hw_inventory::hw_component::NodeSummary` stays valid.

Their `try_from_csm_value(&serde_json::Value)` constructors become
`try_from_generated(&generated::types::HwInventory100HwInventoryByLocation)`
— typed input rather than walking JSON, same projection logic. The
`try_from_generated` impls also live in the wrapper module since they
are part of the wire-to-public translation.

After this change, `hw_component/types.rs` is a pure re-export module
like every other `*/types.rs`: only `pub use generated::...` lines
plus `pub use crate::hsm::wrapper::hw_component_types::*` to surface
the projection types.

### Extension traits for inherent methods

Hand-rolled types currently carry inherent methods used by callers:
- `Group::new(label, members)`, `Group::get_members()`,
  `Group::get_members_opt()`, `Group::add_xnames(&[String])`.
- Constructors / mutators on other types as discovered during
  migration.

Since `impl GeneratedType { ... }` would compete with the
auto-generated impl block, these move to extension traits:

```rust
// src/hsm/group/ext.rs
pub trait GroupExt {
    fn new_with_members(label: &str, member_xnames: Option<Vec<&str>>) -> Self;
    fn get_members(&self) -> Vec<String>;
    fn get_members_opt(&self) -> Option<Vec<String>>;
    fn add_xnames(&mut self, xnames: &[String]) -> Vec<String>;
}

impl GroupExt for Group { ... }
```

Callers add `use csm_rs::hsm::group::GroupExt;` to keep using the
helpers. `src/hsm/group/mod.rs` re-exports the trait at module level
so the import path is stable.

## Data flow (single call)

For `client.hsm_groups_get_all(token).await`:

1. Caller invokes the public method.
2. `wrapper::group::hsm_groups_get_all` calls `gen_client(self, token)`.
3. `gen_client` clones `self.http()` (cheap Arc clone), builds a
   `reqwest::Client` with the bearer token injected as a default
   header (or via middleware — phase 0 confirms which works with
   progenitor's API), constructs
   `generated::Client::new_with_client(format!("{}/smd/hsm/v2",
   self.base_url()), http)`.
4. Wrapper calls `gen_client.do_groups_get_v2(...)`.
5. progenitor builds and sends `GET https://api.cmn.alps.cscs.ch/apis/smd/hsm/v2/groups`,
   deserializes into `ResponseValue<Vec<Group>>`.
6. Wrapper unwraps with `.into_inner()` (or maps via `From` where the
   types differ slightly) and maps errors via `map_err`.

## Error handling

`progenitor_client::Error<T>` variants map into existing
`crate::error::Error` variants:

| progenitor variant | csm-rs variant |
|---|---|
| `CommunicationError(reqwest::Error)` | `Error::NetError` |
| `ErrorResponse(ResponseValue<E>)` | `Error::CsmError(formatted)` |
| `InvalidRequest(String)` | `Error::CsmError(formatted)` |
| `InvalidResponsePayload(_, serde_json::Error)` | `Error::JsonError` |
| `InvalidUpgrade`, `UnexpectedResponse`, etc. | `Error::CsmError(formatted)` |

(Exact variant list verified against `progenitor_client` source in
phase 0.)

## Testing

**Inherited tests**: existing `#[cfg(test)]` blocks in
`src/hsm/*/types.rs` are ported to use the new type aliases. Tests
that round-trip JSON should continue to pass against the generated
types. **Any failing test reveals a real divergence** between the
spec and current wire behavior — those are the latent bugs the design
should flag and decide on case-by-case.

**New tests**:
- One integration test per resource that hits a real CSM endpoint
  (or a mock) and asserts the URL constructed by the generated
  client matches what current code sends today. The starter target
  is `hsm_service_values_role_get` — smallest, idempotent, read-only.
- A unit test for `gen_client()` that asserts the constructed
  baseurl + bearer-header configuration.

**Doctest / regression**: keep the public method signatures
documented with examples; if a doctest in the wrapper compiles, the
public surface didn't break.

## Migration order

One resource per PR-shaped commit. Each commit is independently
revertable.

1. **`service/values/role`** — 1 method. Dogfoods the whole pipeline
   end-to-end. If this fails, fall back to openapi-generator before
   migrating anything else.
2. **`memberships`** — 2 methods, no projections, no inherent
   methods to move.
3. **`group`** — 5 methods, has inherent methods to move to
   `GroupExt`. Highest downstream impact (`manta` uses it heavily) —
   verify `cargo check` in `../manta` after this commit.
4. **`component`** — 5+ methods, 15-field struct. Multiple
   `ComponentArray*` and `ComponentCreate`/`ComponentPut` types to
   re-export.
5. **`component_status`** — keep chunking loop in the wrapper;
   inner GET delegates to generated client.
6. **`hw_component`** — biggest module (1415-line `types.rs`). The
   `NodeSummary::try_from_generated` rewrite happens here.
7. **`redfish_endpoint`** — silent-bug fix for `RediscoverOnUpdate`
   field name. Flag in the commit message.
8. **`ethernet_interfaces`** — resolves the
   `EthernetInterface` vs `ComponentEthernetInterface` ambiguity.
   This is the messiest migration; bump it to last so earlier
   commits are not blocked by it.

Per-commit checklist:

- [ ] Replace `types.rs` content with pure `pub use` aliases to
      generated types. Move any hand-rolled projection types or
      helper structs into the corresponding `wrapper/` module.
- [ ] Move any inherent methods to an `*Ext` extension trait.
- [ ] Delete `http_client*.rs`. Add `wrapper/<resource>.rs`.
- [ ] Update `mod.rs` re-exports (wrapper, extension trait).
- [ ] Port `#[cfg(test)]` tests.
- [ ] `cargo build && cargo test`.
- [ ] `(cd ../manta && cargo check)` — verify downstream compiles.

## Execution: phase 0 (gate before anything else)

Before any of the above is committed, run a half-day timeboxed
feasibility check:

1. `npx swagger2openapi src/hsm/csm_api_docs.yaml -o /tmp/openapi3.json`
   — does conversion succeed? If yes, commit the JSON.
2. Scratch crate with build.rs invoking `progenitor::Generator` on
   `/tmp/openapi3.json` — does `cargo build` succeed? Capture any
   parse errors.
3. Inspect the generated `.rs` file (in `target/`): catalogue the
   actual mangled type names, method names from `operationId`s, and
   the error enum variant list. **These are the contract** the rest
   of the design assumes.
4. Write one integration test against a live CSM cluster (or a
   recorded mock) that calls the generated client's
   `do_service_values_role_get_v2` and confirms the URL it sends is
   `https://api.cmn.alps.cscs.ch/apis/smd/hsm/v2/service/values/role`.

**Exit criteria**:
- Phase 0 succeeds → proceed to migration step 1.
- Phase 0 fails after a day of YAML patching → escalate; fall back to
  openapi-generator (Java toolchain) or scope-reduce to a subset of
  endpoints.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| progenitor rejects the converted spec | Phase 0 catches; patch JSON or fall back to openapi-generator |
| Generated type field names don't match wire format (e.g., progenitor emits `id` from `ID` JSON key in a way that differs from current `#[serde(rename = "ID")]` hand-rolled fields) | Ported tests catch divergence; mitigation is progenitor casing config or post-conversion JSON patches |
| Generated type ergonomics differ (e.g., builder pattern required for construction) | `*Ext::new()` helpers absorb the asymmetry |
| `operationId`s missing in spec → ugly auto-derived method names like `state_components_xname_get` | Wrapper layer hides them; only the wrapper sees the generated names |
| Singular/plural `RedfishEndpoint/Query` path bug (line 28 of `redfish_endpoint/http_client.rs`) | If using the YAML's path as-is the wrapper calls a wrong URL → confirm against a real CSM cluster in phase 0; if needed, patch the JSON or override in the wrapper |
| `base_url()` already includes `/apis`, double-counts with spec basePath | Confirmed at design time; `gen_client()` formula handles it |
| `RediscoverOnUpdate` field rename, `EthernetInterface` casing — latent bugs in current code | Silently fixed by generated code; flagged in respective commit messages so reviewers see the behavior delta |
| Generated code adds compile time | Mitigation: `build.rs` regenerates only when JSON changes (`cargo:rerun-if-changed`); cold builds add ~1–3s |
| Generated `Client` not compatible with custom bearer-auth middleware | If progenitor's `Client` builder doesn't accept a configured reqwest::Client, fall back to building a fresh `reqwest::Client` per call with `default_headers` containing the bearer token — same cost |

## Out of scope

- CFS, BOS, IMS, PCS, and other csm-rs modules. Same approach could
  apply later if this lands cleanly.
- Removing csm-rs's wrapper layer entirely and exposing the generated
  client publicly. (We preserve `client.hsm_*(...)` for stability.)
- Replacing csm-rs's `Error` enum or `ShastaClient` constructor.
- Spec-fixing upstream. We patch the converted JSON in-tree, not the
  upstream Cray YAML.

## Open questions to resolve before / during implementation

1. Does swagger2openapi succeed on `csm_api_docs.yaml` without manual
   patches? (Phase 0.)
2. What exact type-name and method-name mangling does progenitor
   apply? (Phase 0 — affects every `pub use` alias.)
3. Does progenitor's `Client::new_with_client` accept a custom
   `reqwest::Client`, or do we need middleware? (Phase 0.)
4. Are there inherent methods on wire types beyond the ones on `Group`
   that need to move to extension traits? Discover during per-resource
   migration.
