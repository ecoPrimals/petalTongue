// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unit tests for biomeOS provider and JSON-RPC helpers.

#[allow(clippy::unwrap_used, clippy::expect_used)]
mod provider_tests {
    use super::super::*;

    #[test]
    fn provider_cache_default() {
        let cache = ProviderCache::default();
        assert!(cache.devices.is_empty());
        assert!(cache.primals.is_empty());
        assert!(cache.niche_templates.is_empty());
        assert!(cache.last_update.is_none());
    }

    #[test]
    fn provider_new_for_test() {
        let provider = BiomeOSProvider::new_for_test("/tmp/test.sock");
        assert_eq!(provider.endpoint(), "/tmp/test.sock");
    }

    #[test]
    fn derive_websocket_endpoint_format() {
        let provider = BiomeOSProvider::new_for_test("dummy");
        let ep = provider.derive_websocket_endpoint_for_test();
        assert!(ep.starts_with("ws://"));
        assert!(ep.ends_with("/events"));
        assert!(
            ep.contains("127.0.0.1") || ep.contains("localhost"),
            "WebSocket URL should use loopback host, got: {ep}"
        );
    }

    #[test]
    fn provider_cache_clone() {
        let cache = ProviderCache::default();
        let cloned = cache;
        assert!(cloned.devices.is_empty());
        assert!(cloned.primals.is_empty());
    }

    #[test]
    fn assign_device_params_structure() {
        let params = serde_json::json!({
            "device_id": "dev-1",
            "primal_id": "primal-1"
        });
        assert_eq!(params["device_id"], "dev-1");
        assert_eq!(params["primal_id"], "primal-1");
    }

    #[test]
    fn deploy_niche_params_structure() {
        let params = serde_json::json!({
            "name": "test-niche",
            "description": "A test",
            "required_primals": ["p1"],
            "optional_primals": [],
            "metadata": {}
        });
        assert_eq!(params["name"], "test-niche");
        assert!(params["required_primals"].is_array());
    }

