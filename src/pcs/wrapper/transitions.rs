//! Wrapper for PCS `/transitions` endpoints. Replaces
//! `src/pcs/transitions/http_client/`.
//!
//! Routing: **all 5 methods stay on raw `reqwest`.** The generated
//! surface diverges from the hand-written public types in ways that
//! cannot be papered over without an extra conversion layer at the
//! wrapper boundary — and `transitions` is the most heavily
//! dispatcher-coupled PCS resource (182-line `dispatcher_conv.rs`,
//! mirroring `Location`, `Operation`, `Task`, `TaskCounts`,
//! `Transition`, `TransitionResponse`, `TransitionResponseList`, and
//! `TransitionStartOutput` field-for-field to
//! `manta_backend_dispatcher::types::pcs::transitions::types::*`).
//! Adopting the generated shapes would ripple through the dispatcher
//! trait impls, every `From` impl, and the downstream `manta`
//! consumers.
//!
//! Type strategy: **Option B — keep hand-written types**. The
//! `types.rs` module is untouched; the `dispatcher_conv.rs`
//! conversions continue to work without modification.
//!
//! Per-method routing rationale (concrete divergences vs. generated
//! `crate::pcs::generated::types::*`):
//!
//! - `pcs_transitions_get` — generated `get_transitions` returns
//!   `TransitionsGetAll { transitions: Vec<TransitionsGet> }` where
//!   each `TransitionsGet` has `transition_id: Option<uuid::Uuid>`,
//!   `automatic_expiration_time:
//!   Option<chrono::DateTime<chrono::offset::Utc>>`, `operation:
//!   Option<PowerOperation>` (PCS-local enum, distinct from the
//!   transitions `Operation` enum csm-rs shares across PCS),
//!   `task_counts: Option<TaskCounts>`, and `transition_status:
//!   Option<TransitionStatus>`. csm-rs's public
//!   `Vec<TransitionResponse>` has all of those fields as plain
//!   (non-`Option`) `String`s / shared types, so the wrapper would
//!   need a fallible boundary conversion that fills in defaults for
//!   missing fields — same conversion shape as the hand-written impl,
//!   no simplification gained.
//! - `pcs_transitions_get_by_id` — generated `get_transition` takes a
//!   `&uuid::Uuid` path param (Task 0 hint: spec declares
//!   `transitionID` as a raw `uuid::Uuid`, not a newtype) and returns
//!   `TransitionsGetId` with the same `Option<…>` divergences as
//!   `TransitionsGet` above. Same boundary-conversion cost.
//! - `pcs_transitions_post` — generated `post_transition` takes
//!   `&TransitionCreate` where `operation:
//!   Option<TransitionCreateOperation>` (a freshly-emitted enum
//!   per-endpoint, NOT shared with the `PowerOperation` enum used in
//!   responses), `location: Vec<ReservedLocation>` (a separate
//!   schema), and `task_deadline_minutes: Option<i64>` (vs.
//!   hand-written `Option<usize>`). Returns `TransitionStartOutput`
//!   with `transition_id: Option<uuid::Uuid>` and `operation:
//!   Option<PowerOperation>` — both `Option`, both with newtype/enum
//!   identities the public API doesn't surface. Routing through
//!   progenitor here would force the public method to either accept
//!   the generated request shape (rippling through dispatcher_conv
//!   and the trait impls) or perform two field-by-field conversions
//!   at the boundary — duplicating the work of the hand-written
//!   `Operation::from_str` + `Location` construction the existing
//!   impl already does.
//! - `pcs_transitions_post_block` — convenience wrapper that calls
//!   `pcs_transitions_post` then `pcs_transitions_wait_to_complete`.
//!   Not a single-endpoint binding.
//! - `pcs_transitions_wait_to_complete` — polling wrapper around
//!   `pcs_transitions_get_by_id` with exponential backoff (3 s → 30
//!   s, capped at 40 attempts ≈ 18 min wall-clock). Not a
//!   single-endpoint binding.
//!
//! The `gen_client` / `map_err` / `run` helpers in
//! `crate::pcs::wrapper` are retained so a future spec revision (or a
//! decision to swap the public types for the generated newtype-bearing
//! shapes) can migrate any of the single-endpoint methods incrementally
//! without a second scaffolding pass.

