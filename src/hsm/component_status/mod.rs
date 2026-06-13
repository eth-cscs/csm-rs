//! HSM component status — runtime state snapshots for components.
//!
//! Wraps `GET /smd/hsm/v2/State/Components`; the methods chunk large
//! xname lists to work around per-request id limits. Implementation
//! lives in `src/hsm/wrapper/component_status.rs`.
