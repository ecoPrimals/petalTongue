// SPDX-License-Identifier: AGPL-3.0-or-later
//! Concurrent discovery coordinator
//!
//! Modern async patterns for parallel provider discovery with timeout protection.

use crate::errors::{DiscoveryError, DiscoveryFailure};
use crate::traits::VisualizationDataProvider;
use futures::future::{join_all, select_all};
use std::future::Future;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(test)]
use async_trait::async_trait;

/// Result of concurrent discovery with graceful degradation
pub struct ConcurrentDiscoveryResult {
    /// Successfully discovered providers
    pub providers: Vec<Box<dyn VisualizationDataProvider>>,
    /// Sources that failed (for observability)
    pub failures: Vec<DiscoveryFailure>,
}

/// Discover providers from multiple sources concurrently
///
/// This function tries all discovery sources in parallel and returns
/// all successful providers, even if some sources fail.
///
/// # Example
///
/// ```rust,ignore
/// use petal_tongue_discovery::concurrent::discover_concurrent;
///
/// # async fn example() -> anyhow::Result<()> {
/// let result = discover_concurrent(
///     vec![
///         Box::new(mdns_discovery()),
///         Box::new(env_discovery()),
///         Box::new(http_discovery()),
///     ],
///     std::time::Duration::from_secs(5),
/// ).await;
///
/// println!("Found {} providers, {} failures",
///     result.providers.len(),
///     result.failures.len()
/// );
/// # Ok(())
/// # }
/// ```
pub async fn discover_concurrent<F, Fut>(
    sources: Vec<(&str, F)>,
    timeout_duration: Duration,
) -> ConcurrentDiscoveryResult
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<Vec<Box<dyn VisualizationDataProvider>>, DiscoveryError>> + Send,
{
    let mut all_providers = Vec::new();
    let mut failures = Vec::new();

    // Create futures for all discovery sources
    let discoveries: Vec<_> = sources
        .into_iter()
        .map(|(name, discover_fn)| {
            let fut = discover_fn();
            async move {
                match timeout(timeout_duration, fut).await {
                    Ok(Ok(providers)) => Ok((name, providers)),
                    Ok(Err(e)) => Err((name, e)),
                    Err(_) => Err((
                        name,
                        DiscoveryError::OperationTimedOut {
                            duration: timeout_duration,
                        },
                    )),
                }
            }
        })
        .collect();

    // Wait for all discoveries to complete (in parallel)
    let results = join_all(discoveries).await;

    // Process results
    for result in results {
        match result {
            Ok((name, providers)) => {
                tracing::info!(
                    "✅ Discovery source '{}' found {} provider(s)",
                    name,
                    providers.len()
                );
                all_providers.extend(providers);
            }
            Err((name, error)) => {
                tracing::warn!("❌ Discovery source '{}' failed: {}", name, error);
                failures.push(DiscoveryFailure::new(name, error));
            }
        }
    }

    ConcurrentDiscoveryResult {
        providers: all_providers,
        failures,
    }
}

/// Discover first available provider (race to first success)
///
/// Returns immediately once ANY provider succeeds, cancelling other attempts.
///
/// # Errors
///
/// Returns `DiscoveryError::NoProvidersFound` if providers list is empty,
/// `DiscoveryError::OperationTimedOut` if all health checks time out,
/// or `DiscoveryError::AllProvidersFailed` if all providers fail.
///
/// # Example
///
/// ```rust,ignore
/// use petal_tongue_discovery::concurrent::discover_first_available;
///
/// # async fn example() -> anyhow::Result<()> {
/// let provider = discover_first_available(
///     vec![
///         Box::new(primary_provider),
///         Box::new(fallback_provider),
///     ],
///     std::time::Duration::from_secs(5),
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn discover_first_available(
    providers: Vec<Box<dyn VisualizationDataProvider>>,
    timeout_duration: Duration,
) -> crate::errors::DiscoveryResult<Box<dyn VisualizationDataProvider>> {
    if providers.is_empty() {
        return Err(DiscoveryError::NoProvidersFound {
            attempted: 0,
            sources: "none".to_string(),
        });
    }

    let mut futures: Vec<_> = providers
        .into_iter()
        .map(|provider| {
            Box::pin(async move {
                timeout(timeout_duration, provider.health_check())
                    .await
                    .map_err(|_| DiscoveryError::OperationTimedOut {
                        duration: timeout_duration,
                    })??;
                Ok::<_, DiscoveryError>(provider)
            })
        })
        .collect();

    // Race to first success
    loop {
        if futures.is_empty() {
            return Err(DiscoveryError::AllProvidersFailed {
                count: futures.len(),
            });
        }

        let (result, _index, remaining) = select_all(futures).await;

        match result {
            Ok(provider) => {
                tracing::info!(
                    "✅ First available provider: {}",
                    provider.get_metadata().name
                );
                return Ok(provider);
            }
            Err(e) => {
                tracing::warn!("Provider failed: {}", e);
                futures = remaining;
            }
        }
    }
}

