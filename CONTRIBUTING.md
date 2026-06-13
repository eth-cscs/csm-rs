# Contributing to csm-rs

Thanks for your interest in improving csm-rs. This document covers what
you need to know to get set up, run the tests, and submit a change.

It's still evolving — if something here is unclear or missing, an issue
or PR to update this file is welcome.

## Toolchain

Install Rust via [rustup](https://www.rust-lang.org/tools/install). The
crate's MSRV tracks the edition declared in `Cargo.toml` (`edition =
"2024"`), so a recent stable toolchain is required.

## IDE

Use any editor with `rust-analyzer` as the LSP. For VS Code the
extension is
[rust-lang.rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

`rustfmt.toml` pins the project's formatting (80-column, 2-space
indent). Run `cargo fmt` before pushing.

## Project layout

Source mirrors the CSM service it wraps:

```
src/
├── lib.rs                 # crate root + ShastaClient re-export
├── client.rs              # ShastaClient struct + constructor
├── error.rs               # crate-wide Error enum
├── common/                # cross-cutting helpers (auth, k8s, vault, http)
├── hsm/  cfs/  bss/  bos/  pcs/  (also capmc/  ims/)
│   ├── csm_api_docs.yaml  # upstream OpenAPI spec (or Swagger 2.0)
│   ├── csm_api_docs.openapi3.json   # post-conversion JSON (Swagger sources only)
│   ├── generated.rs       # `include!` of $OUT_DIR/<mod>_generated.rs
│   ├── wrapper/           # ShastaClient::<mod>_* methods live here
│   │   ├── mod.rs         # gen_client / map_err / run helpers
│   │   └── <resource>.rs  # per-resource impl blocks (e.g. group.rs)
│   ├── <resource>/        # per-resource types.rs + dispatcher_conv.rs
│   └── ...
├── commands/              # high-level workflows (apply SAT file, …)
├── node/                  # node-level helpers that span services
└── backend_connector/     # manta-backend-dispatcher trait impls
build.rs                   # runs progenitor over every csm_api_docs.* spec
```

All public HTTP calls live as methods on
[`ShastaClient`](src/client.rs) under `src/<mod>/wrapper/`. Method names
follow `<module>_<resource>_<verb>`, optionally with a version suffix
(`cfs_session_v2_get`, `cfs_session_v3_get`).

### Codegen pipeline

`build.rs` runs [`progenitor`](https://crates.io/crates/progenitor) over
each module's `csm_api_docs.yaml` (or, where the upstream ships
Swagger 2.0, the post-conversion `csm_api_docs.openapi3.json`) and writes
a typed `Client` plus generated wire-format types into
`$OUT_DIR/<mod>_generated.rs`. `src/<mod>/generated.rs` is a one-line
`include!` shim around that file. The hand-written wrapper layer at
`src/<mod>/wrapper/` then forwards each `pcs_*` / `hsm_*` / etc. method
to the generated client where the contracts line up, and falls back to
raw `reqwest` (also exposed through `crate::common::http`) where the
spec disagrees with what real CSM emits. The per-method routing decision
is documented at the top of each `wrapper/<resource>.rs` file.

If the YAML doesn't satisfy progenitor's strictness (missing
`operationId`s, ambiguous responses, `application/problem+json`
content-types it can't model), patches are applied to the committed
copy of the spec with an inline `# PATCH (csm-rs):` comment block citing
the migration's output-reference doc. Do not silently revert these
patches when re-syncing from upstream.

## Adding a new endpoint

If the operation is already declared in `src/<service>/csm_api_docs.yaml`:

1. Find the generated method name by grepping the relevant module's
   output-reference doc under `docs/superpowers/plans/*-progenitor-*-output-reference.md`
   (or by inspecting the generated file at
   `target/debug/build/csm-rs-*/out/<mod>_generated.rs`).
2. Add an `impl ShastaClient { … }` block in
   `src/<service>/wrapper/<resource>.rs` exposing the new
   `<service>_<resource>_<verb>` method. Use the `crate::<service>::wrapper::run`
   adapter for typed methods; the helper handles auth + error mapping.
3. Add a per-method routing rationale in the file's module docstring
   (one line per method: "routed via `do_<X>`" or "stays raw because
   `<concrete contract mismatch>`").
4. Add a wiremock test in `tests/shasta_client_<service>.rs` (see
   below).
5. Document the method with rustdoc — see "Documentation style".

If the operation is **not** in the YAML (csm-rs-specific orchestration,
or upstream hasn't published it):

1. Add the request/response types under
   `src/<service>/<resource>/types.rs`.
2. Add the wrapper method in `src/<service>/wrapper/<resource>.rs` using
   raw `reqwest` via `self.http()`. The shared
   `crate::common::http::handle_json_*` helpers cover the standard
   401-vs-other contract.
3. Document why progenitor routing is impossible in the module docstring
   (typically: "no spec coverage" or "endpoint is csm-rs-internal").
4. Same wiremock test + rustdoc steps.

The pre-1.0 layout used `src/<service>/<resource>/http_client/<vN>/mod.rs`
to host `ShastaClient` methods. That tree has been moved under `wrapper/`
for every migrated service. Resources that still carry an `http_client/`
directory are either pre-migration code or csm-rs-only orchestration.

## Tests

The crate has three tiers of tests:

- **Tier 1 (unit)** — pure-Rust tests next to the code under
  `#[cfg(test)] mod tests`. Run by default.
- **Tier 2 (filter/orchestration)** — also under `#[cfg(test)]`, cover
  helpers that compose multiple calls.
- **Tier 3 (wiremock integration)** — `tests/shasta_client_*.rs`. Each
  file spins up a [`wiremock`](https://docs.rs/wiremock) `MockServer`,
  asserts the `ShastaClient` method calls the expected route with the
  expected bearer token, and checks the deserialised body. No live CSM
  needed.

Run the full suite:

```sh
cargo test
```

Run a single file or test:

```sh
cargo test --test shasta_client_hsm
cargo test hsm_group_get_all_hits_smd_v2_groups
```

`tests/common/mod.rs` provides `make_client(&server.uri())` and a
`TEST_TOKEN` to keep wiremock tests short.

## Documentation

Generate and open the rendered docs locally:

```sh
cargo doc --no-deps --open
```

`cargo doc --no-deps` must finish with **zero warnings** —
`#![deny(rustdoc::broken_intra_doc_links)]` in `lib.rs` enforces this.

### Documentation style

When adding rustdoc to a `ShastaClient` method, follow the pattern
already used in `src/hsm/wrapper/group.rs`:

- First sentence summarises what the method does.
- Add a short line naming the HTTP method and path
  (`GET /cfs/v2/components/{component_id}`).
- Use backticks around code identifiers and angle-bracketed
  placeholders (`` `Box<dyn Error>` ``, not `Box<dyn Error>`) — bare
  angle brackets are parsed as HTML tags and trip rustdoc warnings.
- Wrap URLs in `<…>` to make them autolinks
  (`<https://example.com/path>`).
- Use `# Errors` for non-trivial error semantics, `# Arguments` only
  when arguments need more than a name to be obvious.

## Commit messages

The project uses [Conventional
Commits](https://www.conventionalcommits.org/). Recent history:

```
feat!: migrate cfs/session/v2 and v3 to ShastaClient
fix: accept base64url-encoded JWT claims per RFC 7519
docs: add crate-level intro and ShastaClient usage guide
chore: clean up minor clippy warnings in test code
```

Prefixes in use: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`.
Append `!` for breaking changes (the 0.107 migration was a long
sequence of these).

## Pull requests

- Branch off `main`.
- One logical change per PR; if you're touching many files for the
  same reason that's fine.
- Run before pushing:

  ```sh
  cargo fmt
  cargo clippy --all-targets -- -D warnings
  cargo test
  cargo doc --no-deps
  ```

- No `unsafe` — the crate is `unsafe`-free by design.

## Finding CSM API specs

The OpenAPI specification for each CSM service is published in its
upstream repository under an `api/` folder, e.g.
[`hms-smd/api/swagger_v2.yaml`](https://github.com/Cray-HPE/hms-smd/blob/master/api/swagger_v2.yaml).
Paste the YAML into <https://editor.swagger.io/> to browse it.
