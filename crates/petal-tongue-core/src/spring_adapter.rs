// SPDX-License-Identifier: AGPL-3.0-only
//! Universal spring data adapter for heterogeneous push formats.
//!
//! Springs push visualization data via JSON-RPC using different envelope formats:
//!
//! - **neuralSpring / healthSpring / wetSpring**: `{ "bindings": [ { "channel_type": "...", ... } ] }`
//! - **ludoSpring**: `{ "data": { ... }, "channel": "..." }` with `GameChannelType` semantics
//! - **ecoPrimals/time-series/v1**: `{ "schema": "ecoPrimals/time-series/v1", "series": [ ... ] }`
//!
//! `SpringDataAdapter` normalizes all three formats to `Vec<DataBinding>`.

use crate::data_channel::DataBinding;
use serde::{Deserialize, Serialize};

/// Recognized envelope formats from springs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpringPayloadFormat {
    /// Standard `{ "bindings": [...] }` (neuralSpring, healthSpring, wetSpring).
    Bindings,
    /// ludoSpring `{ "data": {...}, "channel": "..." }`.
    GameChannel,
    /// `ecoPrimals/time-series/v1` schema.
    EcoTimeSeries,
    /// Already a raw `DataBinding` array (pass-through).
    Raw,
}

/// ludoSpring game channel types that need mapping to `DataBinding`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameChannelType {
    /// Player engagement over time (maps to `TimeSeries`).
    EngagementCurve,
    /// Difficulty curve across game sections (maps to `TimeSeries`).
    DifficultyProfile,
    /// Flow-state timeline (maps to `Bar`).
    FlowTimeline,
    /// UI heuristic metrics (maps to `Bar`).
    UiAnalysis,
    /// Fitts/Hick/Steering cost heatmap (maps to `Heatmap`).
    InteractionCostMap,
    /// Procedural generation preview (maps to `Scatter`).
    GenerationPreview,
    /// Accessibility audit matrix (maps to `Heatmap`).
    AccessibilityReport,
}

/// ludoSpring-style payload: `{ "data": {...}, "channel": "..." }`.
#[derive(Debug, Clone, Deserialize)]
struct GameChannelPayload {
    channel: GameChannelType,
    data: serde_json::Value,
}

/// Standard bindings envelope: `{ "bindings": [...] }`.
#[derive(Debug, Clone, Deserialize)]
struct BindingsEnvelope {
    bindings: Vec<DataBinding>,
}

/// `ecoPrimals/time-series/v1` envelope.
#[derive(Debug, Clone, Deserialize)]
struct EcoTimeSeriesEnvelope {
    #[expect(dead_code, reason = "used for format detection only")]
    schema: String,
    series: Vec<EcoTimeSeriesEntry>,
}

/// Single entry in `ecoPrimals/time-series/v1`.
#[derive(Debug, Clone, Deserialize)]
struct EcoTimeSeriesEntry {
    id: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    unit: String,
    #[serde(default)]
    x_label: String,
    #[serde(default)]
    y_label: String,
    timestamps: Vec<f64>,
    values: Vec<f64>,
}

/// Universal spring data adapter.
///
/// Accepts any spring push payload and normalizes it to `Vec<DataBinding>`.
pub struct SpringDataAdapter;

impl SpringDataAdapter {
    /// Detect the format of a JSON payload.
    #[must_use]
    pub fn detect_format(value: &serde_json::Value) -> SpringPayloadFormat {
        if let Some(obj) = value.as_object() {
            if obj.get("schema").and_then(|s| s.as_str()) == Some("ecoPrimals/time-series/v1") {
                return SpringPayloadFormat::EcoTimeSeries;
            }
            if obj.contains_key("bindings") {
                return SpringPayloadFormat::Bindings;
            }
            if obj.contains_key("data") && obj.contains_key("channel") {
                return SpringPayloadFormat::GameChannel;
            }
        }
        if value.is_array() {
            return SpringPayloadFormat::Raw;
        }
        SpringPayloadFormat::Raw
    }

    /// Adapt a raw JSON value to a `Vec<DataBinding>`, auto-detecting the format.
    ///
    /// # Errors
    ///
    /// Returns an error if the payload cannot be parsed into any recognized format.
    pub fn adapt(value: &serde_json::Value) -> Result<Vec<DataBinding>, SpringAdapterError> {
        match Self::detect_format(value) {
            SpringPayloadFormat::Bindings => Self::adapt_bindings(value),
            SpringPayloadFormat::GameChannel => Self::adapt_game_channel(value),
            SpringPayloadFormat::EcoTimeSeries => Self::adapt_eco_timeseries(value),
            SpringPayloadFormat::Raw => Self::adapt_raw(value),
        }
    }