/// Health check all providers in parallel
///
/// Returns health status for each provider without failing on errors.
pub async fn check_all_providers_health(
    providers: &[Box<dyn VisualizationDataProvider>],
    timeout_duration: Duration,
) -> Vec<ProviderHealth> {
    let checks = providers.iter().map(|provider| {
        let metadata = provider.get_metadata();
        async move {
            let start = std::time::Instant::now();
            let result = timeout(timeout_duration, provider.health_check()).await;
            let duration = start.elapsed();

            ProviderHealth {
                name: metadata.name,
                endpoint: metadata.endpoint,
                status: match result {
                    Ok(Ok(msg)) => HealthStatus::Healthy {
                        message: msg,
                        response_time: duration,
                    },
                    Ok(Err(e)) => HealthStatus::Unhealthy {
                        error: e.to_string(),
                    },
                    Err(_) => HealthStatus::Timeout {
                        duration: timeout_duration,
                    },
                },
                checked_at: std::time::Instant::now(),
            }
        }
    });

    join_all(checks).await
}

/// Health status for a provider
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub name: String,
    pub endpoint: String,
    pub status: HealthStatus,
    pub checked_at: std::time::Instant,
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy {
        message: String,
        response_time: Duration,
    },
    Unhealthy {
        error: String,
    },
    Timeout {
        duration: Duration,
    },
}

impl ProviderHealth {
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo_provider::DemoVisualizationProvider;
    use crate::errors::DiscoveryError;
    use crate::traits::ProviderMetadata;

    /// Provider that always fails health_check - for testing error paths
    struct FailingProvider;

    #[async_trait]
    impl VisualizationDataProvider for FailingProvider {
        async fn get_primals(
            &self,
        ) -> crate::errors::DiscoveryResult<Vec<petal_tongue_core::PrimalInfo>> {
            Ok(vec![])
        }

        async fn health_check(&self) -> crate::errors::DiscoveryResult<String> {
            Err(DiscoveryError::ConfigError(
                "Intentional failure".to_string(),
            ))
        }

        fn get_metadata(&self) -> ProviderMetadata {
            ProviderMetadata {
                name: "Failing".to_string(),
                endpoint: "fail://".to_string(),
                protocol: "fail".to_string(),
                capabilities: vec![],
            }
        }
    }

    #[tokio::test]
    async fn test_parallel_health_checks() {
        let providers: Vec<Box<dyn VisualizationDataProvider>> = vec![
            Box::new(DemoVisualizationProvider::new()),
            Box::new(DemoVisualizationProvider::new()),
            Box::new(DemoVisualizationProvider::new()),
        ];

        let start = std::time::Instant::now();
        let health = check_all_providers_health(&providers, Duration::from_secs(1)).await;
        let elapsed = start.elapsed();

        assert_eq!(health.len(), 3);
        // Should be much faster than sequential (3 * delay)
        assert!(
            elapsed < Duration::from_millis(200),
            "Should run in parallel"
        );
    }

