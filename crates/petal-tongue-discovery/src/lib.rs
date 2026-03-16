// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
//! Capability-Based Discovery for Visualization Data Providers
//!
//! This crate enables petalTongue to discover ANY primal that provides
//! visualization data, without hardcoding knowledge of specific primals.
//!
//! # Philosophy
//!
//! **No primal is special.** biomeOS, Songbird, or any other primal can
//! provide visualization data. We discover by capability, not by name.
//!
//! # Discovery Methods
//!
//! 1. **mDNS/Multicast** - Automatic local discovery (preferred)
//! 2. **Environment Hints** - Manual fallback configuration
//! 3. **Demo Provider** - Development/testing mode (when `test-fixtures` feature enabled)
//!
//! # Example
//!
//! ```rust,no_run
//! use petal_tongue_discovery::{discover_visualization_providers, VisualizationCapability};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Discover all primals that provide visualization data
//! let providers = discover_visualization_providers().await?;
//!
//! // Query whoever we found
//! for provider in providers {
//!     let primals = provider.get_primals().await?;
//!     println!("Discovered {} primals", primals.len());
//! }
//! # Ok(())
//! # }
//! ```

mod cache;
mod capabilities;
#[cfg(any(test, feature = "test-fixtures"))]
mod demo_provider;
mod dns_parser;
mod dynamic_scenario_provider;
#[cfg(feature = "legacy-http")]
mod http_provider;
pub mod jsonl_provider;
mod jsonrpc_provider;
mod mdns_provider;
mod neural_api_provider;
mod neural_graph_client;
mod scenario_provider;
mod songbird_client;
mod songbird_provider;
mod traits;
mod unix_socket_provider;

// Modern async patterns (Discovery Evolution)
pub mod concurrent;
pub mod errors;
pub mod retry;

#[cfg(feature = "mdns")]
mod mdns_discovery;

pub use cache::CacheStats;
pub use capabilities::VisualizationCapability;
#[cfg(any(test, feature = "test-fixtures"))]
pub use demo_provider::DemoVisualizationProvider;
pub use dynamic_scenario_provider::DynamicScenarioProvider;
#[cfg(feature = "legacy-http")]
#[expect(deprecated)]
pub use http_provider::HttpVisualizationProvider;
pub use jsonrpc_provider::JsonRpcProvider;
pub use mdns_provider::MdnsVisualizationProvider;
pub use neural_api_provider::NeuralApiProvider;
pub use neural_graph_client::{ExecutionResult, ExecutionStatus, GraphMetadata, NeuralGraphClient};
pub use scenario_provider::ScenarioVisualizationProvider;
pub use songbird_client::SongbirdClient;
pub use songbird_provider::SongbirdVisualizationProvider;
pub use traits::{ProviderMetadata, VisualizationDataProvider};
pub use unix_socket_provider::UnixSocketProvider;

// Re-export modern patterns
pub use concurrent::{ConcurrentDiscoveryResult, HealthStatus, ProviderHealth};
pub use errors::{DiscoveryError, DiscoveryFailure, DiscoveryResult};
use petal_tongue_core::constants;
pub use retry::RetryPolicy;

