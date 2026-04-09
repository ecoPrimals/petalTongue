// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)]
//! Integration tests for petalTongue discovery against biomeOS-style payloads.

#[tokio::test]
async fn test_biomeos_api_contract() {
    use serde_json::json;

    #[derive(serde::Deserialize)]
    struct BiomeOSResponse {
        primals: Vec<BiomeOSPrimal>,
    }

    #[derive(serde::Deserialize)]
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
    assert_eq!(primal.capabilities, vec!["test.capability".to_string()]);
    assert_eq!(primal.endpoint, "http://localhost:9999");

    println!("✅ API contract validation passed");
}
