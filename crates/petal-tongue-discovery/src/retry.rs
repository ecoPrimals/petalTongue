//! Retry logic with exponential backoff and jitter
//!
//! Modern async retry patterns for transient failures.

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
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_factor: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(max_attempts: usize) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }

    /// Execute a function with retry logic
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
        let mut last_error = None;

        for attempt in 1..=self.max_attempts {
            match f().await {
                Ok(result) => {
                    if attempt > 1 {
                        tracing::info!("Succeeded on attempt {}/{}", attempt, self.max_attempts);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!("Attempt {}/{} failed: {}", attempt, self.max_attempts, e);

                    last_error = Some(e);

                    if attempt < self.max_attempts {
                        let sleep_duration = if self.jitter {
                            add_jitter(delay)
                        } else {
                            delay
                        };

                        tracing::debug!("Retrying in {:?}...", sleep_duration);
                        sleep(sleep_duration).await;

                        // Exponential backoff
                        delay = (delay.mul_f64(self.backoff_factor)).min(self.max_delay);
                    }
                }
            }
        }

        Err(last_error.expect("last_error should be Some after loop"))
    }

    /// Execute with timeout per attempt
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
    pub async fn execute_with_timeout<F, Fut, T, E>(
        &self,
        timeout: Duration,
        f: F,
    ) -> Result<T, anyhow::Error>
    where
        F: Fn() -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.execute(|| async {
            tokio::time::timeout(timeout, f())
                .await
                .map_err(|_| anyhow::anyhow!("Operation timed out after {:?}", timeout))?
                .map_err(|e| anyhow::anyhow!(e))
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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

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
}
