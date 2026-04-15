// SPDX-License-Identifier: AGPL-3.0-or-later
//! Types for `visualization.render.stream` and stream backpressure.

use petal_tongue_core::DataBinding;
use serde::{Deserialize, Serialize};

/// Request payload for visualization.render.stream (incremental update)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUpdateRequest {
    /// Session ID to update (must match an existing render session)
    pub session_id: String,
    /// Which binding to update (by id)
    pub binding_id: String,
    /// The update operation
    pub operation: StreamOperation,
}

/// Types of incremental stream updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamOperation {
    /// Append new data points to a `TimeSeries` or Spectrum
    #[serde(rename = "append")]
    Append {
        /// X-axis values (timestamps for `TimeSeries`, frequencies for Spectrum)
        x_values: Vec<f64>,
        /// Y-axis values (measurements for `TimeSeries`, amplitudes for Spectrum)
        y_values: Vec<f64>,
    },
    /// Replace the current value of a Gauge
    #[serde(rename = "set_value")]
    SetValue {
        /// New gauge value
        value: f64,
    },
    /// Replace the full binding (for Heatmap, `FieldMap`, etc.)
    #[serde(rename = "replace")]
    Replace {
        /// Replacement binding
        binding: DataBinding,
    },
}

/// Response for visualization.render.stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUpdateResponse {
    /// Session ID (echoed back)
    pub session_id: String,
    /// Binding ID that was updated
    pub binding_id: String,
    /// Whether the update was accepted
    pub accepted: bool,
    /// Whether the server is experiencing backpressure (springs should throttle)
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub backpressure_active: bool,
}

/// Server-side backpressure configuration for stream rate limiting.
///
/// Matches the client-side `BackpressureConfig` from wetSpring/healthSpring.
/// When a session receives updates faster than the budget allows, the server
/// signals `backpressure_active: true` so springs can throttle.
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// Maximum updates per second per session before entering backpressure.
    pub max_updates_per_sec: u32,
    /// Cooldown duration after entering backpressure state.
    pub cooldown: std::time::Duration,
    /// Consecutive fast updates before activating backpressure.
    pub burst_tolerance: u32,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_updates_per_sec: 120,
            cooldown: std::time::Duration::from_millis(200),
            burst_tolerance: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_operation_append_roundtrip() {
        let op = StreamOperation::Append {
            x_values: vec![1.0, 2.0],
            y_values: vec![10.0, 20.0],
        };
        let json = serde_json::to_string(&op).expect("serialize");
        let restored: StreamOperation = serde_json::from_str(&json).expect("deserialize");
        match restored {
            StreamOperation::Append { x_values, y_values } => {
                assert_eq!(x_values, vec![1.0, 2.0]);
                assert_eq!(y_values, vec![10.0, 20.0]);
            }
            _ => panic!("expected Append"),
        }
    }

    #[test]
    fn stream_operation_set_value_roundtrip() {
        let op = StreamOperation::SetValue { value: 42.5 };
        let json = serde_json::to_string(&op).expect("serialize");
        let restored: StreamOperation = serde_json::from_str(&json).expect("deserialize");
        match restored {
            StreamOperation::SetValue { value } => assert_eq!(value, 42.5),
            _ => panic!("expected SetValue"),
        }
    }

    #[test]
    fn stream_update_request_roundtrip() {
        let req = StreamUpdateRequest {
            session_id: "s1".into(),
            binding_id: "b1".into(),
            operation: StreamOperation::SetValue { value: 1.0 },
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let restored: StreamUpdateRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.session_id, "s1");
    }

    #[test]
    fn stream_update_response_backpressure_serialization() {
        let resp = StreamUpdateResponse {
            session_id: "s1".into(),
            binding_id: "b1".into(),
            accepted: true,
            backpressure_active: true,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("backpressure_active"));
    }

    #[test]
    fn backpressure_config_default() {
        let config = BackpressureConfig::default();
        assert_eq!(config.max_updates_per_sec, 120);
        assert_eq!(config.cooldown, std::time::Duration::from_millis(200));
        assert_eq!(config.burst_tolerance, 10);
    }
}