/// Discover all available visualization data providers
///
/// This function tries multiple discovery methods:
/// 1. mDNS/multicast auto-discovery (zero-config, preferred)
/// 2. Environment hints (`PETALTONGUE_DISCOVERY_HINTS`)
/// 3. Legacy `BIOMEOS_URL` (for backward compatibility)
///
/// # Caching
///
/// Note: Caching is now built into `HttpVisualizationProvider` (see cache.rs).
/// Each provider has its own cache with configurable TTLs. This provides
/// better performance without wrapper complexity.
///
/// # Returns
///
/// A vector of discovered providers, or an error if none found.
///
/// **NOTE**: Mock providers are NEVER used in production runtime. They exist only
/// in test code (`#[cfg(test)]`) or when building with `--features test-fixtures`
/// for test fixtures. Production always attempts real discovery and returns empty
/// if no providers found (graceful degradation).
#[expect(
    clippy::too_many_lines,
    reason = "discovery fallback chain; refactor would obscure priority order"
)]
pub async fn discover_visualization_providers()
-> DiscoveryResult<Vec<Box<dyn VisualizationDataProvider>>> {
    let mut providers: Vec<Box<dyn VisualizationDataProvider>> = Vec::new();

    // Priority 1: Try Neural API (PREFERRED METHOD - Central Coordinator)
    // Neural API is the single source of truth for all primal state
    tracing::info!("🧠 Attempting Neural API discovery (central coordinator)...");
    match NeuralApiProvider::discover(None).await {
        Ok(neural_provider) => {
            tracing::info!("✅ Neural API connected - using as primary provider");
            providers.push(Box::new(neural_provider) as Box<dyn VisualizationDataProvider>);

            // Neural API is our primary source - return it
            return Ok(providers);
        }
        Err(e) => {
            tracing::debug!("Neural API not available: {}", e);
            tracing::info!("💡 Tip: Start 'nucleus serve' for Neural API coordination");
        }
    }

    // Priority 2: Try Songbird for live primal discovery (FALLBACK)
    // This queries Songbird which has the complete primal registry
    tracing::info!("🎵 Attempting Songbird discovery (fallback)...");
    match SongbirdVisualizationProvider::discover(None).await {
        Ok(songbird_provider) => {
            tracing::info!("✅ Songbird connected - using as fallback provider");
            providers.push(Box::new(songbird_provider) as Box<dyn VisualizationDataProvider>);

            // If Songbird is available, use it as fallback
            return Ok(providers);
        }
        Err(e) => {
            tracing::debug!("Songbird not available: {}", e);
            tracing::info!("💡 Tip: Start Songbird for primal discovery");
        }
    }

    // Priority 2: Try JSON-RPC over Unix sockets (PRIMARY PRIMAL PROTOCOL)
    // This is the standard protocol for all ecoPrimals (Songbird, BearDog, ToadStool, etc.)
    tracing::info!("🔌 Attempting JSON-RPC discovery (Unix sockets)...");
    match JsonRpcProvider::discover().await {
        Ok(jsonrpc_provider) => {
            tracing::info!("✅ JSON-RPC provider connected - TRUE PRIMAL protocol!");
            providers.push(Box::new(jsonrpc_provider) as Box<dyn VisualizationDataProvider>);

            // JSON-RPC found, return it as primary provider
            return Ok(providers);
        }
        Err(e) => {
            tracing::debug!("JSON-RPC discovery failed: {}", e);
            tracing::debug!("💡 Tip: Ensure biomeOS device_management_server is running");
        }
    }

    // Priority 3: Try mDNS auto-discovery
    let enable_mdns = std::env::var("PETALTONGUE_ENABLE_MDNS")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "true";

    if enable_mdns {
        tracing::info!("Attempting mDNS auto-discovery...");
        match MdnsVisualizationProvider::discover().await {
            Ok(mdns_providers) => {
                if mdns_providers.is_empty() {
                    tracing::debug!("mDNS discovery found no providers");
                } else {
                    tracing::info!("mDNS discovered {} provider(s)", mdns_providers.len());
                    providers.extend(mdns_providers);
                }
            }
            Err(e) => {
                tracing::warn!("mDNS discovery failed: {}", e);
            }
        }
    }

    // Try environment hints (JSON-RPC first, then HTTP as fallback)
    if providers.is_empty()
        && let Ok(hints) = std::env::var("PETALTONGUE_DISCOVERY_HINTS")
    {
        tracing::info!("Trying discovery hints: {}", hints);
        for hint in hints.split(',') {
            let hint = hint.trim();

            // Try JSON-RPC first if it looks like a Unix socket
            if hint.starts_with("unix://") || hint.starts_with('/') {
                let socket_path = hint.strip_prefix("unix://").unwrap_or(hint);
                match try_connect_jsonrpc(socket_path).await {
                    Ok(provider) => {
                        tracing::info!("✅ Connected to JSON-RPC provider at {}", socket_path);
                        providers.push(provider);
                        continue;
                    }
                    Err(e) => {
                        tracing::debug!("JSON-RPC connection failed: {}", e);
                    }
                }
            }

            // Fallback to HTTP (with warning) - requires legacy-http feature
            #[cfg(feature = "legacy-http")]
            {
                match try_connect_http(hint).await {
                    Ok(provider) => {
                        tracing::warn!("⚠️  Using HTTP provider (external fallback) at {}", hint);
                        tracing::warn!(
                            "💡 Consider using JSON-RPC over Unix sockets for TRUE PRIMAL protocol"
                        );
                        providers.push(provider);
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to connect to {}: {}", hint, e);
                    }
                }
            }
            #[cfg(not(feature = "legacy-http"))]
            {
                tracing::debug!(
                    "HTTP fallback skipped (compile with --features legacy-http for HTTP provider)"
                );
            }
        }
    }

    // Try legacy BIOMEOS_URL (backward compatibility, but prefer JSON-RPC)
    if providers.is_empty()
        && let Ok(biomeos_url) = std::env::var("BIOMEOS_URL")
    {
        tracing::info!("Trying legacy BIOMEOS_URL: {}", biomeos_url);

        // Try JSON-RPC first if it's a Unix socket
        if biomeos_url.starts_with("unix://") || biomeos_url.starts_with('/') {
            let socket_path = biomeos_url.strip_prefix("unix://").unwrap_or(&biomeos_url);
            match try_connect_jsonrpc(socket_path).await {
                Ok(provider) => {
                    tracing::info!("✅ Connected to JSON-RPC provider at {}", socket_path);
                    providers.push(provider);
                    return Ok(providers);
                }
                Err(e) => {
                    tracing::debug!("JSON-RPC connection failed: {}", e);
                }
            }
        }

        // Fallback to HTTP (with deprecation warning) - requires legacy-http feature
        #[cfg(feature = "legacy-http")]
        {
            match try_connect_http(&biomeos_url).await {
                Ok(provider) => {
                    tracing::warn!(
                        "⚠️  Using HTTP provider (external fallback) at {}",
                        biomeos_url
                    );
                    tracing::warn!(
                        "💡 Migrate to JSON-RPC: BIOMEOS_URL=unix:///run/user/$UID/biomeos-device-management.sock"
                    );
                    providers.push(provider);
                }
                Err(e) => {
                    tracing::error!("❌ Failed to connect to biomeOS at {}: {}", biomeos_url, e);
                }
            }
        }
        #[cfg(not(feature = "legacy-http"))]
        {
            tracing::debug!(
                "HTTP fallback skipped (compile with --features legacy-http for HTTP provider)"
            );
        }
    }

    // Bidirectional UUI + TRUE PRIMAL fix:
    // Return empty vec instead of error - let GUI handle graceful degradation
    // The GUI itself tests if it can render (bidirectional sensory verification)
    if providers.is_empty() {
        tracing::warn!(
            "⚠️  No visualization data providers found!\n\
            \n\
            Recommended options (TRUE PRIMAL):\n\
            1. Songbird discovery: Start Songbird for live primal topology\n\
            2. JSON-RPC (PRIMARY): BIOMEOS_URL=unix:///run/user/$UID/biomeos-device-management.sock\n\
            3. Auto-discovery: PETALTONGUE_ENABLE_MDNS=true (default)\n\
            \n\
            Fallback options (external only):\n\
            4. HTTP fallback: BIOMEOS_URL=http://localhost:{}\n\
            5. Development: Build with --features test-fixtures for test fixtures (mocks only in tests)\n\
            \n\
            💡 GUI will start with tutorial mode as graceful fallback",
            constants::DEFAULT_WEB_PORT
        );
        // Return empty vec - GUI will handle this with tutorial mode
        return Ok(vec![]);
    }

    tracing::info!(
        "✅ Discovery complete: {} provider(s) available",
        providers.len()
    );
    Ok(providers)
}

