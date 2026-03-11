// SPDX-License-Identifier: AGPL-3.0-only
// Integration test for petalTongue with live biomeOS API
//
// This test validates that petalTongue can discover primals from a running biomeOS API server

#[cfg(feature = "legacy-http")]
use petal_tongue_discovery::{
    HttpVisualizationProvider, VisualizationDataProvider, discover_visualization_providers,
};

#[tokio::test]
#[ignore = "Only run when biomeOS API is actually running"]
#[cfg(feature = "legacy-http")]
async fn test_live_biomeos_integration() {
    // This test expects biomeOS API running on localhost:3000
    let biomeos_url =
        std::env::var("BIOMEOS_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    println!("Testing against biomeOS API at: {biomeos_url}");

    // Create HTTP provider for biomeOS
    let provider = HttpVisualizationProvider::new(&biomeos_url).unwrap();

    // Test 1: Health check
    println!("\n1. Testing health endpoint...");
    let health_result = provider.health_check().await;
    assert!(
        health_result.is_ok(),
        "Health check should succeed: {:?}",
        health_result.err()
    );
    println!("   ✅ Health check passed: {}", health_result.unwrap());

    // Test 2: Discover primals
    println!("\n2. Testing primal discovery...");
    let primals_result = provider.get_primals().await;
    assert!(
        primals_result.is_ok(),
        "Primal discovery should succeed: {:?}",
        primals_result.err()
    );

    let primals = primals_result.unwrap();
    println!("   ✅ Discovered {} primals", primals.len());

    assert!(!primals.is_empty(), "Should discover at least one primal");

    for primal in &primals {
        println!(
            "     - {}: {} ({})",
            primal.id, primal.name, primal.primal_type
        );
        println!("       Endpoint: {}", primal.endpoint);
        println!("       Capabilities: {:?}", primal.capabilities);
        println!("       Last seen: {}", primal.last_seen);

        // Validate required fields
        assert!(
            !primal.id.as_str().is_empty(),
            "Primal ID should not be empty"
        );
        assert!(!primal.name.is_empty(), "Primal name should not be empty");
        assert!(
            !primal.endpoint.is_empty(),
            "Primal endpoint should not be empty"
        );
        assert!(primal.last_seen > 0, "Last seen timestamp should be set");
    }

    // Test 3: Get topology
    println!("\n3. Testing topology...");
    let topology_result = provider.get_topology().await;
    assert!(
        topology_result.is_ok(),
        "Topology query should succeed: {:?}",
        topology_result.err()
    );

    let edges = topology_result.unwrap();
    println!("   ✅ Retrieved {} topology edges", edges.len());

    for edge in &edges {
        println!("     - {} → {} ({})", edge.from, edge.to, edge.edge_type);
    }

    // Test 4: Test auto-discovery flow
    println!("\n4. Testing auto-discovery flow...");
    let providers = petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars_async(
        &[
            ("PETALTONGUE_ENABLE_MDNS", Some("false")),
            ("BIOMEOS_URL", Some(biomeos_url.as_str())),
        ],
        || async { discover_visualization_providers().await },
    )
    .await;
    assert!(
        providers.is_ok(),
        "Auto-discovery should succeed: {:?}",
        providers.err()
    );

    let providers = providers.unwrap();
    println!("   ✅ Auto-discovery found {} provider(s)", providers.len());
    assert!(!providers.is_empty(), "Should find at least one provider");

    // Verify we can query through discovered provider
    let discovered_primals = providers[0].get_primals().await;
    assert!(
        discovered_primals.is_ok(),
        "Query through discovered provider should work"
    );
    println!(
        "   ✅ Queried through discovered provider: {} primals",
        discovered_primals.unwrap().len()
    );

    println!("\n✅ All integration tests passed!");
}

#[tokio::test]
async fn test_biomeos_api_contract() {
    use serde_json::json;

    #[derive(serde::Deserialize)]
    struct BiomeOSResponse {
        primals: Vec<BiomeOSPrimal>,
    }

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct BiomeOSPrimal {
        id: String,
        name: String,
        primal_type: String,
        health: String,
        capabilities: Vec<String>,
        endpoint: String,
        last_seen: u64,
    }

    let response = json!({
        "primals": [
            {
                "id": "test-primal",
                "name": "Test Primal",
                "primal_type": "test",
                "version": "1.0.0",
                "health": "healthy",
                "capabilities": ["test.capability"],
                "endpoint": "http://localhost:9999",
                "last_seen": 1_234_567_890u64,
                "trust_level": 2,
                "family_id": "test-family",
                "allowed_capabilities": ["*"],
                "denied_capabilities": []
            }
        ],
        "count": 1,
        "mode": "test"
    });

    let parsed: BiomeOSResponse = serde_json::from_value(response).unwrap();
    assert_eq!(parsed.primals.len(), 1);

    let primal = &parsed.primals[0];
    assert_eq!(primal.id, "test-primal");
    assert_eq!(primal.name, "Test Primal");
    assert_eq!(primal.primal_type, "test");
    assert_eq!(primal.health, "healthy");
    assert_eq!(primal.last_seen, 1_234_567_890);

    println!("✅ API contract validation passed");
}
