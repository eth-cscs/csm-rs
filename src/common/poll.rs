//! Generic "poll until a condition is true" helper, used by the
//! state-wait loops scattered across the CSM API modules
//! (`capmc::utils::wait_nodes_to_power_*`, `cfs::session::utils::
//! wait_cfs_session_to_finish`, `pcs::transitions::http_client::
//! pcs_transitions_wait_to_complete`).
//!
//! Centralises three concerns those loops used to duplicate:
//!
//! - a max-attempt cap so a stuck remote can't wedge the caller,
//! - exponential backoff so a recovering peer isn't hammered, and
//! - jitter so multiple csm-rs callers aren't synchronised.

use std::time::Duration;

use crate::error::Error;

/// How [`poll_until_with_backoff`] paces its retries.
///
/// `initial_delay` is the first sleep between query attempts;
/// subsequent sleeps double (`initial_delay`, `2*initial_delay`,
/// `4*initial_delay`, …) up to `max_delay`. For constant-delay
/// polling, set `max_delay == initial_delay`. `max_attempts` is the
/// hard cap on query invocations.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PollBackoff {
  pub(crate) initial_delay: Duration,
  pub(crate) max_delay: Duration,
  pub(crate) max_attempts: u32,
}

/// Poll `query` repeatedly until `done(&result)` is true or
/// `max_attempts` invocations have completed. Sleeps with
/// exponential backoff capped at `max_delay`, with ±25 % jitter so
/// multiple concurrent callers don't fire in lockstep.
///
/// If `query` returns `Err`, the error short-circuits — partial
/// progress is not retried. If the attempt cap is reached without
/// `done` becoming true, returns the most recent observed value
/// (callers wanting a hard failure can re-check `done` themselves).
///
/// Panics if `max_attempts` is `0`; the caller must allow at least
/// one query.
pub(crate) async fn poll_until_with_backoff<T, F, Fut, D>(
  config: PollBackoff,
  mut query: F,
  done: D,
) -> Result<T, Error>
where
  F: FnMut() -> Fut,
  Fut: std::future::Future<Output = Result<T, Error>>,
  D: Fn(&T) -> bool,
{
  assert!(config.max_attempts > 0, "max_attempts must be > 0");

  let mut delay = config.initial_delay;
  let mut last: Option<T> = None;

  for attempt in 0..config.max_attempts {
    let value = query().await?;
    if done(&value) {
      return Ok(value);
    }
    if attempt + 1 < config.max_attempts {
      let slept = jittered(delay);
      log::debug!(
        "poll_until_with_backoff: attempt {}/{} not done, sleeping ~{:?}",
        attempt + 1,
        config.max_attempts,
        slept
      );
      tokio::time::sleep(slept).await;
      delay = delay.saturating_mul(2).min(config.max_delay);
    }
    last = Some(value);
  }

  // The `>= 1` assert plus the always-true last assignment above
  // guarantees `last` is `Some` once the loop has run.
  Ok(last.expect("at least one attempt always sets `last`"))
}

/// Apply ±25 % jitter to `d`, using the current wall-clock nanos as a
/// cheap entropy source. Not cryptographic — just enough randomness
/// to break up synchronised pollers.
fn jittered(d: Duration) -> Duration {
  let entropy = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map_or(0, |t| t.subsec_nanos());
  let pct = i64::from(entropy % 50) - 25; // -25..=24
  let nanos = d.as_nanos() as i64;
  let adjusted = nanos + (nanos * pct / 100);
  Duration::from_nanos(adjusted.max(0) as u64)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicU32, Ordering};

  #[tokio::test]
  async fn returns_first_value_that_satisfies_done() {
    let calls = AtomicU32::new(0);
    let result: u32 = poll_until_with_backoff(
      PollBackoff {
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(1),
        max_attempts: 10,
      },
      || async {
        let n = calls.fetch_add(1, Ordering::SeqCst) + 1;
        Ok::<_, Error>(n)
      },
      |&n| n == 3,
    )
    .await
    .expect("should succeed");
    assert_eq!(result, 3);
    assert_eq!(calls.load(Ordering::SeqCst), 3);
  }

  #[tokio::test]
  async fn returns_last_value_when_attempts_exhausted() {
    let result: u32 = poll_until_with_backoff(
      PollBackoff {
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(1),
        max_attempts: 3,
      },
      || async { Ok::<_, Error>(42) },
      |&_| false,
    )
    .await
    .expect("should succeed (returns last observed)");
    assert_eq!(result, 42);
  }

  #[tokio::test]
  async fn query_error_short_circuits() {
    let calls = AtomicU32::new(0);
    let err: Result<u32, _> = poll_until_with_backoff(
      PollBackoff {
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(1),
        max_attempts: 10,
      },
      || async {
        let n = calls.fetch_add(1, Ordering::SeqCst) + 1;
        if n == 2 {
          Err(Error::Message("boom".into()))
        } else {
          Ok(n)
        }
      },
      |&_| false,
    )
    .await;
    match err {
      Err(Error::Message(m)) => assert_eq!(m, "boom"),
      other => panic!("expected Message('boom'), got {other:?}"),
    }
    assert_eq!(calls.load(Ordering::SeqCst), 2);
  }

  #[test]
  fn jittered_stays_within_band() {
    let d = Duration::from_secs(1);
    for _ in 0..50 {
      let j = jittered(d);
      assert!(j >= Duration::from_millis(750));
      assert!(j <= Duration::from_millis(1240));
    }
  }
}
