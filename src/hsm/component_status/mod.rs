//! HSM component status — runtime state snapshots for components.
//!
//! Wraps `GET /smd/hsm/v2/State/Components`; the methods chunk large
//! xname lists to work around per-request id limits.

pub mod http_client;
