use std::sync::Mutex;
use std::time::{Duration, Instant};

use domain::errors::errors::AppResult;
use tracing::{debug, warn};

struct RateState {
    next_allowed_at: Instant,
}

const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(350);
const DEFAULT_RETRY_AFTER: Duration = Duration::from_secs(3);
const LOCAL_AUTOWAIT_THRESHOLD: Duration = Duration::from_secs(1);

pub(crate) struct RequestScheduler {
    request_lock: Mutex<()>,
    rate_state: Mutex<RateState>,
}

impl RequestScheduler {
    pub fn new() -> Self {
        Self {
            request_lock: Mutex::new(()),
            rate_state: Mutex::new(RateState {
                next_allowed_at: Instant::now(),
            }),
        }
    }

    pub fn run<T, F>(&self, op_name: &str, mut op: F) -> AppResult<T>
    where
        F: FnMut() -> Result<T, ureq::Error>,
    {
        let _request_guard = match self.request_lock.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        self.try_acquire_rate_slot(op_name)?;
        match op() {
            Ok(value) => Ok(value),
            Err(ureq::Error::Status(429, response)) => {
                let wait = Self::parse_retry_after(&response);
                self.defer_requests(wait);
                warn!(
                    operation = op_name,
                    retry_after_secs = wait.as_secs(),
                    "Spotify returned 429 Too Many Requests"
                );
                Err(anyhow::anyhow!(
                    "{op_name} rate-limited by Spotify; retry in {}s",
                    wait.as_secs()
                )
                .into())
            }
            Err(ureq::Error::Status(status, response)) => {
                let status_text = response.status_text().to_string();
                Err(anyhow::anyhow!("{op_name} failed with status {status} {status_text}").into())
            }
            Err(ureq::Error::Transport(e)) => {
                Err(anyhow::anyhow!("{op_name} transport error: {e}").into())
            }
        }
    }

    fn try_acquire_rate_slot(&self, op_name: &str) -> AppResult<()> {
        let mut state = match self.rate_state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let now = Instant::now();
        if state.next_allowed_at > now {
            let wait = state.next_allowed_at.saturating_duration_since(now);
            if wait <= LOCAL_AUTOWAIT_THRESHOLD {
                debug!(
                    operation = op_name,
                    wait_ms = wait.as_millis(),
                    "Applying short local cooldown delay"
                );
                drop(state);
                std::thread::sleep(wait);
                let mut state = match self.rate_state.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                state.next_allowed_at = Instant::now() + MIN_REQUEST_INTERVAL;
                debug!(operation = op_name, "Rate limiter slot acquired after short wait");
                return Ok(());
            }

            let retry_after_secs = wait.as_secs().max(1);
            warn!(
                operation = op_name,
                retry_after_secs,
                retry_after_ms = wait.as_millis(),
                "Request blocked by local cooldown"
            );
            return Err(anyhow::anyhow!(
                "Spotify cooldown active for operation '{op_name}', retry in {}s",
                retry_after_secs
            )
            .into());
        }
        state.next_allowed_at = Instant::now() + MIN_REQUEST_INTERVAL;
        debug!(operation = op_name, "Rate limiter slot acquired");
        Ok(())
    }

    fn defer_requests(&self, wait: Duration) {
        let mut state = match self.rate_state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        state.next_allowed_at = std::cmp::max(state.next_allowed_at, Instant::now() + wait);
    }

    fn parse_retry_after(resp: &ureq::Response) -> Duration {
        let secs = resp
            .header("Retry-After")
            .and_then(|h| h.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_RETRY_AFTER.as_secs());
        Duration::from_secs(secs.max(1))
    }
}

