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
├── common/                # cross-cutting helpers (auth, k8s, vault, …)
├── bos/  bss/  capmc/     # one folder per CSM service
├── cfs/  hsm/  ims/  pcs/
│   └── <resource>/
│       ├── http_client/   # ShastaClient methods (often split v2/v3)
│       ├── types/         # request/response shapes
│       └── utils.rs       # helpers built on top
├── commands/              # high-level workflows (apply SAT file, …)
├── node/                  # node-level helpers that span services
└── backend_connector/     # manta-backend-dispatcher trait impls
```

All public HTTP calls live as methods on
[`ShastaClient`](src/client.rs). Method names follow
`<module>_<resource>_<verb>`, optionally with a version suffix
(`cfs_session_v2_get`, `cfs_session_v3_get`).

## Adding a new endpoint

1. Add the request/response types under
   `src/<service>/<resource>/http_client/<vN>/types/`.
2. Add an `impl ShastaClient { … }` block in the corresponding
   `http_client/<vN>/mod.rs` exposing the call as a method. Use
   `self.http()`, `self.base_url()`, `self.token()` rather than
   threading raw values.
3. Add a wiremock test in `tests/shasta_client_<service>.rs` (see
   below).
4. Document the method with rustdoc — see "Documentation style".

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
already used in `src/cfs/component/http_client/v2/mod.rs`:

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
