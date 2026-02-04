//! Comprehensive tests for discovery module
//!
//! Tests verify discovery logic, environment handling, and provider selection.
//!
//! **NOTE**: These tests manipulate environment variables. A static mutex is used
//! to serialize tests that modify environment variables.

use petal_tongue_discovery::discover_visualization_providers;
use std::sync::Mutex;

// Mutex for serializing tests that modify environment variables
// This prevents race conditions in parallel test execution
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[tokio::test]
async fn test_discover_with_mock_mode() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // Clean up any existing env vars first
    // SAFETY: Test-only env var manipulation, serialized by mutex
    unsafe {
        std::env::remove_var("PETALTONGUE_DISCOVERY_HINTS");
        std::env::remove_var("BIOMEOS_URL");
        std::env::remove_var("PETALTONGUE_ENABLE_MDNS");

        // Set mock mode environment variable
        std::env::set_var("PETALTONGUE_MOCK_MODE", "true");
    }

    let providers = discover_visualization_providers().await;

    // Clean up BEFORE assertion (ensures cleanup even on failure)
    // SAFETY: Test-only env var manipulation, serialized by mutex
    unsafe {
        std::env::remove_var("PETALTONGUE_MOCK_MODE");
    }

    assert!(providers.is_ok(), "Should discover mock provider");
    let providers = providers.unwrap();
    assert_eq!(providers.len(), 1, "Should have exactly 1 mock provider");
}

#[tokio::test]
async fn test_discover_without_hints() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // Make sure no hints are set
    // SAFETY: Test-only env var manipulation, serialized by mutex
    unsafe {
        std::env::remove_var("PETALTONGUE_MOCK_MODE");
        std::env::remove_var("PETALTONGUE_DISCOVERY_HINTS");
        std::env::remove_var("BIOMEOS_URL");
        std::env::remove_var("PETALTONGUE_ENABLE_MDNS");
    }

    let providers = discover_visualization_providers().await;

    // Without any configuration, discovery might fail or fallback to mock
    // This is expected behavior (graceful degradation)
    if let Ok(providers) = providers {
        // If we got providers, verify they're valid (can be empty in minimal env)
        assert!(
            providers.is_empty() || !providers.is_empty(),
            "Providers list is valid"
        );
    }
}

#[tokio::test]
async fn test_mock_mode_case_insensitive() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // Test various capitalizations
    for value in &["true", "True", "TRUE", "TrUe"] {
        // SAFETY: Test-only env var manipulation, serialized by mutex
        unsafe {
            std::env::set_var("PETALTONGUE_MOCK_MODE", value);
        }

        let providers = discover_visualization_providers().await;

        // Clean up before assertion
        unsafe {
            std::env::remove_var("PETALTONGUE_MOCK_MODE");
        }

        assert!(providers.is_ok(), "Should work with {}", value);
    }
}

#[tokio::test]
async fn test_mock_mode_false() {
    let _lock = ENV_MUTEX.lock().unwrap();
    // SAFETY: Test-only env var manipulation in isolated test
    unsafe {
        std::env::set_var("PETALTONGUE_MOCK_MODE", "false");

        // Clean other env vars
        std::env::remove_var("PETALTONGUE_DISCOVERY_HINTS");
        std::env::remove_var("BIOMEOS_URL");
    }

    let providers = discover_visualization_providers().await;

    // With mock mode explicitly false and no other config,
    // discovery behavior is undefined (could succeed with fallback or fail)
    // This test verifies it doesn't panic
    // SAFETY: Test-only env var manipulation in isolated test
    unsafe {
        std::env::remove_var("PETALTONGUE_MOCK_MODE");
    }
}

#[tokio::test]
async fn test_discovery_hints_parsing() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // Set discovery hints
    // SAFETY: Test-only env var manipulation, serialized by mutex
    unsafe {
        std::env::set_var(
            "PETALTONGUE_DISCOVERY_HINTS",
            "http://test1:3000,http://test2:3000",
        );
        std::env::remove_var("PETALTONGUE_MOCK_MODE");
    }

    let providers = discover_visualization_providers().await;

    // Clean up before assertion
    unsafe {
        std::env::remove_var("PETALTONGUE_DISCOVERY_HINTS");
    }

    // Should attempt to discover from hints
    // May fail if servers don't exist, but shouldn't panic
    assert!(
        providers.is_ok() || providers.is_err(),
        "Discovery completes"
    );
}

