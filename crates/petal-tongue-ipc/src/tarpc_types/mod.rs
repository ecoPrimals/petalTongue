// SPDX-License-Identifier: AGPL-3.0-or-later
//! # 🚀 tarpc Types and Traits for petalTongue
//!
//! **HIGH-PERFORMANCE PRIMAL-TO-PRIMAL RPC**
//!
//! Provides shared types and service traits for tarpc-based communication.
//! This module defines the interface used by both clients and servers.
//!
//! ## Performance
//! - ~10-20 μs latency (vs 50-100 μs for JSON-RPC)
//! - ~100K requests/sec (vs 10K for JSON-RPC)
//! - Zero-copy binary serialization with bincode
//! - Type-safe at compile time
//!
//! ## Philosophy
//! - tarpc PRIMARY for primal-to-primal communication
//! - JSON-RPC SECONDARY for local IPC and debugging
//! - HTTPS OPTIONAL for external/browser access
//! - Protocol-agnostic architecture
//! - Zero unsafe blocks in this module
//! - Modern idiomatic Rust
//!
//! ## Safety
//! The `#[tarpc::service]` macro generates safe code using the tarpc framework.
//! All serialization is handled by serde with compile-time type checking.
//! No manual memory manipulation or unsafe operations are performed.
//! The generated client/server implementations use only safe Rust abstractions.

mod discovery;
mod health;
mod metrics;
mod render;
mod service;

// Re-export all public types for external consumers (no API change)
pub use discovery::PrimalEndpoint;
pub use health::{HealthStatus, ProtocolInfo, VersionInfo};
pub use metrics::PrimalMetrics;
pub use render::{RenderRequest, RenderResponse};
pub use service::{PetalTongueRpc, PetalTongueRpcClient};

#[cfg(test)]
use petal_tongue_core::constants::{DEFAULT_LOOPBACK_HOST, DEFAULT_TOADSTOOL_PORT, PRIMAL_NAME};

#[cfg(test)]
/// Default tarpc endpoint (loopback:port) for tests and fallbacks.
#[must_use]
fn default_tarpc_endpoint() -> String {
    format!("tarpc://{DEFAULT_LOOPBACK_HOST}:{DEFAULT_TOADSTOOL_PORT}")
}

