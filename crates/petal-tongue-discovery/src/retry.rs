// SPDX-License-Identifier: AGPL-3.0-or-later
//! Retry logic with exponential backoff and jitter
//!
//! Modern async retry patterns for transient failures.

use petal_tongue_core::constants;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of attempts (including initial)
    pub max_attempts: usize,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier (typically 2.0 for exponential)
    pub backoff_factor: f64,
    /// Add random jitter to prevent thundering herd
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: constants::default_retry_initial_delay(),
            max_delay: constants::default_retry_max_delay(),
            backoff_factor: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy
    #[must_use]
    pub fn new(max_attempts: usize) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }

    /// Execute a function with retry logic
    ///
    /// # Errors
    ///
    /// Returns the last error from the closure if all attempts fail.
    ///
    /// # Panics
    ///
    /// Panics if the loop completes without setting `last_error` (should never happen).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use petal_tongue_discovery::retry::RetryPolicy;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let policy = RetryPolicy::default();
    /// let result = policy.execute(|| async {
    ///     // Potentially failing operation
    ///     reqwest::get("http://example.com").await
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute<F, Fut, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut delay = self.initial_delay;
        let attempts = self.max_attempts.max(1);
        let mut last_error: Option<E> = None;

        for attempt in 1..=attempts {
            match f().await {
                Ok(result) => {
                    if attempt > 1 {
                        tracing::info!("Succeeded on attempt {}/{attempts}", attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!("Attempt {attempt}/{attempts} failed: {e}");

                    last_error = Some(e);

                    if attempt < attempts {
                        let sleep_duration = if self.jitter {
                            add_jitter(delay)
                        } else {
                            delay
                        };

                        tracing::debug!("Retrying in {sleep_duration:?}...");
                        sleep(sleep_duration).await;

                        delay = (delay.mul_f64(self.backoff_factor)).min(self.max_delay);
                    }
                }
            }
        }

        #[expect(
            clippy::expect_used,
            reason = "loop runs at least once, last_error is always Some"
        )]
        Err(last_error.expect("at least one attempt always executes"))
    }

    /// Execute with timeout per attempt
    ///
    /// # Errors
    ///
    /// Returns `DiscoveryError::OperationTimedOut` if the operation times out,
    /// or `DiscoveryError::OperationFailed` if the closure returns an error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use petal_tongue_discovery::retry::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let policy = RetryPolicy::default();
    /// let result = policy.execute_with_timeout(
    ///     Duration::from_secs(5),
    ///     move || async move {
    ///         reqwest::get("http://example.com").await
    ///     }
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[expect(
        clippy::future_not_send,
        reason = "inner execute() captures F which may not be Sync; retry is typically single-threaded"
    )]
    pub async fn execute_with_timeout<F, Fut, T, E>(
        &self,
        timeout: Duration,
        f: F,
    ) -> Result<T, crate::errors::DiscoveryError>
    where
        F: Fn() -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.execute(|| async {
            tokio::time::timeout(timeout, f())
                .await
                .map_err(|_| crate::errors::DiscoveryError::OperationTimedOut {
                    duration: timeout,
                })?
                .map_err(|e| crate::errors::DiscoveryError::OperationFailed {
                    source: Box::new(e),
                })
        })
        .await
    }
}

/// Add random jitter to duration (±20%)
fn add_jitter(duration: Duration) -> Duration {
    use rand::Rng;
    let jitter_factor = rand::thread_rng().gen_range(0.8..1.2);
    duration.mul_f64(jitter_factor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_retry_succeeds_on_third_attempt() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_factor: 2.0,
            jitter: false,
        };

        let result = policy
            .execute(|| {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err("Not yet")
                    } else {
                        Ok("Success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_after_max_attempts() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_factor: 2.0,
            jitter: false,
        };

        let result = policy
            .execute(|| {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>("Always fails")
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_with_timeout() {
        let policy = RetryPolicy::default();

        let result = policy
            .execute_with_timeout(Duration::from_millis(50), || async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok::<_, std::io::Error>("Should timeout")
            })
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.backoff_factor, 2.0);
        assert!(policy.jitter);
    }

    #[test]
    fn test_retry_policy_new() {
        let policy = RetryPolicy::new(5);
        assert_eq!(policy.max_attempts, 5);
    }

    #[tokio::test]
    async fn test_retry_succeeds_immediately() {
        let policy = RetryPolicy::default();
        let result = policy
            .execute(|| async { Ok::<_, &str>("immediate") })
            .await;
        assert_eq!(result.unwrap(), "immediate");
    }

    #[tokio::test]
    async fn test_retry_with_jitter() {
        let policy = RetryPolicy {
            max_attempts: 2,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_factor: 2.0,
            jitter: true,
        };
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();
        let result = policy
            .execute(|| {
                let counter = c.clone();
                async move {
                    if counter.fetch_add(1, Ordering::SeqCst) < 1 {
                        Err::<(), _>("fail")
                    } else {
                        Ok(())
                    }
                }
            })
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_with_timeout_operation_fails() {
        let policy = RetryPolicy::default();
        let result = policy
            .execute_with_timeout(Duration::from_secs(2), || async {
                Err::<(), std::io::Error>(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "connection refused",
                ))
            })
            .await;
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(
            err_str.contains("connection refused") || err_str.contains("failed"),
            "got: {err_str}"
        );
    }

    #[tokio::test]
    async fn test_retry_max_delay_capped() {
        let policy = RetryPolicy {
            max_attempts: 2,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_millis(50),
            backoff_factor: 10.0,
            jitter: false,
        };
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c = counter.clone();
        let result = policy
            .execute(|| {
                let counter = c.clone();
                async move {
                    if counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) < 1 {
                        Err::<(), _>("fail")
                    } else {
                        Ok(())
                    }
                }
            })
            .await;
        assert!(result.is_ok());
    }
}
