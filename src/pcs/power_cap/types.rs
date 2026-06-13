//! Re-exports of the progenitor-generated PCS `/power-cap` schemas.
//!
//! Task 4 (Option A — full type swap). The hand-written types this file
//! previously contained diverged from the OpenAPI source of truth on
//! several fronts; the swap takes the generated shapes verbatim and
//! lets the spec be the single source of truth.
//!
//! Behaviour deltas vs. the previous hand-written types:
//!
//! - `PowerCapTaskInfo.task_id`: was `Option<String>`, now
//!   `Option<uuid::Uuid>` (`uuid` is already in the crate dep tree;
//!   serialisation is unchanged — a UUID string).
//! - `PowerCapTaskInfo.r#type`: was `Option<String>`, now
//!   `Option<PowerCapTaskInfoType>` (typed `snapshot`/`patch` enum). On
//!   the wire the serialisation is identical.
//! - `PowerCapTaskInfo.automatic_expiration_time`: was `Option<String>`,
//!   now `Option<chrono::DateTime<Utc>>`. RFC 3339 on the wire.
//! - `PowerCapTaskInfo.components`: **removed**. The spec keeps the
//!   list-element type (`PowerCapTaskInfo`) free of components; the
//!   per-task detail endpoint returns `PowerCapsRetdata`, which adds
//!   `components: Vec<RspPowerCapComponents>`. The hand-written type
//!   conflated the two.
//! - `TaskCounts.{total,new,in_progress,failed,succeeded,un_supported}`:
//!   were `usize` (non-optional), now `Option<i64>` (matches the spec —
//!   the response may omit the field if absent on the server side).
//! - `Limit` -> renamed to `CapabilitiesLimits` and field names
//!   corrected: hand-written used `hostsLimitMax`/`hostsLimitMin` (typo
//!   — extra `s`), the spec is `hostLimitMax`/`hostLimitMin`. `usize` →
//!   `i64`.
//! - `PowerCapLimit` -> renamed to `RspPowerCapComponentsControl`.
//!   `name: Option<String>` -> `Option<RspPowerCapComponentsControlName>`
//!   (typed `Node`/`Accel` enum). The serialised wire shape for the
//!   field also changes container key: hand-written rename was
//!   `power_cap_limits` (snake_case) but the spec uses
//!   `powerCapLimits` (camelCase) — the hand-written code therefore
//!   silently dropped this field when present in real responses.
//! - `PowerCapComponent` -> renamed to `RspPowerCapComponents`.
//!   `power_cap_limits` is now `Vec<RspPowerCapComponentsControl>` (the
//!   spec returns an array; the hand-written code expected a single
//!   object).
//! - `PowerCapsRetdata` (newly exposed) — what
//!   `GET /power-cap/{taskID}` actually returns: a `PowerCapTaskInfo`
//!   shape plus a `components: Vec<RspPowerCapComponents>` array.
//! - `OpTaskStartResponse` (newly exposed) — what the POST snapshot and
//!   PATCH endpoints actually return (a thin `{ "taskID": <uuid> }`).
//!   The hand-written code claimed both return `PowerCapTaskInfo`.
//!
//! Wrapper file `src/pcs/wrapper/power_cap.rs` documents how each of
//! the 4 public methods routes onto these types.

pub use crate::pcs::generated::types::{
  CapabilitiesLimits, OpTaskStartResponse, PowerCapPatch,
  PowerCapPatchComponent, PowerCapPatchComponentControl, PowerCapSnapshotReq,
  PowerCapTaskInfo, PowerCapTaskInfoType, PowerCapTaskList, PowerCapsRetdata,
  PowerCapsRetdataType, RspPowerCapComponents, RspPowerCapComponentsControl,
  RspPowerCapComponentsControlName, TaskCounts, TaskId,
};
