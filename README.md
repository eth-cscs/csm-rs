# csm-rs

A Rust library for interacting with the HPE Cray Shasta CSM (Cray System
Management) API.

`csm-rs` (formerly *Mesa*) is the foundation used by applications like
[Manta](https://github.com/eth-cscs/manta) to integrate with Shasta-based
systems. It avoids `unsafe` code and aims to provide a safe, ergonomic
async interface to the CSM control plane.

Typical use cases:

- Building applications that integrate Shasta/CSM systems into your
  ecosystem.
- Simplifying or scripting common CSM operations.
- Extending CSM functionality beyond what the official CLIs expose.

## Supported APIs

The crate currently wraps the following CSM components:

- HSM (Hardware State Manager) — `hsm`
- CFS (Configuration Framework Service) — configurations & sessions
- BOS (Boot Orchestration Service) — `bos`
- BSS (Boot Script Service) — `bss`
- CAPMC (Cray Advanced Platform Monitoring and Control) — `capmc`
- IMS (Image Management Service) — `ims`
- PCS (Power Control Service) — `pcs`
- Node operations — `node`
- Kubernetes & Keycloak helpers — `common`

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
csm-rs = "0.107"
tokio = { version = "1", features = ["full"] }
```

## Quick start

All HTTP calls are exposed as methods on [`ShastaClient`]. Construct one
per Shasta installation and reuse it — it bundles the auth token, base
URL, root certificate, and an optional SOCKS5 proxy with a pre-built
`reqwest::Client`:

```rust,no_run
use csm_rs::ShastaClient;

#[tokio::main]
async fn main() -> Result<(), csm_rs::error::Error> {
    let client = ShastaClient::new(
        "https://api.shasta.example.com",
        "your-bearer-token",
        std::fs::read("/etc/shasta/ca.crt").unwrap(),
        None, // or Some("socks5://localhost:9050".to_string())
    )?;

    // Methods are namespaced by API module: `<module>_<resource>_<verb>`.
    let images  = client.ims_image_get_all().await?;
    let groups  = client.hsm_group_get_all().await?;
    let configs = client.cfs_configuration_v2_get_all().await?;

    Ok(())
}
```

Method names are versioned where the underlying API is — e.g.
`cfs_session_v2_get` vs `cfs_session_v3_get`. See each module's rustdoc
for the full list.

## Examples

Runnable programs under [`examples/`](examples/):

- [`list_hsm_groups`](examples/list_hsm_groups.rs) — minimal client
  construction plus one GET.
- [`list_cfs_sessions`](examples/list_cfs_sessions.rs) — paginated CFS
  v3 session listing.
- [`power_cycle_nodes`](examples/power_cycle_nodes.rs) — PCS transition
  with synchronous wait.

Each reads `CSM_BASE_URL`, `CSM_TOKEN`, and `CSM_ROOT_CERT_PATH` from
the environment. Run with `cargo run --example <name>`.

## Migrating from 0.106 and earlier

Releases up to and including 0.106 exposed each HTTP call as a free
function with a 4-parameter auth quartet:

```rust,ignore
// 0.106 and earlier — removed in 0.107.
ims::image::http_client::get_all(token, base_url, root_cert, proxy).await?;
```

In 0.107 these free functions were removed in favor of methods on
[`ShastaClient`]:

```rust,ignore
// 0.107+
client.ims_image_get_all().await?;
```

The auth context now lives on the client, so callers no longer need to
thread token/base-url/cert/proxy through every call site.

## Building & testing

Build:

```sh
cargo build
```

Run the test suite (some tests require access to a live Shasta backend
and are gated accordingly):

```sh
cargo test -- --show-output
```

Generate API documentation locally:

```sh
cargo doc --open
```

## Release

Releases are cut with [`cargo-release`](https://github.com/crate-ci/cargo-release):

```sh
cargo release patch --execute
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under the terms of the [LICENSE](LICENSE) file in the repository
root.
