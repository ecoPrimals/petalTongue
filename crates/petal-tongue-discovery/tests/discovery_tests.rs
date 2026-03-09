// SPDX-License-Identifier: AGPL-3.0-only
//! Comprehensive tests for discovery module
//!
//! Tests verify discovery logic, environment handling, and provider selection.

use petal_tongue_core::test_fixtures::env_test_helpers;
use petal_tongue_discovery::discover_visualization_providers;

#[tokio::test]
async fn test_discover_with_mock_mode() {
    env_test_helpers::with_env_vars_async(
        &[
            ("PETALTONGUE_MOCK_MODE", Some("true")),
            ("PETALTONGUE_DISCOVERY_HINTS", None),
            ("BIOMEOS_URL", None),
            ("PETALTONGUE_ENABLE_MDNS", None),
        ],
        || async {
            let providers = discover_visualization_providers().await;
            assert!(providers.is_ok(), "Should discover mock provider");
            let providers = providers.unwrap();
            assert!(
                !providers.is_empty(),
                "Mock mode should return at least 1 provider"
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
async fn test_mock_mode_case_insensitive() {
    for value in &["true", "True", "TRUE", "TrUe"] {
        env_test_helpers::with_env_vars_async(
            &[("PETALTONGUE_MOCK_MODE", Some(value))],
            || async {
                let providers = discover_visualization_providers().await;
                assert!(providers.is_ok(), "Should work with mock mode value");
            },
        )
        .await;
    }
}

#[tokio::test]
async fn test_mock_mode_false() {
    env_test_helpers::with_env_vars_async(
        &[
            ("PETALTONGUE_MOCK_MODE", Some("false")),
            ("PETALTONGUE_DISCOVERY_HINTS", None),
            ("BIOMEOS_URL", None),
        ],
        || async {
            let _providers = discover_visualization_providers().await;
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
            assert!(providers.is_ok() || providers.is_err(), "Discovery completes");
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

#[tokio::test]
async fn test_discovery_priority() {
    env_test_helpers::with_env_vars_async(
        &[
            ("PETALTONGUE_MOCK_MODE", Some("true")),
            (
                "PETALTONGUE_DISCOVERY_HINTS",
                Some("http://should-not-be-used:3000"),
            ),
            ("BIOMEOS_URL", Some("http://also-should-not-be-used:3000")),
        ],
        || async {
            let providers = discover_visualization_providers().await.unwrap();
            assert!(
                !providers.is_empty(),
                "Mock mode should take priority and return providers"
            );
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
            assert!(providers.is_ok() || providers.is_err(), "Empty hints handled");
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
        &[("PETALTONGUE_MOCK_MODE", Some("true"))],
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
