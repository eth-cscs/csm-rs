//! Cross-cutting helpers shared by the CSM API modules.
//!
//! Nothing here is bound to a specific CSM service — these are the
//! plumbing pieces other modules build on.
//!
//! Submodules:
//!
//! - [`authentication`] — Keycloak / OIDC token acquisition for Shasta.
//! - [`jwt_ops`] — JWT decoding helpers (RFC 7519 base64url-aware) used
//!   by callers that need to introspect a Shasta token without verifying
//!   its signature.
//! - [`kubernetes`] — in-cluster API client used to read CSM-side state
//!   that isn't exposed over REST (e.g. the `cray-product-catalog`
//!   ConfigMap).
//! - [`vault`] — fetch K8s service-account secrets from Vault, which is
//!   the supported way to obtain CSM cluster credentials off-cluster.
//! - [`gitea`] — small client for the embedded CSM Gitea instance used
//!   by CFS configuration layers.
//! - [`cluster_ops`] — generic cluster-scoped helpers used by the
//!   commands layer.
//!
//! `http` and `yaml` exist as crate-internal utilities and are not
//! part of the public surface.

pub mod authentication;
pub mod cluster_ops;
pub mod gitea;
pub(crate) mod http;
pub mod jwt_ops;
pub(crate) mod poll;
/// In-cluster Kubernetes client helpers (used to read ConfigMaps such
/// as `cray-product-catalog`). Requires the `k8s-console` Cargo
/// feature.
#[cfg(feature = "k8s-console")]
pub mod kubernetes;
// The only user of `vault::http_client::fetch_shasta_k8s_secrets_from_vault`
// is the Kubernetes secret-fetching path (CFS session log streaming
// and `cfs::session::i_post_sync`), so the whole module rides the
// same `k8s-console` feature gate.
#[cfg(feature = "k8s-console")]
pub mod vault;
pub(crate) mod yaml;