    /// Parse standard `{ "bindings": [...] }` envelope.
    fn adapt_bindings(value: &serde_json::Value) -> Result<Vec<DataBinding>, SpringAdapterError> {
        let envelope: BindingsEnvelope =
            serde_json::from_value(value.clone()).map_err(SpringAdapterError::DeserializeFailed)?;
        Ok(envelope.bindings)
    }

    /// Parse a bare `Vec<DataBinding>` array.
    fn adapt_raw(value: &serde_json::Value) -> Result<Vec<DataBinding>, SpringAdapterError> {
        let bindings: Vec<DataBinding> =
            serde_json::from_value(value.clone()).map_err(SpringAdapterError::DeserializeFailed)?;
        Ok(bindings)
    }

    /// Parse ludoSpring `{ "data": {...}, "channel": "..." }` and convert to `DataBinding`.
    fn adapt_game_channel(
        value: &serde_json::Value,
    ) -> Result<Vec<DataBinding>, SpringAdapterError> {
        let payload: GameChannelPayload =
            serde_json::from_value(value.clone()).map_err(SpringAdapterError::DeserializeFailed)?;
        let binding = Self::game_channel_to_binding(&payload)?;
        Ok(vec![binding])
    }

    /// Parse `ecoPrimals/time-series/v1` and convert each series to `DataBinding::TimeSeries`.
    fn adapt_eco_timeseries(
        value: &serde_json::Value,
    ) -> Result<Vec<DataBinding>, SpringAdapterError> {
        let envelope: EcoTimeSeriesEnvelope =
            serde_json::from_value(value.clone()).map_err(SpringAdapterError::DeserializeFailed)?;
        let bindings = envelope
            .series
            .into_iter()
            .map(|entry| DataBinding::TimeSeries {
                id: entry.id,
                label: entry.label,
                x_label: if entry.x_label.is_empty() {
                    "Time".to_string()
                } else {
                    entry.x_label
                },
                y_label: entry.y_label,
                unit: entry.unit,
                x_values: entry.timestamps,
                y_values: entry.values,
            })
            .collect();
        Ok(bindings)
    }

    /// Map a single `GameChannelPayload` to a `DataBinding`.
    #[expect(
        clippy::unnecessary_wraps,
        reason = "Result allows future GameChannelType variants to produce errors"
    )]
    fn game_channel_to_binding(
        payload: &GameChannelPayload,
    ) -> Result<DataBinding, SpringAdapterError> {
        let data = &payload.data;
        let id = data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let label = data
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let unit = data
            .get("unit")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        match payload.channel {
            GameChannelType::EngagementCurve | GameChannelType::DifficultyProfile => {
                let x_values = extract_f64_array(data, "x_values")
                    .or_else(|| extract_f64_array(data, "timestamps"))
                    .unwrap_or_default();
                let y_values = extract_f64_array(data, "y_values")
                    .or_else(|| extract_f64_array(data, "values"))
                    .unwrap_or_default();
                Ok(DataBinding::TimeSeries {
                    id,
                    label,
                    x_label: extract_string(data, "x_label").unwrap_or_else(|| "Time".to_string()),
                    y_label: extract_string(data, "y_label").unwrap_or_else(|| "Value".to_string()),
                    unit,
                    x_values,
                    y_values,
                })
            }
            GameChannelType::FlowTimeline => {
                let categories = extract_string_array(data, "categories")
                    .or_else(|| extract_string_array(data, "labels"))
                    .unwrap_or_default();
                let values = extract_f64_array(data, "values").unwrap_or_default();
                Ok(DataBinding::Bar {
                    id,
                    label,
                    categories,
                    values,
                    unit,
                })
            }
            GameChannelType::UiAnalysis => {
                let categories = extract_string_array(data, "categories")
                    .or_else(|| extract_string_array(data, "metrics"))
                    .unwrap_or_default();
                let values = extract_f64_array(data, "values").unwrap_or_default();
                Ok(DataBinding::Bar {
                    id,
                    label,
                    categories,
                    values,
                    unit,
                })
            }
            GameChannelType::InteractionCostMap | GameChannelType::AccessibilityReport => {
                let x_labels = extract_string_array(data, "x_labels").unwrap_or_default();
                let y_labels = extract_string_array(data, "y_labels").unwrap_or_default();
                let values = extract_f64_array(data, "values").unwrap_or_default();
                Ok(DataBinding::Heatmap {
                    id,
                    label,
                    x_labels,
                    y_labels,
                    values,
                    unit,
                })
            }
            GameChannelType::GenerationPreview => {
                let x = extract_f64_array(data, "x").unwrap_or_default();
                let y = extract_f64_array(data, "y").unwrap_or_default();
                let point_labels = extract_string_array(data, "point_labels").unwrap_or_default();
                Ok(DataBinding::Scatter {
                    id,
                    label,
                    x,
                    y,
                    point_labels,
                    x_label: extract_string(data, "x_label").unwrap_or_default(),
                    y_label: extract_string(data, "y_label").unwrap_or_default(),
                    unit,
                })
            }
        }
    }
}

