use std::str::FromStr;
use std::time;

use crate::{
  ShastaClient,
  common::http,
  error::Error,
  pcs::transitions::types::{
    Location, Operation, TransitionResponse, TransitionResponseList,
    TransitionStartOutput,
  },
};

use super::types::Transition;

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
    log::debug!("PCS transition details\n{:#?}", transition);
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
    log::debug!("Create PCS transition '{}' on {:?}", operation, xname_vec);

    let location_vec: Vec<Location> = xname_vec
      .iter()
      .map(|xname| Location {
        xname: xname.to_string(),
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
