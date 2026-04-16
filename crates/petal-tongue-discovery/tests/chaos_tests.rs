// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Chaos and fault injection tests for petal-tongue-discovery.

use petal_tongue_core::test_fixtures::env_test_helpers;
use petal_tongue_discovery::HealthStatus;
use petal_tongue_discovery::UnixSocketProvider;
use petal_tongue_discovery::cache::ProviderCache;
use petal_tongue_discovery::capability_parse::parse_capabilities_from_response;
use petal_tongue_discovery::concurrent::{
    check_all_providers_health, discover_concurrent, discover_first_available,
};
use petal_tongue_discovery::discover_visualization_providers;
use petal_tongue_discovery::errors::{DiscoveryError, DiscoveryResult};
use petal_tongue_discovery::parse_mdns_response;
use petal_tongue_discovery::{HangHealthCheckProvider, KnownVisualizationProvider};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "test-fixtures")]
use petal_tongue_discovery::DemoVisualizationProvider;

#[tokio::test]
async fn chaos_no_providers_graceful_degradation() {
    env_test_helpers::with_env_vars_async(
        &[
            ("BIOMEOS_URL", None),
            ("PETALTONGUE_DISCOVERY_HINTS", None),
            ("PETALTONGUE_ENABLE_MDNS", Some("false")),
        ],
        || async {
            let providers = discover_visualization_providers().await.unwrap();
            assert!(providers.is_empty());
        },
    )
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn chaos_concurrent_discovery_sources() {
    let empty = || async { Ok::<_, DiscoveryError>(vec![]) };
    let result =
        discover_concurrent(vec![("a", empty), ("b", empty)], Duration::from_secs(2)).await;
    assert!(result.failures.is_empty());
    assert!(result.providers.is_empty());
}

#[cfg(feature = "test-fixtures")]
async fn one_demo_provider() -> DiscoveryResult<Vec<KnownVisualizationProvider>> {
    Ok(vec![KnownVisualizationProvider::Demo(
        DemoVisualizationProvider::new(),
    )])
}

#[cfg(feature = "test-fixtures")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn chaos_concurrent_demo_provider_registration() {
    let run = || one_demo_provider();
    let result = discover_concurrent(
        vec![("demo-a", run), ("demo-b", run)],
        Duration::from_secs(5),
    )
    .await;
    assert_eq!(result.providers.len(), 2);
    assert!(result.failures.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn chaos_cache_invalidation_under_load() {
    let cache: Arc<ProviderCache<Vec<u8>>> = Arc::new(ProviderCache::new(16));
    let mut handles = Vec::new();
    for _ in 0..32 {
        let c = Arc::clone(&cache);
        handles.push(tokio::spawn(async move {
            c.put_primals(vec![1, 2, 3]).await;
            let _ = c.get_primals().await;
            c.invalidate_all().await;
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    cache.invalidate_all().await;
    assert!(cache.get_primals().await.is_none());
}

#[test]
fn chaos_malformed_capability_payloads_filtered() {
    let junk = json!({ "capabilities": [[[]]], "nested": { "x": [null, false, 3.5] } });
    let caps = parse_capabilities_from_response(&junk);
    assert!(caps.is_empty() || caps.iter().all(|s| !s.is_empty()));
    let not_result = json!("definitely not a structured provider payload");
    assert!(parse_capabilities_from_response(&not_result).is_empty());
}

#[test]
fn chaos_dns_garbage_rejected() {
    let addr: SocketAddr = "192.0.2.1:5353".parse().unwrap();
    let garbage: Vec<u8> = (0_u8..=255).cycle().take(512).collect();
    assert!(parse_mdns_response(&garbage, addr).is_err());
    assert!(parse_mdns_response(&[], addr).is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn chaos_rapid_socket_discovery_cycles() {
    for _ in 0..40 {
        let provider = UnixSocketProvider::new();
        let _ = provider.discover().await;
    }
}

#[tokio::test]
async fn chaos_discovery_source_never_finishes_times_out() {
    let result = discover_concurrent(
        vec![("stuck", || async {
            std::future::pending::<DiscoveryResult<Vec<KnownVisualizationProvider>>>().await
        })],
        Duration::from_millis(80),
    )
    .await;
    assert!(result.providers.is_empty());
    assert_eq!(result.failures.len(), 1);
}

#[tokio::test]
async fn chaos_provider_health_never_responds_times_out() {
    let providers = vec![KnownVisualizationProvider::HangHealth(
        HangHealthCheckProvider,
    )];
    let health = check_all_providers_health(&providers, Duration::from_millis(60)).await;
    assert_eq!(health.len(), 1);
    assert!(
        matches!(health[0].status, HealthStatus::Timeout { .. }),
        "expected timeout, got {:?}",
        health[0].status
    );
}

#[tokio::test]
async fn chaos_first_available_all_hang() {
    let providers = vec![
        KnownVisualizationProvider::HangHealth(HangHealthCheckProvider),
        KnownVisualizationProvider::HangHealth(HangHealthCheckProvider),
    ];
    match discover_first_available(providers, Duration::from_millis(50)).await {
        Ok(_) => panic!("expected all providers to fail or time out"),
        Err(e) => assert!(
            e.to_string().contains("failed") || e.to_string().contains("providers"),
            "{e}"
        ),
    }
}
