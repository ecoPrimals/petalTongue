// SPDX-License-Identifier: AGPL-3.0-or-later
//! ludoSpring `{ "data": {...}, "channel": "..." }` envelope parsing.

use serde::{Deserialize, Serialize};

use crate::data_channel::DataBinding;

use super::SpringAdapterError;
use super::helpers::{extract_f64_array, extract_string, extract_string_array};

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

/// Parse ludoSpring `{ "data": {...}, "channel": "..." }` and convert to `DataBinding`.
pub(super) fn adapt_game_channel(
    value: serde_json::Value,
) -> Result<Vec<DataBinding>, SpringAdapterError> {
    let payload: GameChannelPayload =
        serde_json::from_value(value).map_err(SpringAdapterError::DeserializeFailed)?;
    let binding = game_channel_to_binding(&payload)?;
    Ok(vec![binding])
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