    #[tokio::test]
    async fn test_first_available_success() {
        let providers: Vec<Box<dyn VisualizationDataProvider>> = vec![
            Box::new(DemoVisualizationProvider::new()),
            Box::new(DemoVisualizationProvider::new()),
        ];

        let result = discover_first_available(providers, Duration::from_secs(1)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_first_available_empty() {
        let providers: Vec<Box<dyn VisualizationDataProvider>> = vec![];

        let result = discover_first_available(providers, Duration::from_secs(1)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_first_available_all_fail() {
        let providers: Vec<Box<dyn VisualizationDataProvider>> =
            vec![Box::new(FailingProvider), Box::new(FailingProvider)];

        let result = discover_first_available(providers, Duration::from_secs(1)).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(
                e.to_string().contains("All providers failed") || e.to_string().contains("failed")
            );
        }
    }

    #[tokio::test]
    async fn test_check_all_providers_health_with_unhealthy() {
        let providers: Vec<Box<dyn VisualizationDataProvider>> = vec![
            Box::new(DemoVisualizationProvider::new()),
            Box::new(FailingProvider),
        ];

        let health = check_all_providers_health(&providers, Duration::from_secs(1)).await;
        assert_eq!(health.len(), 2);
        assert!(health[0].is_healthy());
        assert!(!health[1].is_healthy());
    }

    #[tokio::test]
    async fn test_concurrent_with_failures() {
        // Test graceful degradation when some sources fail
        let result = ConcurrentDiscoveryResult {
            providers: vec![Box::new(DemoVisualizationProvider::new())],
            failures: vec![
                DiscoveryFailure::new("mDNS", "Timeout"),
                DiscoveryFailure::new("HTTP", "Connection refused"),
            ],
        };

        assert_eq!(result.providers.len(), 1);
        assert_eq!(result.failures.len(), 2);
        assert_eq!(result.failures[0].source, "mDNS");
        assert_eq!(result.failures[1].source, "HTTP");
    }

    #[tokio::test]
    async fn test_health_status_variants() {
        let healthy = HealthStatus::Healthy {
            message: "OK".to_string(),
            response_time: Duration::from_millis(50),
        };
        let unhealthy = HealthStatus::Unhealthy {
            error: "Service unavailable".to_string(),
        };
        let timeout = HealthStatus::Timeout {
            duration: Duration::from_secs(5),
        };

        assert!(matches!(healthy, HealthStatus::Healthy { .. }));
        assert!(matches!(unhealthy, HealthStatus::Unhealthy { .. }));
        assert!(matches!(timeout, HealthStatus::Timeout { .. }));
    }

    #[tokio::test]
    async fn test_provider_health_is_healthy() {
        let healthy = ProviderHealth {
            name: "test".to_string(),
            endpoint: "http://test:8080".to_string(),
            status: HealthStatus::Healthy {
                message: "OK".to_string(),
                response_time: Duration::from_millis(10),
            },
            checked_at: std::time::Instant::now(),
        };

        let unhealthy = ProviderHealth {
            name: "test2".to_string(),
            endpoint: "http://test2:8080".to_string(),
            status: HealthStatus::Unhealthy {
                error: "Error".to_string(),
            },
            checked_at: std::time::Instant::now(),
        };

        assert!(healthy.is_healthy());
        assert!(!unhealthy.is_healthy());
    }

    #[tokio::test]
    async fn test_parallel_faster_than_sequential() {
        let providers: Vec<Box<dyn VisualizationDataProvider>> = vec![
            Box::new(DemoVisualizationProvider::new()),
            Box::new(DemoVisualizationProvider::new()),
            Box::new(DemoVisualizationProvider::new()),
        ];

        let start = std::time::Instant::now();
        let health = check_all_providers_health(&providers, Duration::from_secs(1)).await;
        let elapsed = start.elapsed();

        assert_eq!(health.len(), 3);
        assert!(
            elapsed < Duration::from_millis(500),
            "Parallel execution should be fast, took {elapsed:?}"
        );
    }

    #[tokio::test]
    async fn test_discover_concurrent_single_success() {
        let ok_providers =
            vec![Box::new(DemoVisualizationProvider::new()) as Box<dyn VisualizationDataProvider>];
        let result = discover_concurrent(
            vec![("success", || async { Ok(ok_providers) })],
            Duration::from_secs(2),
        )
        .await;

        assert_eq!(result.providers.len(), 1);
        assert!(result.failures.is_empty());
    }

    #[tokio::test]
    async fn test_discover_concurrent_single_fail() {
        let result = discover_concurrent(
            vec![("fail", || async {
                Err(DiscoveryError::ConfigError(
                    "Intentional failure".to_string(),
                ))
            })],
            Duration::from_secs(2),
        )
        .await;

        assert!(result.providers.is_empty());
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].source, "fail");
    }

    #[tokio::test]
    async fn test_discover_concurrent_timeout() {
        let result = discover_concurrent(
            vec![("slow", || async {
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok(vec![])
            })],
            Duration::from_millis(50),
        )
        .await;

        assert!(result.providers.is_empty());
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].source, "slow");
    }

    #[tokio::test]
    async fn test_provider_health_timeout_variant() {
        let ph = ProviderHealth {
            name: "t".to_string(),
            endpoint: "e".to_string(),
            status: HealthStatus::Timeout {
                duration: Duration::from_secs(5),
            },
            checked_at: std::time::Instant::now(),
        };
        assert!(!ph.is_healthy());
    }

    #[tokio::test]
    async fn test_concurrent_discovery_result_structure() {
        let result = ConcurrentDiscoveryResult {
            providers: vec![],
            failures: vec![DiscoveryFailure::new("src", "err")],
        };
        assert!(result.providers.is_empty());
        assert_eq!(result.failures[0].error, "err");
    }
}
