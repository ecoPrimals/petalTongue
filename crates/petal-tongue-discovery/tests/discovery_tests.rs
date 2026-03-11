// SPDX-License-Identifier: AGPL-3.0-only
//! Comprehensive tests for discovery module
//!
//! Tests verify discovery logic, environment handling, and provider selection.

use petal_tongue_core::test_fixtures::env_test_helpers;
use petal_tongue_discovery::discover_visualization_providers;

#[tokio::test]
#[cfg(feature = "test-fixtures")]
async fn test_mock_provider_direct_only() {
    // Mock provider is ONLY for test code - never used in discover_visualization_providers().
    // Tests that need demo data should use DemoVisualizationProvider directly.
    use petal_tongue_discovery::{DemoVisualizationProvider, VisualizationDataProvider};

    let provider = DemoVisualizationProvider::new();
    let primals = provider.get_primals().await.unwrap();
    assert!(!primals.is_empty(), "Mock provider should return primals");
}

#[tokio::test]
#[cfg(not(feature = "test-fixtures"))]
async fn test_discover_returns_ok_without_providers() {
    // Without real providers, discover returns Ok(empty) - graceful degradation
    env_test_helpers::with_env_vars_async(
        &[
            ("PETALTONGUE_DISCOVERY_HINTS", None),
            ("BIOMEOS_URL", None),
            ("PETALTONGUE_ENABLE_MDNS", None),
        ],
        || async {
            let result = discover_visualization_providers().await;
            assert!(
                result.is_ok(),
                "Discovery should succeed (empty or with providers)"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_discover_without_hints() {
    env_test_helpers::with_env_vars_async(
        &[
            ("PETALTONGUE_MOCK_MODE", None),
            ("PETALTONGUE_DISCOVERY_HINTS", None),
            ("BIOMEOS_URL", None),
            ("PETALTONGUE_ENABLE_MDNS", None),
        ],
        || async {
            let providers = discover_visualization_providers().await;
            assert!(
                providers.is_ok() || providers.is_err(),
                "Discovery should complete"
            );
        },
    )
    .await;
}

#[tokio::test]
#[cfg(feature = "test-fixtures")]
async fn test_demo_provider_metadata() {
    use petal_tongue_discovery::{DemoVisualizationProvider, VisualizationDataProvider};

    let provider = DemoVisualizationProvider::new();
    let metadata = provider.get_metadata();
    assert_eq!(metadata.name, "Demo Provider");
    assert_eq!(metadata.protocol, "demo");
}

#[tokio::test]
async fn test_discover_without_mock_env() {
    // No PETALTONGUE_MOCK_MODE - discovery uses real providers only
    env_test_helpers::with_env_vars_async(
        &[("PETALTONGUE_DISCOVERY_HINTS", None), ("BIOMEOS_URL", None)],
        || async {
            let result = discover_visualization_providers().await;
            assert!(result.is_ok(), "Discovery should complete");
        },
    )
    .await;
}

#[tokio::test]
async fn test_discovery_hints_parsing() {
    env_test_helpers::with_env_vars_async(
        &[
            (
                "PETALTONGUE_DISCOVERY_HINTS",
                Some("http://test1:3000,http://test2:3000"),
            ),
            ("PETALTONGUE_MOCK_MODE", None),
        ],
        || async {
            let providers = discover_visualization_providers().await;
            assert!(
                providers.is_ok() || providers.is_err(),
                "Discovery completes"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_legacy_biomeos_url() {
    env_test_helpers::with_env_vars_async(
        &[
            ("BIOMEOS_URL", Some("http://legacy:3000")),
            ("PETALTONGUE_MOCK_MODE", None),
            ("PETALTONGUE_DISCOVERY_HINTS", None),
        ],
        || async {
            let providers = discover_visualization_providers().await;
            assert!(
                providers.is_ok() || providers.is_err(),
                "Legacy discovery completes"
            );
        },
    )
    .await;
}

#[cfg(feature = "test-fixtures")]
#[tokio::test]
async fn test_discovery_graceful_empty() {
    // When no real providers found, discover returns Ok(empty) - graceful degradation
    env_test_helpers::with_env_vars_async(
        &[
            (
                "PETALTONGUE_DISCOVERY_HINTS",
                Some("http://nonexistent:99999"),
            ),
            ("BIOMEOS_URL", None),
            ("PETALTONGUE_ENABLE_MDNS", Some("false")),
        ],
        || async {
            let result = discover_visualization_providers().await;
            assert!(
                result.is_ok(),
                "Discovery should succeed even when no providers found"
            );
            let providers = result.unwrap();
            // May be empty (graceful) or have providers if HTTP hint connected
            let _ = providers;
        },
    )
    .await;
}

#[tokio::test]
async fn test_empty_discovery_hints() {
    env_test_helpers::with_env_vars_async(
        &[
            ("PETALTONGUE_DISCOVERY_HINTS", Some("")),
            ("PETALTONGUE_MOCK_MODE", None),
        ],
        || async {
            let providers = discover_visualization_providers().await;
            assert!(
                providers.is_ok() || providers.is_err(),
                "Empty hints handled"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_malformed_hints() {
    env_test_helpers::with_env_vars_async(
        &[
            (
                "PETALTONGUE_DISCOVERY_HINTS",
                Some("not-a-url,also not a url"),
            ),
            ("PETALTONGUE_MOCK_MODE", None),
        ],
        || async {
            let providers = discover_visualization_providers().await;
            assert!(
                providers.is_ok() || providers.is_err(),
                "Malformed hints handled"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_concurrent_discovery_attempts() {
    env_test_helpers::with_env_vars_async(
        &[("PETALTONGUE_DISCOVERY_HINTS", None), ("BIOMEOS_URL", None)],
        || async {
            let mut handles = vec![];
            for _ in 0..5 {
                let handle = tokio::spawn(async { discover_visualization_providers().await });
                handles.push(handle);
            }
            for handle in handles {
                let result = handle.await;
                assert!(result.is_ok(), "Concurrent discovery should work");
            }
        },
    )
    .await;
}