/// Errors from the spring data adapter.
#[derive(Debug, thiserror::Error)]
pub enum SpringAdapterError {
    /// JSON deserialization failed.
    #[error("failed to deserialize spring payload: {0}")]
    DeserializeFailed(serde_json::Error),
}

fn extract_f64_array(value: &serde_json::Value, key: &str) -> Option<Vec<f64>> {
    value.get(key)?.as_array().map(|arr| {
        arr.iter()
            .filter_map(serde_json::Value::as_f64)
            .collect::<Vec<f64>>()
    })
}

fn extract_string_array(value: &serde_json::Value, key: &str) -> Option<Vec<String>> {
    value.get(key)?.as_array().map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<String>>()
    })
}

fn extract_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_bindings_format() {
        let json = serde_json::json!({
            "bindings": [
                {"channel_type": "gauge", "id": "g1", "label": "Test", "value": 50.0,
                 "min": 0.0, "max": 100.0, "unit": "u", "normal_range": [20.0, 80.0],
                 "warning_range": [0.0, 100.0]}
            ]
        });
        assert_eq!(
            SpringDataAdapter::detect_format(&json),
            SpringPayloadFormat::Bindings
        );
    }

    #[test]
    fn detect_game_channel_format() {
        let json = serde_json::json!({
            "channel": "engagement_curve",
            "data": {"id": "ec1", "label": "Engagement", "timestamps": [0.0, 1.0], "values": [0.5, 0.8], "unit": "score"}
        });
        assert_eq!(
            SpringDataAdapter::detect_format(&json),
            SpringPayloadFormat::GameChannel
        );
    }

    #[test]
    fn detect_eco_timeseries_format() {
        let json = serde_json::json!({
            "schema": "ecoPrimals/time-series/v1",
            "series": []
        });
        assert_eq!(
            SpringDataAdapter::detect_format(&json),
            SpringPayloadFormat::EcoTimeSeries
        );
    }

    #[test]
    fn detect_raw_array_format() {
        let json = serde_json::json!([
            {"channel_type": "gauge", "id": "g1", "label": "Test", "value": 50.0,
             "min": 0.0, "max": 100.0, "unit": "u", "normal_range": [20.0, 80.0],
             "warning_range": [0.0, 100.0]}
        ]);
        assert_eq!(
            SpringDataAdapter::detect_format(&json),
            SpringPayloadFormat::Raw
        );
    }

    #[test]
    fn adapt_bindings_envelope() {
        let json = serde_json::json!({
            "bindings": [
                {"channel_type": "timeseries", "id": "ts1", "label": "Metric",
                 "x_label": "t", "y_label": "v", "unit": "u",
                 "x_values": [0.0, 1.0], "y_values": [10.0, 20.0]}
            ]
        });
        let result = SpringDataAdapter::adapt(&json).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(&result[0], DataBinding::TimeSeries { id, .. } if id == "ts1"));
    }

    #[test]
    fn adapt_game_channel_engagement_curve() {
        let json = serde_json::json!({
            "channel": "engagement_curve",
            "data": {
                "id": "ec1", "label": "Player Engagement",
                "timestamps": [0.0, 1.0, 2.0], "values": [0.5, 0.8, 0.6],
                "unit": "score"
            }
        });
        let result = SpringDataAdapter::adapt(&json).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            DataBinding::TimeSeries {
                id,
                x_values,
                y_values,
                ..
            } => {
                assert_eq!(id, "ec1");
                assert_eq!(x_values.len(), 3);
                assert_eq!(y_values.len(), 3);
            }
            _ => panic!("expected TimeSeries"),
        }
    }

    #[test]
    fn adapt_game_channel_flow_timeline() {
        let json = serde_json::json!({
            "channel": "flow_timeline",
            "data": {
                "id": "ft1", "label": "Flow States",
                "categories": ["Zone 1", "Zone 2", "Zone 3"],
                "values": [0.3, 0.5, 0.2],
                "unit": "ratio"
            }
        });
        let result = SpringDataAdapter::adapt(&json).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            DataBinding::Bar { id, categories, .. } => {
                assert_eq!(id, "ft1");
                assert_eq!(categories.len(), 3);
            }
            _ => panic!("expected Bar"),
        }
    }

    #[test]
    fn adapt_game_channel_interaction_cost_map() {
        let json = serde_json::json!({
            "channel": "interaction_cost_map",
            "data": {
                "id": "icm1", "label": "Cost Map",
                "x_labels": ["A", "B"], "y_labels": ["X", "Y"],
                "values": [1.0, 2.0, 3.0, 4.0],
                "unit": "ms"
            }
        });
        let result = SpringDataAdapter::adapt(&json).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(&result[0], DataBinding::Heatmap { .. }));
    }

    #[test]
    fn adapt_game_channel_generation_preview() {
        let json = serde_json::json!({
            "channel": "generation_preview",
            "data": {
                "id": "gp1", "label": "Preview",
                "x": [1.0, 2.0], "y": [3.0, 4.0],
                "unit": "px"
            }
        });
        let result = SpringDataAdapter::adapt(&json).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(&result[0], DataBinding::Scatter { .. }));
    }

    #[test]
    fn adapt_eco_timeseries() {
        let json = serde_json::json!({
            "schema": "ecoPrimals/time-series/v1",
            "series": [
                {
                    "id": "et0_daily",
                    "label": "ET0 Daily",
                    "unit": "mm/day",
                    "timestamps": [0.0, 86_400.0, 172_800.0],
                    "values": [3.2, 4.1, 3.8]
                },
                {
                    "id": "rainfall",
                    "label": "Rainfall",
                    "unit": "mm",
                    "timestamps": [0.0, 86_400.0],
                    "values": [0.0, 12.5]
                }
            ]
        });
        let result = SpringDataAdapter::adapt(&json).unwrap();
        assert_eq!(result.len(), 2);
        match &result[0] {
            DataBinding::TimeSeries {
                id,
                x_label,
                unit,
                x_values,
                ..
            } => {
                assert_eq!(id, "et0_daily");
                assert_eq!(x_label, "Time");
                assert_eq!(unit, "mm/day");
                assert_eq!(x_values.len(), 3);
            }
            _ => panic!("expected TimeSeries"),
        }
    }

    #[test]
    fn adapt_raw_array() {
        let json = serde_json::json!([
            {"channel_type": "spectrum", "id": "s1", "label": "Spec",
             "frequencies": [1.0, 2.0], "amplitudes": [0.5, 0.3], "unit": "dB"}
        ]);
        let result = SpringDataAdapter::adapt(&json).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(&result[0], DataBinding::Spectrum { .. }));
    }

    #[test]
    fn adapt_invalid_json_returns_error() {
        let json = serde_json::json!("not a valid payload");
        let result = SpringDataAdapter::adapt(&json);
        assert!(result.is_err());
    }

    #[test]
    fn game_channel_difficulty_profile_maps_to_timeseries() {
        let json = serde_json::json!({
            "channel": "difficulty_profile",
            "data": {
                "id": "dp1", "label": "Difficulty",
                "x_values": [0.0, 1.0], "y_values": [1.0, 5.0],
                "unit": "level"
            }
        });
        let result = SpringDataAdapter::adapt(&json).unwrap();
        assert!(matches!(&result[0], DataBinding::TimeSeries { .. }));
    }

    #[test]
    fn game_channel_ui_analysis_maps_to_bar() {
        let json = serde_json::json!({
            "channel": "ui_analysis",
            "data": {
                "id": "ua1", "label": "UI Metrics",
                "metrics": ["Fitts", "Hick", "Steering"],
                "values": [0.9, 0.8, 0.7],
                "unit": "score"
            }
        });
        let result = SpringDataAdapter::adapt(&json).unwrap();
        assert!(matches!(&result[0], DataBinding::Bar { .. }));
    }

    #[test]
    fn game_channel_accessibility_report_maps_to_heatmap() {
        let json = serde_json::json!({
            "channel": "accessibility_report",
            "data": {
                "id": "ar1", "label": "Accessibility",
                "x_labels": ["Visual", "Motor"], "y_labels": ["Low", "High"],
                "values": [0.9, 0.7, 0.8, 0.6],
                "unit": "score"
            }
        });
        let result = SpringDataAdapter::adapt(&json).unwrap();
        assert!(matches!(&result[0], DataBinding::Heatmap { .. }));
    }
}