#[cfg(test)]
/// Default tcp endpoint (loopback:port) for tests and fallbacks.
#[must_use]
fn default_tcp_endpoint() -> String {
    format!("tcp://{DEFAULT_LOOPBACK_HOST}:{DEFAULT_TOADSTOOL_PORT}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use std::collections::HashMap;

    #[test]
    fn test_primal_endpoint_serialization() {
        let endpoint = PrimalEndpoint {
            primal_id: "test-123".to_string(),
            name: Some("Test Primal".to_string()),
            endpoint: default_tarpc_endpoint(),
            capabilities: vec!["visualization".to_string()],
            primal_type: PRIMAL_NAME.to_string(),
            protocol: "tarpc".to_string(),
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&endpoint).expect("serialize");
        let deserialized: PrimalEndpoint = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(endpoint.primal_id, deserialized.primal_id);
        assert_eq!(endpoint.primal_type, deserialized.primal_type);
    }

    #[test]
    fn test_health_status() {
        let health = HealthStatus {
            status: "healthy".to_string(),
            version: "1.2.0".to_string(),
            uptime_seconds: 3600,
            capabilities: vec!["visualization".to_string(), "graph-compute".to_string()],
            details: HashMap::new(),
        };

        assert_eq!(health.status, "healthy");
        assert_eq!(health.capabilities.len(), 2);
    }

    #[test]
    fn test_protocol_info_priority() {
        let tarpc = ProtocolInfo {
            name: "tarpc".to_string(),
            endpoint: default_tarpc_endpoint(),
            enabled: true,
            priority: 1, // PRIMARY
            info: HashMap::new(),
        };

        let jsonrpc = ProtocolInfo {
            name: "jsonrpc".to_string(),
            endpoint: "unix:///tmp/petaltongue.sock".to_string(),
            enabled: true,
            priority: 2, // SECONDARY
            info: HashMap::new(),
        };

        assert!(tarpc.priority < jsonrpc.priority);
        assert_eq!(tarpc.name, "tarpc");
    }

    #[test]
    fn test_render_request() {
        let request = RenderRequest {
            topology: Bytes::from_static(&[1, 2, 3, 4]),
            data: Bytes::new(), // Empty for topology rendering
            width: 1920,
            height: 1080,
            format: "png".to_string(),
            settings: HashMap::new(),
            metadata: None,
        };

        assert_eq!(request.width, 1920);
        assert_eq!(request.format, "png");
    }

    #[test]
    fn test_primal_metrics() {
        let metrics = PrimalMetrics {
            fps: Some(60.0),
            time_since_last_frame: Some(0.016),
            is_hanging: false,
            total_frames: 1000,
            cpu_usage: Some(25.5),
            memory_usage: Some(104_857_600),
            uptime_seconds: 3600,
            custom: HashMap::new(),
        };

        assert_eq!(metrics.fps, Some(60.0));
        assert!(!metrics.is_hanging);
        assert_eq!(metrics.total_frames, 1000);
    }

    #[test]
    fn test_version_info() {
        let version = VersionInfo {
            version: "1.2.0".to_string(),
            tarpc_version: "0.34".to_string(),
            jsonrpc_version: "2.0".to_string(),
            https_version: None, // Not enabled
            capabilities: vec!["visualization".to_string()],
        };

        assert_eq!(version.version, "1.2.0");
        assert!(version.https_version.is_none());
        assert!(!version.capabilities.is_empty());
    }

    #[test]
    fn test_primal_endpoint_serialization_roundtrip() {
        let endpoint = PrimalEndpoint {
            primal_id: "uuid-123".to_string(),
            name: Some(PRIMAL_NAME.to_string()),
            endpoint: default_tarpc_endpoint(),
            capabilities: vec!["visualization".to_string()],
            primal_type: PRIMAL_NAME.to_string(),
            protocol: "tarpc".to_string(),
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&endpoint).expect("serialize");
        let restored: PrimalEndpoint = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.primal_id, endpoint.primal_id);
        assert_eq!(restored.name, endpoint.name);
    }

    #[test]
    fn test_health_status_serialization_roundtrip() {
        let mut details = HashMap::new();
        details.insert("cpu".to_string(), "25%".to_string());
        let status = HealthStatus {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            capabilities: vec!["visualization".to_string()],
            details,
        };
        let json = serde_json::to_string(&status).expect("serialize");
        let restored: HealthStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.status, "healthy");
        assert_eq!(restored.uptime_seconds, 3600);
    }

    #[test]
    fn test_render_response_serialization() {
        let response = RenderResponse {
            success: true,
            data: Bytes::from_static(b"PNG\x89"),
            width: 800,
            height: 600,
            error: None,
            render_time_ms: 50,
        };
        let json = serde_json::to_value(&response).expect("serialize");
        assert_eq!(json["success"], true);
        assert_eq!(json["width"], 800);
        assert_eq!(json["height"], 600);
    }

    #[test]
    fn test_primal_endpoint_display_and_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());
        let endpoint = PrimalEndpoint {
            primal_id: "ep-1".to_string(),
            name: None,
            endpoint: default_tarpc_endpoint(),
            capabilities: vec!["viz".to_string(), "compute".to_string()],
            primal_type: PRIMAL_NAME.to_string(),
            protocol: "tarpc".to_string(),
            metadata: metadata.clone(),
        };
        let json = serde_json::to_string(&endpoint).expect("serialize");
        let restored: PrimalEndpoint = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.metadata.get("version"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_health_status_with_details() {
        let mut details = HashMap::new();
        details.insert("cpu".to_string(), "25%".to_string());
        details.insert("memory".to_string(), "512MB".to_string());
        let status = HealthStatus {
            status: "degraded".to_string(),
            version: "1.0.0".to_string(),
            uptime_seconds: 7200,
            capabilities: vec![],
            details,
        };
        assert_eq!(status.status, "degraded");
        assert_eq!(status.uptime_seconds, 7200);
    }

    #[test]
    fn test_version_info_with_https() {
        let version = VersionInfo {
            version: "2.0.0".to_string(),
            tarpc_version: "0.34".to_string(),
            jsonrpc_version: "2.0".to_string(),
            https_version: Some("1.0".to_string()),
            capabilities: vec!["visualization".to_string(), "https".to_string()],
        };
        let json = serde_json::to_string(&version).expect("serialize");
        let restored: VersionInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.https_version, Some("1.0".to_string()));
    }

    #[test]
    fn test_protocol_info_with_info_map() {
        let mut info = HashMap::new();
        info.insert("latency_us".to_string(), "15".to_string());
        let proto = ProtocolInfo {
            name: "tarpc".to_string(),
            endpoint: default_tarpc_endpoint(),
            enabled: true,
            priority: 1,
            info,
        };
        let json = serde_json::to_string(&proto).expect("serialize");
        let restored: ProtocolInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.info.get("latency_us"), Some(&"15".to_string()));
    }

    #[test]
    fn test_render_request_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "topology".to_string());
        let request = RenderRequest {
            topology: Bytes::new(),
            data: Bytes::from(vec![0u8; 4]),
            width: 100,
            height: 100,
            format: "rgba8".to_string(),
            settings: HashMap::new(),
            metadata: Some(metadata),
        };
        assert_eq!(request.format, "rgba8");
        assert_eq!(request.data.len(), 4);
    }

    #[test]
    fn test_render_response_with_error() {
        let response = RenderResponse {
            success: false,
            data: Bytes::new(),
            width: 0,
            height: 0,
            error: Some("Render failed".to_string()),
            render_time_ms: 0,
        };
        assert!(!response.success);
        assert_eq!(response.error.as_deref(), Some("Render failed"));
    }

    #[test]
    fn test_primal_metrics_hanging() {
        let metrics = PrimalMetrics {
            fps: None,
            time_since_last_frame: Some(6.0),
            is_hanging: true,
            total_frames: 100,
            cpu_usage: None,
            memory_usage: None,
            uptime_seconds: 60,
            custom: HashMap::new(),
        };
        assert!(metrics.is_hanging);
        assert_eq!(metrics.total_frames, 100);
    }

    #[test]
    fn test_primal_metrics_custom() {
        let mut custom = HashMap::new();
        custom.insert("gpu_usage".to_string(), "45%".to_string());
        let metrics = PrimalMetrics {
            fps: Some(60.0),
            time_since_last_frame: None,
            is_hanging: false,
            total_frames: 0,
            cpu_usage: None,
            memory_usage: None,
            uptime_seconds: 0,
            custom,
        };
        assert_eq!(metrics.custom.get("gpu_usage"), Some(&"45%".to_string()));
    }

    #[test]
    fn test_bincode_roundtrip_primal_endpoint() {
        let endpoint = PrimalEndpoint {
            primal_id: "bincode-test".to_string(),
            name: Some("Test".to_string()),
            endpoint: default_tarpc_endpoint(),
            capabilities: vec!["viz".to_string()],
            primal_type: PRIMAL_NAME.to_string(),
            protocol: "tarpc".to_string(),
            metadata: HashMap::new(),
        };
        let encoded = bincode::serialize(&endpoint).expect("bincode serialize");
        let decoded: PrimalEndpoint = bincode::deserialize(&encoded).expect("bincode deserialize");
        assert_eq!(decoded.primal_id, endpoint.primal_id);
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use bytes::Bytes;
    use proptest::prelude::*;
    use std::collections::HashMap;

    fn string_strategy() -> impl Strategy<Value = String> {
        "\\PC{0,50}"
    }

    #[test]
    fn prop_primal_endpoint_bincode_roundtrip() {
        fn prop(
            primal_id: String,
            endpoint: String,
            primal_type: String,
            protocol: String,
        ) -> Result<(), TestCaseError> {
            let ep = PrimalEndpoint {
                primal_id: if primal_id.is_empty() {
                    "id".to_string()
                } else {
                    primal_id
                },
                name: None,
                endpoint: if endpoint.is_empty() {
                    "tcp://0:0".to_string()
                } else {
                    endpoint
                },
                capabilities: vec![],
                primal_type: if primal_type.is_empty() {
                    PRIMAL_NAME.to_string()
                } else {
                    primal_type
                },
                protocol: if protocol.is_empty() {
                    "tarpc".to_string()
                } else {
                    protocol
                },
                metadata: HashMap::new(),
            };
            let enc = bincode::serialize(&ep).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: PrimalEndpoint =
                bincode::deserialize(&enc).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.primal_id, ep.primal_id);
            prop_assert_eq!(dec.endpoint, ep.endpoint);
            Ok(())
        }
        proptest!(|(primal_id in string_strategy(), endpoint in string_strategy(), primal_type in string_strategy(), protocol in string_strategy())| prop(primal_id, endpoint, primal_type, protocol)?);
    }

    #[test]
    fn prop_primal_endpoint_serde_json_roundtrip() {
        fn prop(primal_id: String, endpoint: String) -> Result<(), TestCaseError> {
            let ep = PrimalEndpoint {
                primal_id: if primal_id.is_empty() {
                    "id".to_string()
                } else {
                    primal_id
                },
                name: None,
                endpoint: if endpoint.is_empty() {
                    "tcp://0:0".to_string()
                } else {
                    endpoint
                },
                capabilities: vec![],
                primal_type: PRIMAL_NAME.to_string(),
                protocol: "tarpc".to_string(),
                metadata: HashMap::new(),
            };
            let json =
                serde_json::to_string(&ep).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: PrimalEndpoint =
                serde_json::from_str(&json).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.primal_id, ep.primal_id);
            Ok(())
        }
        proptest!(|(primal_id in string_strategy(), endpoint in string_strategy())| prop(primal_id, endpoint)?);
    }

    #[test]
    fn prop_health_status_bincode_roundtrip() {
        fn prop(status: String, version: String, uptime: u64) -> Result<(), TestCaseError> {
            let h = HealthStatus {
                status: if status.is_empty() {
                    "ok".to_string()
                } else {
                    status
                },
                version: if version.is_empty() {
                    "1.0".to_string()
                } else {
                    version
                },
                uptime_seconds: uptime,
                capabilities: vec![],
                details: HashMap::new(),
            };
            let enc = bincode::serialize(&h).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: HealthStatus =
                bincode::deserialize(&enc).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.status, h.status);
            prop_assert_eq!(dec.uptime_seconds, h.uptime_seconds);
            Ok(())
        }
        proptest!(|(status in string_strategy(), version in string_strategy(), uptime in 0u64..1_000_000u64)| prop(status, version, uptime)?);
    }

    #[test]
    fn prop_health_status_serde_json_roundtrip() {
        fn prop(status: String, uptime: u64) -> Result<(), TestCaseError> {
            let h = HealthStatus {
                status: if status.is_empty() {
                    "ok".to_string()
                } else {
                    status
                },
                version: "1.0".to_string(),
                uptime_seconds: uptime,
                capabilities: vec![],
                details: HashMap::new(),
            };
            let json = serde_json::to_string(&h).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: HealthStatus =
                serde_json::from_str(&json).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.status, h.status);
            prop_assert_eq!(dec.uptime_seconds, h.uptime_seconds);
            Ok(())
        }
        proptest!(|(status in string_strategy(), uptime in 0u64..1_000_000u64)| prop(status, uptime)?);
    }

    #[test]
    fn prop_version_info_bincode_roundtrip() {
        fn prop(version: String) -> Result<(), TestCaseError> {
            let v = VersionInfo {
                version: if version.is_empty() {
                    "1.0".to_string()
                } else {
                    version
                },
                tarpc_version: "0.34".to_string(),
                jsonrpc_version: "2.0".to_string(),
                https_version: None,
                capabilities: vec![],
            };
            let enc = bincode::serialize(&v).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: VersionInfo =
                bincode::deserialize(&enc).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.version, v.version);
            Ok(())
        }
        proptest!(|(version in string_strategy())| prop(version)?);
    }

    #[test]
    fn prop_version_info_serde_json_roundtrip() {
        fn prop(version: String) -> Result<(), TestCaseError> {
            let v = VersionInfo {
                version: if version.is_empty() {
                    "1.0".to_string()
                } else {
                    version
                },
                tarpc_version: "0.34".to_string(),
                jsonrpc_version: "2.0".to_string(),
                https_version: None,
                capabilities: vec![],
            };
            let json = serde_json::to_string(&v).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: VersionInfo =
                serde_json::from_str(&json).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.version, v.version);
            Ok(())
        }
        proptest!(|(version in string_strategy())| prop(version)?);
    }

    #[test]
    fn prop_protocol_info_bincode_roundtrip() {
        fn prop(name: String, endpoint: String, priority: u8) -> Result<(), TestCaseError> {
            let p = ProtocolInfo {
                name: if name.is_empty() {
                    "tarpc".to_string()
                } else {
                    name
                },
                endpoint: if endpoint.is_empty() {
                    "tcp://0:0".to_string()
                } else {
                    endpoint
                },
                enabled: true,
                priority,
                info: HashMap::new(),
            };
            let enc = bincode::serialize(&p).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: ProtocolInfo =
                bincode::deserialize(&enc).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.name, p.name);
            prop_assert_eq!(dec.priority, p.priority);
            Ok(())
        }
        proptest!(|(name in string_strategy(), endpoint in string_strategy(), priority in any::<u8>())| prop(name, endpoint, priority)?);
    }

    #[test]
    fn prop_protocol_info_serde_json_roundtrip() {
        fn prop(name: String) -> Result<(), TestCaseError> {
            let p = ProtocolInfo {
                name: if name.is_empty() {
                    "tarpc".to_string()
                } else {
                    name
                },
                endpoint: default_tcp_endpoint(),
                enabled: true,
                priority: 1,
                info: HashMap::new(),
            };
            let json = serde_json::to_string(&p).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: ProtocolInfo =
                serde_json::from_str(&json).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.name, p.name);
            Ok(())
        }
        proptest!(|(name in string_strategy())| prop(name)?);
    }

    #[test]
    fn prop_render_request_bincode_roundtrip() {
        fn prop(width: u32, height: u32, format: String) -> Result<(), TestCaseError> {
            let r = RenderRequest {
                topology: Bytes::new(),
                data: Bytes::new(),
                width,
                height,
                format: if format.is_empty() {
                    "png".to_string()
                } else {
                    format
                },
                settings: HashMap::new(),
                metadata: None,
            };
            let enc = bincode::serialize(&r).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: RenderRequest =
                bincode::deserialize(&enc).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.width, r.width);
            prop_assert_eq!(dec.height, r.height);
            Ok(())
        }
        proptest!(|(width in 0u32..4096u32, height in 0u32..4096u32, format in string_strategy())| prop(width, height, format)?);
    }

    #[test]
    fn prop_render_request_serde_json_roundtrip() {
        fn prop(width: u32, height: u32) -> Result<(), TestCaseError> {
            let r = RenderRequest {
                topology: Bytes::new(),
                data: Bytes::new(),
                width,
                height,
                format: "png".to_string(),
                settings: HashMap::new(),
                metadata: None,
            };
            let json = serde_json::to_string(&r).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: RenderRequest =
                serde_json::from_str(&json).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.width, r.width);
            prop_assert_eq!(dec.height, r.height);
            Ok(())
        }
        proptest!(|(width in 0u32..4096u32, height in 0u32..4096u32)| prop(width, height)?);
    }

    #[test]
    fn prop_render_response_bincode_roundtrip() {
        fn prop(width: u32, height: u32, success: bool) -> Result<(), TestCaseError> {
            let r = RenderResponse {
                success,
                data: Bytes::new(),
                width,
                height,
                error: None,
                render_time_ms: 0,
            };
            let enc = bincode::serialize(&r).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: RenderResponse =
                bincode::deserialize(&enc).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.width, r.width);
            prop_assert_eq!(dec.success, r.success);
            Ok(())
        }
        proptest!(|(width in 0u32..4096u32, height in 0u32..4096u32, success in any::<bool>())| prop(width, height, success)?);
    }

    #[test]
    fn prop_render_response_serde_json_roundtrip() {
        fn prop(success: bool) -> Result<(), TestCaseError> {
            let r = RenderResponse {
                success,
                data: Bytes::new(),
                width: 100,
                height: 100,
                error: None,
                render_time_ms: 50,
            };
            let json = serde_json::to_string(&r).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: RenderResponse =
                serde_json::from_str(&json).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.success, r.success);
            Ok(())
        }
        proptest!(|(success in any::<bool>())| prop(success)?);
    }

    #[test]
    fn prop_primal_metrics_bincode_roundtrip() {
        fn prop(total_frames: u64, uptime: u64, is_hanging: bool) -> Result<(), TestCaseError> {
            let m = PrimalMetrics {
                fps: None,
                time_since_last_frame: None,
                is_hanging,
                total_frames,
                cpu_usage: None,
                memory_usage: None,
                uptime_seconds: uptime,
                custom: HashMap::new(),
            };
            let enc = bincode::serialize(&m).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: PrimalMetrics =
                bincode::deserialize(&enc).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.total_frames, m.total_frames);
            prop_assert_eq!(dec.is_hanging, m.is_hanging);
            Ok(())
        }
        proptest!(|(total_frames in 0u64..1_000_000u64, uptime in 0u64..1_000_000u64, is_hanging in any::<bool>())| prop(total_frames, uptime, is_hanging)?);
    }

    #[test]
    fn prop_primal_metrics_serde_json_roundtrip() {
        fn prop(total_frames: u64) -> Result<(), TestCaseError> {
            let m = PrimalMetrics {
                fps: Some(60.0),
                time_since_last_frame: None,
                is_hanging: false,
                total_frames,
                cpu_usage: None,
                memory_usage: None,
                uptime_seconds: 3600,
                custom: HashMap::new(),
            };
            let json = serde_json::to_string(&m).map_err(|_| TestCaseError::reject("serialize"))?;
            let dec: PrimalMetrics =
                serde_json::from_str(&json).map_err(|_| TestCaseError::reject("deserialize"))?;
            prop_assert_eq!(dec.total_frames, m.total_frames);
            Ok(())
        }
        proptest!(|(total_frames in 0u64..1_000_000u64)| prop(total_frames)?);
    }
}
