// SPDX-License-Identifier: AGPL-3.0-or-later
//! Circuit breaker and retry policy for IPC resilience (absorbed from rhizoCrypt v0.13).
#![expect(
    missing_docs,
    reason = "kept under 250 lines; API matches rhizoCrypt v0.13"
)]

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_factor: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Exponential backoff for attempt (0-indexed), capped at max_delay.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let exponent = i32::try_from(attempt).unwrap_or(i32::MAX);
        let delay_secs = self.initial_delay.as_secs_f64() * self.backoff_factor.powi(exponent);
        let delay = Duration::from_secs_f64(delay_secs);
        delay.min(self.max_delay)
    }

    /// True if attempt < max_attempts.
    pub const fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    half_open_successes: u32,
    last_failure: Option<Instant>,
    recovery_timeout: Duration,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(30))
    }
}

impl CircuitBreaker {
    pub const fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            success_threshold: 1,
            half_open_successes: 0,
            last_failure: None,
            recovery_timeout,
        }
    }

    /// True if attempt allowed; may transition Open → HalfOpen when timeout elapsed.
    pub fn can_attempt(&mut self) -> bool {
        match self.state {
            CircuitState::Closed | CircuitState::HalfOpen => true,
            CircuitState::Open => {
                if let Some(last) = self.last_failure
                    && last.elapsed() >= self.recovery_timeout
                {
                    self.state = CircuitState::HalfOpen;
                    self.half_open_successes = 0;
                    return true;
                }
                false
            }
        }
    }

    pub const fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.half_open_successes += 1;
                if self.half_open_successes >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.half_open_successes = 0;
                }
            }
            CircuitState::Open => {}
        }
    }

    pub fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.last_failure = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.last_failure = Some(Instant::now());
            }
            CircuitState::Open => {
                self.last_failure = Some(Instant::now());
            }
        }
    }

    pub const fn state(&self) -> &CircuitState {
        &self.state
    }

    pub const fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.half_open_successes = 0;
        self.last_failure = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_policy_default() {
        let p = RetryPolicy::default();
        assert_eq!(p.max_attempts, 3);
        assert_eq!(p.initial_delay, Duration::from_millis(100));
        assert_eq!(p.max_delay, Duration::from_secs(10));
        assert_eq!(p.backoff_factor, 2.0);
    }

    #[test]
    fn retry_policy_delay() {
        let p = RetryPolicy::default();
        assert_eq!(p.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(p.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(p.delay_for_attempt(2), Duration::from_millis(400));
        assert!(p.delay_for_attempt(20) <= p.max_delay);
    }

    #[test]
    fn retry_policy_should_retry() {
        let p = RetryPolicy::default();
        assert!(p.should_retry(0));
        assert!(p.should_retry(2));
        assert!(!p.should_retry(3));
        assert!(!p.should_retry(4));
    }

    #[test]
    fn circuit_breaker_default() {
        let cb = CircuitBreaker::default();
        assert_eq!(cb.state(), &CircuitState::Closed);
    }

    #[test]
    fn circuit_breaker_closed_allows_attempts() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(1));
        assert!(cb.can_attempt());
        assert!(cb.can_attempt());
    }

    #[test]
    fn circuit_breaker_opens_after_threshold() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(1));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), &CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), &CircuitState::Open);
        assert!(!cb.can_attempt());
    }

    #[test]
    fn circuit_breaker_success_resets_failures() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(1));
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert_eq!(cb.state(), &CircuitState::Closed);
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), &CircuitState::Open);
    }

    #[test]
    fn circuit_breaker_half_open_transitions() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), &CircuitState::Open);
        std::thread::sleep(Duration::from_millis(15));
        assert!(cb.can_attempt());
        assert_eq!(cb.state(), &CircuitState::HalfOpen);
        cb.record_success();
        assert_eq!(cb.state(), &CircuitState::Closed);
        cb.record_failure();
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(15));
        let _ = cb.can_attempt();
        cb.record_failure();
        assert_eq!(cb.state(), &CircuitState::Open);
    }

    #[test]
    fn circuit_breaker_reset() {
        let mut cb = CircuitBreaker::new(2, Duration::from_secs(1));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), &CircuitState::Open);
        cb.reset();
        assert_eq!(cb.state(), &CircuitState::Closed);
        assert!(cb.can_attempt());
    }
}
