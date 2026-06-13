//! PCS power caps — read and update power caps on capable hardware.
//! Wraps `/power-control/v1/power-cap`.

/// Request / response types for the PCS power-cap endpoints.
pub mod types;

// Canonical names: callers should prefer these over the deeper
// `types::*` paths so the internal layout can evolve without rippling
// through every command.
//
// Post-Task-4 (Option A swap), these are aliases for the
// progenitor-generated schemas — see `types.rs` for the per-name
// behaviour delta vs. the previous hand-written shapes.
pub use types::{
  CapabilitiesLimits, OpTaskStartResponse, PowerCapPatch,
  PowerCapPatchComponent, PowerCapPatchComponentControl, PowerCapSnapshotReq,
  PowerCapTaskInfo, PowerCapTaskInfoType, PowerCapTaskList, PowerCapsRetdata,
  PowerCapsRetdataType, RspPowerCapComponents, RspPowerCapComponentsControl,
  RspPowerCapComponentsControlName, TaskCounts, TaskId,
};