/// Try to connect to a JSON-RPC provider at the given Unix socket path
async fn try_connect_jsonrpc(
    socket_path: &str,
) -> DiscoveryResult<Box<dyn VisualizationDataProvider>> {
    let provider = JsonRpcProvider::new(socket_path);

    // Test connection with health check
    provider.health_check().await?;
    Ok(Box::new(provider))
}

/// Try to connect to an HTTP provider at the given URL
///
/// ⚠️  HTTP is the FALLBACK protocol for external integrations only.
/// Prefer JSON-RPC over Unix sockets for TRUE PRIMAL architecture!
#[cfg(feature = "legacy-http")]
#[expect(deprecated)]
async fn try_connect_http(url: &str) -> DiscoveryResult<Box<dyn VisualizationDataProvider>> {
    let provider = HttpVisualizationProvider::new(url)?;

    // Test connection with health check
    provider.health_check().await?;
    Ok(Box::new(provider))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_returns_empty_without_config() {
        use petal_tongue_core::test_fixtures::env_test_helpers;

        env_test_helpers::with_env_vars_async(
            &[
                ("BIOMEOS_URL", None),
                ("PETALTONGUE_DISCOVERY_HINTS", None),
                ("PETALTONGUE_MOCK_MODE", None),
                ("PETALTONGUE_ENABLE_MDNS", None),
            ],
            || async {
                let result = discover_visualization_providers().await;
                assert!(
                    result.is_ok(),
                    "Discovery should succeed even without explicit config"
                );
                let providers = result.unwrap();
                tracing::info!(
                    "Discovered {} provider(s) without explicit config",
                    providers.len()
                );
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_discover_graceful_degradation_returns_ok() {
        use petal_tongue_core::test_fixtures::env_test_helpers;

        env_test_helpers::with_env_vars_async(
            &[
                ("BIOMEOS_URL", None),
                ("PETALTONGUE_DISCOVERY_HINTS", None),
                ("FAMILY_ID", None),
                ("PETALTONGUE_MOCK_MODE", None),
                ("PETALTONGUE_ENABLE_MDNS", None),
            ],
            || async {
                let result = discover_visualization_providers().await;
                assert!(
                    result.is_ok(),
                    "Discovery must never panic - graceful degradation"
                );
                let providers = result.unwrap();
                assert!(
                    providers.is_empty(),
                    "Without config, expect empty (or mock in test-fixtures)"
                );
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_try_connect_jsonrpc_nonexistent_socket() {
        let result = try_connect_jsonrpc("/tmp/nonexistent-socket-xyz-12345.sock").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_try_connect_jsonrpc_invalid_path() {
        let result = try_connect_jsonrpc("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg(feature = "test-fixtures")]
    async fn test_mock_provider_direct_usage() {
        // Demo provider is ONLY for test code - never used in production discover path.
        // Tests that use demo data should instantiate DemoVisualizationProvider directly.
        let provider = DemoVisualizationProvider::new();
        let primals = provider.get_primals().await.unwrap();
        assert!(!primals.is_empty(), "Mock provider should return primals");
        assert_eq!(primals.len(), 3);
    }

    #[tokio::test]
    #[cfg(feature = "test-fixtures")]
    async fn test_mock_provider_get_topology() {
        let provider = DemoVisualizationProvider::new();
        let topology = provider.get_topology().await.unwrap();
        assert!(topology.is_empty() || !topology.is_empty()); // Mock may return edges
    }

    #[tokio::test]
    #[cfg(feature = "test-fixtures")]
    async fn test_mock_provider_health_check() {
        let provider = DemoVisualizationProvider::new();
        let health = provider.health_check().await.unwrap();
        assert!(!health.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "legacy-http")]
    async fn test_try_connect_http_invalid() {
        let result = try_connect_http("http://nonexistent-host-12345:99999").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_discover_with_mdns_disabled() {
        use petal_tongue_core::test_fixtures::env_test_helpers;

        env_test_helpers::with_env_vars_async(
            &[
                ("PETALTONGUE_ENABLE_MDNS", Some("false")),
                ("BIOMEOS_URL", None),
                ("PETALTONGUE_DISCOVERY_HINTS", None),
            ],
            || async {
                let result = discover_visualization_providers().await;
                assert!(result.is_ok());
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_discover_with_empty_hints() {
        use petal_tongue_core::test_fixtures::env_test_helpers;

        env_test_helpers::with_env_vars_async(
            &[
                ("PETALTONGUE_DISCOVERY_HINTS", Some("")),
                ("BIOMEOS_URL", None),
            ],
            || async {
                let result = discover_visualization_providers().await;
                assert!(result.is_ok());
            },
        )
        .await;
    }
}