    #[test]
    fn jsonrpc_request_structure() {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "device.list",
            "params": {},
            "id": 1
        });
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "device.list");
    }

    #[test]
    fn subscribe_events_params_structure() {
        let params = serde_json::json!({
            "events": ["device.added", "device.removed", "primal.status", "niche.deployed"]
        });
        let arr = params["events"].as_array().expect("array");
        assert!(arr.contains(&serde_json::json!("device.added")));
    }

    #[test]
    fn health_check_jsonrpc_params() {
        let params = serde_json::json!({});
        assert!(params.is_object());
    }

    #[test]
    fn build_jsonrpc_request_structure() {
        let req = build_jsonrpc_request("device.list", serde_json::json!({}), 1);
        assert_eq!(req["jsonrpc"], "2.0");
        assert_eq!(req["method"], "device.list");
        assert_eq!(req["id"], 1);
    }

    #[test]
    fn parse_jsonrpc_result_present() {
        let res = serde_json::json!({"result": {"ok": true}, "id": 1});
        let r = parse_jsonrpc_result(&res);
        assert!(r.is_some());
        assert_eq!(r.unwrap()["ok"], true);
    }

    #[test]
    fn parse_jsonrpc_result_absent() {
        let res = serde_json::json!({"id": 1});
        assert!(parse_jsonrpc_result(&res).is_none());
    }

    #[test]
    fn parse_jsonrpc_error_present() {
        let res = serde_json::json!({"error": {"code": -32600, "message": "Invalid"}});
        assert!(parse_jsonrpc_error(&res).is_some());
    }

    #[test]
    fn test_health_response_status() {
        let v = serde_json::json!({"status": "ok"});
        assert_eq!(health_response_status(&v), "ok");
    }

    #[test]
    fn test_health_response_status_unknown() {
        let v = serde_json::json!({});
        assert_eq!(health_response_status(&v), "unknown");
    }

    #[test]
    fn test_health_response_healthy() {
        let v = serde_json::json!({"healthy": true});
        assert_eq!(health_response_healthy(&v), Some(true));
    }

    #[test]
    fn test_build_assign_device_params() {
        let params = build_assign_device_params("dev-1", "primal-1");
        assert_eq!(params["device_id"], "dev-1");
        assert_eq!(params["primal_id"], "primal-1");
    }

    #[test]
    fn test_build_deploy_niche_params() {
        use crate::biomeos_integration::NicheTemplate;
        let niche = NicheTemplate {
            id: "id1".to_string(),
            name: "test".to_string(),
            description: "desc".to_string(),
            required_primals: vec!["p1".to_string()],
            optional_primals: vec![],
            metadata: serde_json::json!({}),
        };
        let params = build_deploy_niche_params(&niche);
        assert_eq!(params["name"], "test");
        assert!(params["required_primals"].is_array());
    }

    #[test]
    fn test_extract_niche_id_from_response() {
        let v = serde_json::json!({"niche_id": "niche-123"});
        let id = extract_niche_id_from_response(&v);
        assert!(id.is_some());
        assert_eq!(id.unwrap(), "niche-123");
    }

    #[test]
    fn test_build_subscribe_events_params() {
        let params = build_subscribe_events_params();
        let arr = params["events"].as_array().expect("array");
        assert!(arr.contains(&serde_json::json!("device.added")));
    }

    #[test]
    fn test_extract_niche_id_absent() {
        let v = serde_json::json!({});
        assert!(extract_niche_id_from_response(&v).is_none());

        let v = serde_json::json!({"other": "value"});
        assert!(extract_niche_id_from_response(&v).is_none());
    }

    #[test]
    fn test_health_response_healthy_false() {
        let v = serde_json::json!({"healthy": false});
        assert_eq!(health_response_healthy(&v), Some(false));
    }

    #[test]
    fn test_health_response_healthy_absent() {
        let v = serde_json::json!({});
        assert!(health_response_healthy(&v).is_none());
    }

    #[test]
    fn test_health_response_status_from_str() {
        let v = serde_json::json!({"status": "degraded"});
        assert_eq!(health_response_status(&v), "degraded");
    }

    #[test]
    fn test_build_jsonrpc_request_with_params() {
        let req = build_jsonrpc_request(
            "device.assign",
            serde_json::json!({"device_id": "d1", "primal_id": "p1"}),
            42,
        );
        assert_eq!(req["method"], "device.assign");
        assert_eq!(req["id"], 42);
        assert_eq!(req["params"]["device_id"], "d1");
    }

    #[test]
    fn test_parse_jsonrpc_error_absent() {
        let res = serde_json::json!({"result": true, "id": 1});
        assert!(parse_jsonrpc_error(&res).is_none());
    }

    #[test]
    fn test_provider_cache_last_update() {
        let mut cache = ProviderCache::default();
        assert!(cache.last_update.is_none());
        cache.last_update = Some(std::time::Instant::now());
        assert!(cache.last_update.is_some());
    }

    #[test]
    fn test_health_response_status_non_string() {
        let v = serde_json::json!({"status": 123});
        assert_eq!(health_response_status(&v), "unknown");
        let v = serde_json::json!({"status": null});
        assert_eq!(health_response_status(&v), "unknown");
        let v = serde_json::json!({"status": true});
        assert_eq!(health_response_status(&v), "unknown");
    }

    #[test]
    fn test_health_response_healthy_non_bool() {
        let v = serde_json::json!({"healthy": "true"});
        assert!(health_response_healthy(&v).is_none());
        let v = serde_json::json!({"healthy": 1});
        assert!(health_response_healthy(&v).is_none());
    }

    #[test]
    fn test_provider_cache_with_data() {
        use crate::biomeos_integration::{
            Device, DeviceStatus, DeviceType, Health, NicheTemplate, Primal,
        };
        let mut cache = ProviderCache::default();
        cache.devices.push(Device {
            id: "d1".to_string(),
            name: "Device 1".to_string(),
            device_type: DeviceType::CPU,
            status: DeviceStatus::Online,
            resource_usage: 0.5,
            assigned_to: None,
            metadata: serde_json::json!({}),
        });
        cache.primals.push(Primal {
            id: "p1".to_string(),
            name: "Primal 1".to_string(),
            health: Health::Healthy,
            capabilities: vec!["compute".to_string()],
            load: 0.0,
            assigned_devices: vec![],
            metadata: serde_json::json!({}),
        });
        cache.niche_templates.push(NicheTemplate {
            id: "n1".to_string(),
            name: "Niche 1".to_string(),
            description: "Desc".to_string(),
            required_primals: vec![],
            optional_primals: vec![],
            metadata: serde_json::json!({}),
        });
        assert_eq!(cache.devices.len(), 1);
        assert_eq!(cache.primals.len(), 1);
        assert_eq!(cache.niche_templates.len(), 1);
    }

    #[test]
    fn test_build_jsonrpc_request_empty_params() {
        let req = build_jsonrpc_request("health.ping", serde_json::json!(null), 0);
        assert_eq!(req["params"], serde_json::Value::Null);
        assert_eq!(req["id"], 0);
    }

    #[test]
    fn test_parse_jsonrpc_result_array() {
        let res = serde_json::json!({"result": [1, 2, 3], "id": 1});
        let binding = parse_jsonrpc_result(&res);
        let arr = binding.as_ref().and_then(|v| v.as_array()).expect("array");
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_parse_jsonrpc_result_string() {
        let res = serde_json::json!({"result": "ok", "id": 1});
        let r = parse_jsonrpc_result(&res);
        assert_eq!(r.as_ref().and_then(|v| v.as_str()), Some("ok"));
    }

    #[test]
    fn test_parse_jsonrpc_error_object() {
        let res = serde_json::json!({"error": {"code": -32600, "message": "Invalid Request"}});
        let err = parse_jsonrpc_error(&res);
        assert!(err.is_some());
        let obj = err.unwrap();
        assert_eq!(
            obj.get("code").and_then(serde_json::Value::as_i64),
            Some(-32600)
        );
    }

    #[test]
    fn test_parse_jsonrpc_error_string() {
        let res = serde_json::json!({"error": "Connection refused"});
        let err = parse_jsonrpc_error(&res);
        assert!(err.is_some());
    }

    #[test]
    fn test_extract_niche_id_from_response_value_types() {
        let v = serde_json::json!({"niche_id": 12345});
        let id = extract_niche_id_from_response(&v);
        assert!(id.is_some());
    }

    #[test]
    fn test_build_deploy_niche_params_full() {
        use crate::biomeos_integration::NicheTemplate;
        let niche = NicheTemplate {
            id: "n1".to_string(),
            name: "Full Niche".to_string(),
            description: "Full description".to_string(),
            required_primals: vec!["p1".to_string(), "p2".to_string()],
            optional_primals: vec!["p3".to_string()],
            metadata: serde_json::json!({"key": "value"}),
        };
        let params = build_deploy_niche_params(&niche);
        assert_eq!(params["name"], "Full Niche");
        assert_eq!(params["description"], "Full description");
        assert_eq!(params["required_primals"].as_array().unwrap().len(), 2);
        assert_eq!(params["optional_primals"].as_array().unwrap().len(), 1);
        assert_eq!(params["metadata"]["key"], "value");
    }

    #[test]
    fn test_provider_cache_debug() {
        let cache = ProviderCache::default();
        let dbg = format!("{:?}", cache);
        assert!(dbg.contains("ProviderCache"));
    }

    #[test]
    fn test_build_assign_device_params_empty_strings() {
        let params = build_assign_device_params("", "");
        assert_eq!(params["device_id"], "");
        assert_eq!(params["primal_id"], "");
    }

    #[test]
    fn test_health_response_status_array_status() {
        let v = serde_json::json!({"status": ["a", "b"]});
        assert_eq!(health_response_status(&v), "unknown");
    }

    #[test]
    fn test_health_response_healthy_object() {
        let v = serde_json::json!({"healthy": {}});
        assert!(health_response_healthy(&v).is_none());
    }

    #[test]
    fn test_build_subscribe_events_params_structure() {
        let params = build_subscribe_events_params();
        let events = params["events"].as_array().expect("array");
        assert_eq!(events.len(), 4);
        assert!(events.contains(&serde_json::json!("device.removed")));
        assert!(events.contains(&serde_json::json!("primal.status")));
        assert!(events.contains(&serde_json::json!("niche.deployed")));
    }
}
