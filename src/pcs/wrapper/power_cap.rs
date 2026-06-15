//! Wrapper for PCS `/power-cap` endpoints. Replaces
//! `src/pcs/power_cap/http_client.rs`.
//!
//! Type strategy: **Option A — full type swap**. Unlike other PCS
//! resources, `power_cap` has no `dispatcher_conv` module and no
//! downstream `manta` / `manta_backend_dispatcher` consumers (verified
//! via repo-wide grep at the time of writing), so the cost of swapping
//! to the generated shapes is contained. The hand-written types it
//! replaces had several spec divergences (see
//! `crate::pcs::power_cap::types` module docstring for the full
//! list) — they would have made an Option B "convert at the boundary"
//! approach awkward because there is no faithful mapping for the bugs
//! (e.g. the hand-written `pcs_power_cap_get` claimed to return a
//! single `PowerCapTaskInfo` when the spec returns a `PowerCapTaskList`
//! — the difference cannot be papered over).
//!
//! All 4 methods route through progenitor via the `run` adapter:
//!
//! | Method                          | Generated client call           |
//! |---------------------------------|---------------------------------|
//! | `pcs_power_cap_get`             | `get_power_cap_tasks`           |
//! | `pcs_power_cap_get_task_id`     | `get_power_cap_task`            |
//! | `pcs_power_cap_post_snapshot`   | `post_power_cap_snapshot`       |
//! | `pcs_power_cap_patch`           | `patch_power_cap`               |
//!
//! Signature changes vs. the hand-written wrappers (callers will see
//! these at compile time):
//!
//! - `pcs_power_cap_get` -> returns `PowerCapTaskList` (was
//!   `PowerCapTaskInfo`). This was a hand-written bug; the spec for
//!   `GET /power-cap` returns a list.
//! - `pcs_power_cap_get_task_id` -> returns `PowerCapsRetdata` (was
//!   `PowerCapTaskInfo`). Same root cause — the per-task detail
//!   endpoint returns a different shape (includes `components`). Also
//!   takes a `&TaskId` newtype (constructed from a `Uuid` or parsed
//!   from a `&str`) — the public string-based call site is preserved
//!   by parsing internally and surfacing the parse error as
//!   `Error::Message`.
//! - `pcs_power_cap_post_snapshot` -> returns `OpTaskStartResponse`
//!   (was `PowerCapTaskInfo`). The POST endpoint only returns a task
//!   id, not a full task info.
//! - `pcs_power_cap_patch` -> takes `PowerCapPatch` (was
//!   `Vec<PowerCapComponent>`), returns `OpTaskStartResponse` (was
//!   `PowerCapTaskInfo`), and now hits the correct
//!   `PATCH /power-cap` instead of the hand-written
//!   `PUT /power-cap/snapshot` (another bug).

use crate::{ShastaClient, error::Error};

use crate::pcs::power_cap::types::{
  OpTaskStartResponse, PowerCapPatch, PowerCapSnapshotReq, PowerCapTaskList,
  PowerCapsRetdata, TaskId,
};

use super::run;

impl ShastaClient {
  /// List all power-cap tasks known to PCS.
  ///
  /// `GET /power-control/v1/power-cap`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn pcs_power_cap_get(
    &self,
    token: &str,
  ) -> Result<PowerCapTaskList, Error> {
    run(self, token, |c| async move { c.get_power_cap_tasks().await }).await
  }

  /// Fetch a single power-cap task (with components) by its `task_id`.
  ///
  /// `GET /power-control/v1/power-cap/{task_id}`.
  ///
  /// `task_id` must be a UUID-shaped string; it is parsed locally
  /// before dispatch to surface a clean error if malformed.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn pcs_power_cap_get_task_id(
    &self,
    token: &str,
    task_id: &str,
  ) -> Result<PowerCapsRetdata, Error> {
    let parsed: TaskId = task_id.parse().map_err(|e| {
      Error::Message(format!("invalid power-cap task id {task_id:?}: {e}"))
    })?;
    run(self, token, move |c| async move {
      c.get_power_cap_task(&parsed).await
    })
    .await
  }

  /// Capture a power-cap snapshot for the given component xnames.
  ///
  /// `POST /power-control/v1/power-cap/snapshot` with
  /// `{"xnames": [...]}`. Returns a task id; poll
  /// [`pcs_power_cap_get_task_id`](Self::pcs_power_cap_get_task_id)
  /// for progress.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set. Xnames that fail the spec regex surface as
  /// `Error::Message`.
  pub async fn pcs_power_cap_post_snapshot(
    &self,
    token: &str,
    xname_vec: Vec<&str>,
  ) -> Result<OpTaskStartResponse, Error> {
    log::debug!("Create PCS power snapshot for nodes:\n{xname_vec:?}");

    // Spec-typed xnames: parsing rejects ill-formed identifiers before
    // hitting the network.
    let xnames = xname_vec
      .iter()
      .map(|x| {
        x.parse().map_err(|e| {
          Error::Message(format!("invalid xname {x:?}: {e}"))
        })
      })
      .collect::<Result<Vec<_>, _>>()?;

    let body = PowerCapSnapshotReq { xnames };

    run(self, token, move |c| async move {
      c.post_power_cap_snapshot(&body).await
    })
    .await
  }

  /// Apply a set of power-cap values to the given components.
  ///
  /// `PATCH /power-control/v1/power-cap`. Returns a task id; poll
  /// [`pcs_power_cap_get_task_id`](Self::pcs_power_cap_get_task_id)
  /// for progress.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn pcs_power_cap_patch(
    &self,
    token: &str,
    power_cap: PowerCapPatch,
  ) -> Result<OpTaskStartResponse, Error> {
    log::debug!("Patch PCS power cap:\n{power_cap:#?}");

    run(self, token, move |c| async move {
      c.patch_power_cap(&power_cap).await
    })
    .await
  }
}