use std::str::FromStr;
use std::time;

use crate::{
  ShastaClient,
  common::http,
  error::Error,
  pcs::transitions::types::{
    Location, Operation, Transition, TransitionResponse, TransitionResponseList,
    TransitionStartOutput,
  },
};

impl ShastaClient {
  /// List every power transition currently known to PCS.
  ///
  /// `GET /power-control/v1/transitions`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn pcs_transitions_get(
    &self,
    token: &str,
  ) -> Result<Vec<TransitionResponse>, Error> {
    let url = format!("{}/power-control/v1/transitions", self.base_url());
    let list: TransitionResponseList =
      http::get_json(self.http(), &url, token).await?;
    Ok(list.transitions)
  }

  /// Fetch a single power transition by its `id`.
  ///
  /// `GET /power-control/v1/transitions/{id}`.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn pcs_transitions_get_by_id(
    &self,
    token: &str,
    id: &str,
  ) -> Result<TransitionResponse, Error> {
    let url =
      format!("{}/power-control/v1/transitions/{}", self.base_url(), id);
    let transition: TransitionResponse =
      http::get_json(self.http(), &url, token).await?;
    log::debug!("PCS transition details\n{transition:#?}");
    Ok(transition)
  }

  /// Start a power transition (`on`, `off`, `reset`, …) against a set
  /// of xnames and return immediately with the transition id.
  ///
  /// `POST /power-control/v1/transitions`. Use
  /// [`Self::pcs_transitions_post_block`] if you want to wait for the
  /// transition to complete.
  ///
  /// # Errors
  ///
  /// Returns an error if `operation` is not a valid PCS [`Operation`].
  pub async fn pcs_transitions_post(
    &self,
    token: &str,
    operation: &str,
    xname_vec: &[String],
  ) -> Result<TransitionStartOutput, Error> {
    log::debug!("Create PCS transition '{operation}' on {xname_vec:?}");

    let location_vec: Vec<Location> = xname_vec
      .iter()
      .map(|xname| Location {
        xname: xname.clone(),
        deputy_key: None,
      })
      .collect();

    let request_payload = Transition {
      operation: Operation::from_str(operation)?,
      task_deadline_minutes: None,
      location: location_vec,
    };

    let url = format!("{}/power-control/v1/transitions", self.base_url());
    http::post_json(self.http(), &url, token, &request_payload).await
  }

  /// Like [`Self::pcs_transitions_post`] but waits for the transition to
  /// finish before returning.
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn pcs_transitions_post_block(
    &self,
    token: &str,
    operation: &str,
    xname_vec: &[String],
  ) -> Result<TransitionResponse, Error> {
    let started = self
      .pcs_transitions_post(token, operation, xname_vec)
      .await?;

    log::debug!("PCS transition ID: {}", started.transition_id);

    self
      .pcs_transitions_wait_to_complete(token, &started.transition_id)
      .await
  }

  /// Polls a transition until it reaches `completed` status, with
  /// exponential backoff (3 s → 30 s, capped at 40 attempts ≈ 18 min
  /// wall-clock).
  ///
  /// # Errors
  ///
  /// Returns an [`Error`] variant on CSM, transport, or
  /// deserialization failure; see the crate-level `Error` enum
  /// for the full set.
  pub async fn pcs_transitions_wait_to_complete(
    &self,
    token: &str,
    transition_id: &str,
  ) -> Result<TransitionResponse, Error> {
    let backoff = crate::common::poll::PollBackoff {
      initial_delay: time::Duration::from_secs(3),
      max_delay: time::Duration::from_secs(30),
      max_attempts: 40,
    };

    crate::common::poll::poll_until_with_backoff(
      backoff,
      || async {
        let transition =
          self.pcs_transitions_get_by_id(token, transition_id).await?;
        log::debug!(
          "Power '{}' summary - status: {}, failed: {}, in-progress: {}, succeeded: {}, total: {}",
          transition.operation,
          transition.transition_status,
          transition.task_counts.failed,
          transition.task_counts.in_progress,
          transition.task_counts.succeeded,
          transition.task_counts.total,
        );
        Ok(transition)
      },
      |t| t.transition_status == "completed",
    )
    .await
  }
}