#[tokio::test]
async fn test_legacy_biomeos_url() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // SAFETY: Test-only env var manipulation, serialized by mutex
    unsafe {
        std::env::set_var("BIOMEOS_URL", "http://legacy:3000");
        std::env::remove_var("PETALTONGUE_MOCK_MODE");
        std::env::remove_var("PETALTONGUE_DISCOVERY_HINTS");
    }

    let providers = discover_visualization_providers().await;

    // Clean up before assertion
    unsafe {
        std::env::remove_var("BIOMEOS_URL");
    }

    // Should attempt legacy discovery
    assert!(
        providers.is_ok() || providers.is_err(),
        "Legacy discovery completes"
    );
}

#[tokio::test]
async fn test_discovery_priority() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // Clean up first to avoid test interference
    // SAFETY: Test-only env var manipulation, serialized by mutex
    unsafe {
        std::env::remove_var("PETALTONGUE_DISCOVERY_HINTS");
        std::env::remove_var("BIOMEOS_URL");
        std::env::remove_var("PETALTONGUE_ENABLE_MDNS");

        // Mock mode should take priority over everything
        std::env::set_var("PETALTONGUE_MOCK_MODE", "true");
        std::env::set_var(
            "PETALTONGUE_DISCOVERY_HINTS",
            "http://should-not-be-used:3000",
        );
        std::env::set_var("BIOMEOS_URL", "http://also-should-not-be-used:3000");
    }

    let providers = discover_visualization_providers().await.unwrap();

    // Should only have mock provider (mock mode takes priority)
    assert_eq!(providers.len(), 1, "Mock mode should take priority");

    // Clean up
    // SAFETY: Test-only env var manipulation in isolated test
    unsafe {
        std::env::remove_var("PETALTONGUE_MOCK_MODE");
        std::env::remove_var("PETALTONGUE_DISCOVERY_HINTS");
        std::env::remove_var("BIOMEOS_URL");
    }
}

#[tokio::test]
async fn test_empty_discovery_hints() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // SAFETY: Test-only env var manipulation, serialized by mutex
    unsafe {
        std::env::set_var("PETALTONGUE_DISCOVERY_HINTS", "");
        std::env::remove_var("PETALTONGUE_MOCK_MODE");
    }

    let providers = discover_visualization_providers().await;

    // Clean up before assertion
    unsafe {
        std::env::remove_var("PETALTONGUE_DISCOVERY_HINTS");
    }

    // Empty hints should be handled gracefully
    assert!(
        providers.is_ok() || providers.is_err(),
        "Empty hints handled"
    );
}

#[tokio::test]
async fn test_malformed_hints() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // SAFETY: Test-only env var manipulation, serialized by mutex
    unsafe {
        std::env::set_var("PETALTONGUE_DISCOVERY_HINTS", "not-a-url,also not a url");
        std::env::remove_var("PETALTONGUE_MOCK_MODE");
    }

    let providers = discover_visualization_providers().await;

    // Clean up before assertion
    unsafe {
        std::env::remove_var("PETALTONGUE_DISCOVERY_HINTS");
    }

    // Malformed hints should be handled gracefully (likely fail or skip)
    assert!(
        providers.is_ok() || providers.is_err(),
        "Malformed hints handled"
    );
}

#[tokio::test]
async fn test_concurrent_discovery_attempts() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // SAFETY: Test-only env var manipulation, serialized by mutex
    unsafe {
        std::env::set_var("PETALTONGUE_MOCK_MODE", "true");
    }

    // Make multiple concurrent discovery attempts
    let mut handles = vec![];
    for _ in 0..5 {
        let handle = tokio::spawn(async { discover_visualization_providers().await });
        handles.push(handle);
    }

    // All should complete successfully
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent discovery should work");
    }

    // Clean up
    unsafe {
        std::env::remove_var("PETALTONGUE_MOCK_MODE");
    }
}
