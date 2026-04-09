// SPDX-License-Identifier: AGPL-3.0-or-later

use super::test_support::{PRIMAL_NAME, default_tcp_endpoint};
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
        let json = serde_json::to_string(&ep).map_err(|_| TestCaseError::reject("serialize"))?;
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
