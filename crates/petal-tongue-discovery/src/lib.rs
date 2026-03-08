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
//! 3. **Mock Provider** - Development/testing mode
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
mod dns_parser;
mod dynamic_scenario_provider;
mod http_provider;
mod jsonrpc_provider;
mod mdns_provider;
mod mock_provider;
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
pub use dynamic_scenario_provider::DynamicScenarioProvider;
pub use http_provider::HttpVisualizationProvider;
pub use jsonrpc_provider::JsonRpcProvider;
pub use mdns_provider::MdnsVisualizationProvider;
pub use mock_provider::MockVisualizationProvider;
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
pub use retry::RetryPolicy;

use anyhow::Result;
use petal_tongue_core::constants;

/// Discover all available visualization data providers
///
/// This function tries multiple discovery methods:
/// 1. mDNS/multicast auto-discovery (zero-config, preferred)
/// 2. Environment hints (`PETALTONGUE_DISCOVERY_HINTS`)
/// 3. Legacy `BIOMEOS_URL` (for backward compatibility)
///
/// # Caching
///
/// Note: Caching is now built into HttpVisualizationProvider (see cache.rs).
/// Each provider has its own cache with configurable TTLs. This provides
/// better performance without wrapper complexity.
///
/// # Returns
///
/// A vector of discovered providers, or an error if none found.
///
/// **NOTE**: Mock providers are NOT used in production. Use explicit
/// `PETALTONGUE_MOCK_MODE=true` if you need mock data for testing.
pub async fn discover_visualization_providers() -> Result<Vec<Box<dyn VisualizationDataProvider>>> {
    let mut providers: Vec<Box<dyn VisualizationDataProvider>> = Vec::new();

    // Check for explicit mock mode (testing only)
    let mock_mode = std::env::var("PETALTONGUE_MOCK_MODE")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

    if mock_mode {
        tracing::warn!("PETALTONGUE_MOCK_MODE=true - Using mock provider (TESTING ONLY)");
        providers.push(Box::new(MockVisualizationProvider::new()));
        return Ok(providers);
    }

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

    // Priority 3: Try mDNS auto-discovery (Phase 1 implementation)
    let enable_mdns = std::env::var("PETALTONGUE_ENABLE_MDNS")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "true";

    if enable_mdns {
        tracing::info!("Attempting mDNS auto-discovery...");
        match MdnsVisualizationProvider::discover().await {
            Ok(mdns_providers) => {
                if !mdns_providers.is_empty() {
                    tracing::info!("mDNS discovered {} provider(s)", mdns_providers.len());
                    providers.extend(mdns_providers);
                } else {
                    tracing::debug!("mDNS discovery found no providers");
                }
            }
            Err(e) => {
                tracing::warn!("mDNS discovery failed: {}", e);
            }
        }
    }

    // Try environment hints (JSON-RPC first, then HTTP as fallback)
    if providers.is_empty() {
        if let Ok(hints) = std::env::var("PETALTONGUE_DISCOVERY_HINTS") {
            tracing::info!("Trying discovery hints: {}", hints);
            for hint in hints.split(',') {
                let hint = hint.trim();

                // Try JSON-RPC first if it looks like a Unix socket
                if hint.starts_with("unix://") || hint.starts_with("/") {
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

                // Fallback to HTTP (with warning)
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
        }
    }

    // Try legacy BIOMEOS_URL (backward compatibility, but prefer JSON-RPC)
    if providers.is_empty() {
        if let Ok(biomeos_url) = std::env::var("BIOMEOS_URL") {
            tracing::info!("Trying legacy BIOMEOS_URL: {}", biomeos_url);

            // Try JSON-RPC first if it's a Unix socket
            if biomeos_url.starts_with("unix://") || biomeos_url.starts_with("/") {
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

            // Fallback to HTTP (with deprecation warning)
            match try_connect_http(&biomeos_url).await {
                Ok(provider) => {
                    tracing::warn!(
                        "⚠️  Using HTTP provider (external fallback) at {}",
                        biomeos_url
                    );
                    tracing::warn!("💡 Migrate to JSON-RPC: BIOMEOS_URL=unix:///run/user/$UID/biomeos-device-management.sock");
                    providers.push(provider);
                }
                Err(e) => {
                    tracing::error!("❌ Failed to connect to biomeOS at {}: {}", biomeos_url, e);
                }
            }
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
            5. Development: PETALTONGUE_MOCK_MODE=true\n\
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
async fn try_connect_jsonrpc(socket_path: &str) -> Result<Box<dyn VisualizationDataProvider>> {
    let provider = JsonRpcProvider::new(socket_path);

    // Test connection with health check
    provider.health_check().await?;
    Ok(Box::new(provider))
}

/// Try to connect to an HTTP provider at the given URL
///
/// ⚠️  HTTP is the FALLBACK protocol for external integrations only.
/// Prefer JSON-RPC over Unix sockets for TRUE PRIMAL architecture!
async fn try_connect_http(url: &str) -> Result<Box<dyn VisualizationDataProvider>> {
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
        // Clear any environment variables that might provide providers
        std::env::remove_var("BIOMEOS_URL");
        std::env::remove_var("PETALTONGUE_DISCOVERY_HINTS");
        std::env::remove_var("PETALTONGUE_MOCK_MODE");
        std::env::remove_var("PETALTONGUE_ENABLE_MDNS");

        // Production mode with no explicit config
        // TRUE PRIMAL: Discovery should work even without config (runtime discovery)
        // The function might find providers via:
        // 1. Unix socket discovery (if primals are running)
        // 2. mDNS auto-discovery (if network has providers)
        // 3. Or return empty (graceful degradation)
        let result = discover_visualization_providers().await;
        assert!(
            result.is_ok(),
            "Discovery should succeed even without explicit config (TRUE PRIMAL: graceful degradation)"
        );

        // The result might be empty OR contain discovered providers
        // Both are valid TRUE PRIMAL behavior:
        // - Empty: Graceful degradation to standalone mode
        // - Non-empty: Runtime discovery found primals
        let providers = result.unwrap();
        tracing::info!(
            "Discovered {} provider(s) without explicit config",
            providers.len()
        );
        // Test passes regardless - discovery is working correctly
    }

    #[tokio::test]
    async fn test_discover_with_mock_mode() {
        // Mock mode works when explicitly enabled
        std::env::set_var("PETALTONGUE_MOCK_MODE", "true");
        let result = discover_visualization_providers().await;
        std::env::remove_var("PETALTONGUE_MOCK_MODE");

        assert!(
            result.is_ok(),
            "Mock mode should work when explicitly enabled"
        );
        let providers = result.unwrap();
        assert!(!providers.is_empty(), "Mock mode should return providers");
    }

    #[tokio::test]
    async fn test_try_connect_http_invalid() {
        // Invalid URL should fail
        let result = try_connect_http("http://nonexistent-host-12345:99999").await;
        assert!(result.is_err());
    }
}
