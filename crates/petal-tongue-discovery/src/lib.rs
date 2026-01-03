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
mod http_provider;
mod mdns_provider;
mod mock_provider;
mod traits; // Phase 2: Caching layer (complete)

// Modern async patterns (Discovery Evolution)
pub mod concurrent;
pub mod errors;
pub mod retry;

#[cfg(feature = "mdns")]
mod mdns_discovery;

pub use cache::CacheStats;
pub use capabilities::VisualizationCapability;
pub use http_provider::HttpVisualizationProvider;
pub use mdns_provider::MdnsVisualizationProvider;
pub use mock_provider::MockVisualizationProvider;
pub use traits::{ProviderMetadata, VisualizationDataProvider}; // Export cache stats for monitoring

// Re-export modern patterns
pub use concurrent::{ConcurrentDiscoveryResult, HealthStatus, ProviderHealth};
pub use errors::{DiscoveryError, DiscoveryFailure, DiscoveryResult};
pub use retry::RetryPolicy;

use anyhow::Result;

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

    // Try mDNS auto-discovery (Phase 1 implementation)
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

    // Try environment hints
    if providers.is_empty() {
        if let Ok(hints) = std::env::var("PETALTONGUE_DISCOVERY_HINTS") {
            tracing::info!("Trying discovery hints: {}", hints);
            for hint in hints.split(',') {
                let hint = hint.trim();
                match try_connect_http(hint).await {
                    Ok(provider) => {
                        tracing::info!("✅ Connected to provider at {}", hint);
                        providers.push(provider);
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to connect to {}: {}", hint, e);
                    }
                }
            }
        }
    }

    // Try legacy BIOMEOS_URL (backward compatibility)
    if providers.is_empty() {
        if let Ok(biomeos_url) = std::env::var("BIOMEOS_URL") {
            tracing::info!("Trying legacy BIOMEOS_URL: {}", biomeos_url);
            match try_connect_http(&biomeos_url).await {
                Ok(provider) => {
                    tracing::info!("✅ Connected to legacy biomeOS at {}", biomeos_url);
                    providers.push(provider);
                }
                Err(e) => {
                    tracing::error!("❌ Failed to connect to biomeOS at {}: {}", biomeos_url, e);
                }
            }
        }
    }

    // NO FALLBACK TO MOCK - Fail properly if no providers found
    if providers.is_empty() {
        anyhow::bail!(
            "No visualization data providers found!\n\
            \n\
            Please configure at least one provider:\n\
            \n\
            Option 1: Automatic mDNS discovery (zero-config)\n\
            - Ensure biomeOS API is running with mDNS enabled\n\
            - Set PETALTONGUE_ENABLE_MDNS=true (default)\n\
            \n\
            Option 2: Manual configuration\n\
            - Set BIOMEOS_URL=http://localhost:3000\n\
            - Or set PETALTONGUE_DISCOVERY_HINTS=http://provider1:8080,http://provider2:9000\n\
            \n\
            Option 3: Testing/Development only\n\
            - Set PETALTONGUE_MOCK_MODE=true (NOT for production!)"
        );
    }

    tracing::info!(
        "✅ Discovery complete: {} provider(s) available",
        providers.len()
    );
    Ok(providers)
}

/// Try to connect to an HTTP provider at the given URL
async fn try_connect_http(url: &str) -> Result<Box<dyn VisualizationDataProvider>> {
    let provider = HttpVisualizationProvider::new(url);

    // Test connection with health check
    provider.health_check().await?;
    Ok(Box::new(provider))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_returns_error_without_config() {
        // Clear any environment variables that might provide providers
        std::env::remove_var("BIOMEOS_URL");
        std::env::remove_var("PETALTONGUE_DISCOVERY_HINTS");
        std::env::remove_var("PETALTONGUE_MOCK_MODE");
        std::env::remove_var("PETALTONGUE_ENABLE_MDNS");

        // Production mode requires explicit configuration - no automatic fallback
        let result = discover_visualization_providers().await;
        assert!(
            result.is_err(),
            "Should return error when no providers configured (production mode)"
        );
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
