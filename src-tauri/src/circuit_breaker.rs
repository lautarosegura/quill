//! Simple sliding-window circuit breaker for the Groq cloud engine.
//! Transitions: Closed → Open(since) → HalfOpen → Closed|Open.

use std::time::{Duration, Instant};

use tokio::sync::Mutex;

#[derive(Debug)]
pub enum BreakerError {
    /// Requests are rejected without hitting the service.
    Open,
}

#[derive(Debug)]
enum State {
    Closed { failures: Vec<Instant> },
    Open { since: Instant },
    HalfOpen,
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    failure_window: Duration,
    reset_timeout: Duration,
    state: Mutex<State>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, failure_window_secs: u64, reset_timeout_secs: u64) -> Self {
        Self {
            failure_threshold,
            failure_window: Duration::from_secs(failure_window_secs),
            reset_timeout: Duration::from_secs(reset_timeout_secs),
            state: Mutex::new(State::Closed {
                failures: Vec::new(),
            }),
        }
    }

    /// Production default: 5 failures in 60s → Open for 30s.
    pub fn default_groq() -> Self {
        Self::new(5, 60, 30)
    }

    /// Returns Ok if the request should proceed, Err(Open) if the breaker is open.
    /// Side effect: if Open elapsed past the reset timeout, transitions to HalfOpen.
    pub async fn allow_request(&self) -> Result<(), BreakerError> {
        let mut state = self.state.lock().await;
        match &*state {
            State::Closed { .. } => Ok(()),
            State::Open { since } => {
                if since.elapsed() >= self.reset_timeout {
                    *state = State::HalfOpen;
                    Ok(())
                } else {
                    Err(BreakerError::Open)
                }
            }
            State::HalfOpen => Ok(()),
        }
    }

    pub async fn record_success(&self) {
        let mut state = self.state.lock().await;
        *state = State::Closed {
            failures: Vec::new(),
        };
    }

    pub async fn record_failure(&self) {
        let mut state = self.state.lock().await;
        match &mut *state {
            State::Closed { failures } => {
                let now = Instant::now();
                let cutoff = now - self.failure_window;
                failures.retain(|t| *t >= cutoff);
                failures.push(now);
                if failures.len() as u32 >= self.failure_threshold {
                    *state = State::Open { since: now };
                }
            }
            State::HalfOpen => {
                *state = State::Open {
                    since: Instant::now(),
                };
            }
            State::Open { .. } => {
                // Still open — no-op.
            }
        }
    }
}

#[cfg(test)]
mod tests;
