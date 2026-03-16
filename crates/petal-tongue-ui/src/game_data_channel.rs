// SPDX-License-Identifier: AGPL-3.0-or-later
//! ludoSpring `GameDataChannel` → petalTongue `DataBinding` mapping.
//!
//! When ludoSpring (or any game-domain primal) calls `visualization.render` with
//! game science data, this module converts the JSON payload into the appropriate
//! `DataBinding` variant so the live session panel can render it with game theming.
//!
//! ## Channel Mapping (from `REALTIME_COLLABORATIVE_PIPELINE.md`)
//!
//! | ludoSpring Channel      | DataBinding Variant | Notes                                      |
//! |-------------------------|---------------------|--------------------------------------------|
//! | EngagementCurve         | TimeSeries          | x=time, y=engagement                       |
//! | DifficultyProfile       | TimeSeries          | x=progress, y=difficulty                   |
//! | FlowTimeline            | Bar                 | categories=flow states, values=durations   |
//! | InteractionCostMap      | Heatmap             | x=screen region, y=action, z=Fitts cost   |
//! | GenerationPreview       | Scatter             | Procedural content preview                 |
//! | AccessibilityReport     | FieldMap            | WCAG metrics per component                 |
//! | UiAnalysis              | FieldMap            | Tufte metrics per panel                    |

use petal_tongue_core::DataBinding;
use serde_json::Value;

/// Known ludoSpring game data channel types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameChannel {
    EngagementCurve,
    DifficultyProfile,
    FlowTimeline,
    InteractionCostMap,
    GenerationPreview,
    AccessibilityReport,
    UiAnalysis,
}

impl GameChannel {
    /// Parse a channel name string into a known variant.
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "EngagementCurve" | "engagement_curve" => Some(Self::EngagementCurve),
            "DifficultyProfile" | "difficulty_profile" => Some(Self::DifficultyProfile),
            "FlowTimeline" | "flow_timeline" => Some(Self::FlowTimeline),
            "InteractionCostMap" | "interaction_cost_map" => Some(Self::InteractionCostMap),
            "GenerationPreview" | "generation_preview" => Some(Self::GenerationPreview),
            "AccessibilityReport" | "accessibility_report" => Some(Self::AccessibilityReport),
            "UiAnalysis" | "ui_analysis" => Some(Self::UiAnalysis),
            _ => None,
        }
    }
}

/// Convert a ludoSpring game data payload into a `DataBinding`.
///
/// The `channel` field in the JSON selects the mapping. The remaining fields
/// provide the data. Returns `None` if the channel is unrecognized or
/// required fields are missing.
#[must_use]
pub fn map_game_channel(payload: &Value) -> Option<DataBinding> {
    let channel_name = payload.get("channel")?.as_str()?;
    let channel = GameChannel::from_str(channel_name)?;

    match channel {
        GameChannel::EngagementCurve => map_engagement_curve(payload),
        GameChannel::DifficultyProfile => map_difficulty_profile(payload),
        GameChannel::FlowTimeline => map_flow_timeline(payload),
        GameChannel::InteractionCostMap => map_interaction_cost_map(payload),
        GameChannel::GenerationPreview => map_generation_preview(payload),
        GameChannel::AccessibilityReport => map_accessibility_report(payload),
        GameChannel::UiAnalysis => map_ui_analysis(payload),
    }
}

fn extract_f64_array(v: &Value, key: &str) -> Vec<f64> {
    v.get(key)
        .and_then(|a| a.as_array())
        .map(|arr| arr.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default()
}

fn extract_string_array(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_str_or(v: &Value, key: &str, default: &str) -> String {
    v.get(key)
        .and_then(|s| s.as_str())
        .unwrap_or(default)
        .to_string()
}

fn map_engagement_curve(v: &Value) -> Option<DataBinding> {
    let x = extract_f64_array(v, "timestamps");
    let y = extract_f64_array(v, "engagement");
    if x.is_empty() || y.is_empty() {
        return None;
    }
    Some(DataBinding::TimeSeries {
        id: extract_str_or(v, "id", "engagement-curve"),
        label: extract_str_or(v, "label", "Engagement Curve"),
        x_label: "Time (s)".to_string(),
        y_label: "Engagement".to_string(),
        unit: extract_str_or(v, "unit", "score"),
        x_values: x,
        y_values: y,
    })
}

fn map_difficulty_profile(v: &Value) -> Option<DataBinding> {
    let x = extract_f64_array(v, "progress");
    let y = extract_f64_array(v, "difficulty");
    if x.is_empty() || y.is_empty() {
        return None;
    }
    Some(DataBinding::TimeSeries {
        id: extract_str_or(v, "id", "difficulty-profile"),
        label: extract_str_or(v, "label", "Difficulty Profile"),
        x_label: "Progress".to_string(),
        y_label: "Difficulty".to_string(),
        unit: extract_str_or(v, "unit", "level"),
        x_values: x,
        y_values: y,
    })
}

fn map_flow_timeline(v: &Value) -> Option<DataBinding> {
    let categories = extract_string_array(v, "flow_states");
    let values = extract_f64_array(v, "durations");
    if categories.is_empty() || values.is_empty() {
        return None;
    }
    Some(DataBinding::Bar {
        id: extract_str_or(v, "id", "flow-timeline"),
        label: extract_str_or(v, "label", "Flow Timeline"),
        categories,
        values,
        unit: extract_str_or(v, "unit", "seconds"),
    })
}

fn map_interaction_cost_map(v: &Value) -> Option<DataBinding> {
    let x_labels = extract_string_array(v, "screen_regions");
    let y_labels = extract_string_array(v, "actions");
    let values = extract_f64_array(v, "costs");
    if x_labels.is_empty() || y_labels.is_empty() || values.is_empty() {
        return None;
    }
    Some(DataBinding::Heatmap {
        id: extract_str_or(v, "id", "interaction-cost-map"),
        label: extract_str_or(v, "label", "Interaction Cost (Fitts)"),
        x_labels,
        y_labels,
        values,
        unit: extract_str_or(v, "unit", "bits"),
    })
}

fn map_generation_preview(v: &Value) -> Option<DataBinding> {
    let x = extract_f64_array(v, "x");
    let y = extract_f64_array(v, "y");
    if x.is_empty() || y.is_empty() {
        return None;
    }
    Some(DataBinding::Scatter {
        id: extract_str_or(v, "id", "generation-preview"),
        label: extract_str_or(v, "label", "Generation Preview"),
        x,
        y,
        point_labels: extract_string_array(v, "labels"),
        x_label: extract_str_or(v, "x_label", "X"),
        y_label: extract_str_or(v, "y_label", "Y"),
        unit: extract_str_or(v, "unit", "tile"),
    })
}

fn map_accessibility_report(v: &Value) -> Option<DataBinding> {
    let grid_x = extract_f64_array(v, "grid_x");
    let grid_y = extract_f64_array(v, "grid_y");
    let values = extract_f64_array(v, "values");
    if grid_x.is_empty() || grid_y.is_empty() || values.is_empty() {
        return None;
    }
    Some(DataBinding::FieldMap {
        id: extract_str_or(v, "id", "accessibility-report"),
        label: extract_str_or(v, "label", "Accessibility Report (WCAG)"),
        grid_x,
        grid_y,
        values,
        unit: extract_str_or(v, "unit", "score"),
    })
}

fn map_ui_analysis(v: &Value) -> Option<DataBinding> {
    let grid_x = extract_f64_array(v, "grid_x");
    let grid_y = extract_f64_array(v, "grid_y");
    let values = extract_f64_array(v, "values");
    if grid_x.is_empty() || grid_y.is_empty() || values.is_empty() {
        return None;
    }
    Some(DataBinding::FieldMap {
        id: extract_str_or(v, "id", "ui-analysis"),
        label: extract_str_or(v, "label", "UI Analysis (Tufte)"),
        grid_x,
        grid_y,
        values,
        unit: extract_str_or(v, "unit", "ratio"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_known_channels() {
        assert_eq!(
            GameChannel::from_str("EngagementCurve"),
            Some(GameChannel::EngagementCurve)
        );
        assert_eq!(
            GameChannel::from_str("flow_timeline"),
            Some(GameChannel::FlowTimeline)
        );
        assert_eq!(GameChannel::from_str("unknown"), None);
    }

    #[test]
    fn map_engagement_curve() {
        let payload = json!({
            "channel": "EngagementCurve",
            "timestamps": [0.0, 1.0, 2.0],
            "engagement": [0.5, 0.8, 0.6]
        });
        let binding = map_game_channel(&payload).expect("should map");
        match binding {
            DataBinding::TimeSeries {
                id,
                x_values,
                y_values,
                ..
            } => {
                assert_eq!(id, "engagement-curve");
                assert_eq!(x_values.len(), 3);
                assert_eq!(y_values.len(), 3);
            }
            _ => panic!("expected TimeSeries"),
        }
    }

    #[test]
    fn map_difficulty_profile() {
        let payload = json!({
            "channel": "DifficultyProfile",
            "progress": [0.0, 0.5, 1.0],
            "difficulty": [1.0, 3.0, 5.0]
        });
        let binding = map_game_channel(&payload).expect("should map");
        assert!(matches!(binding, DataBinding::TimeSeries { .. }));
    }

    #[test]
    fn map_flow_timeline() {
        let payload = json!({
            "channel": "FlowTimeline",
            "flow_states": ["anxiety", "flow", "boredom"],
            "durations": [10.0, 45.0, 5.0]
        });
        let binding = map_game_channel(&payload).expect("should map");
        match binding {
            DataBinding::Bar {
                categories, values, ..
            } => {
                assert_eq!(categories.len(), 3);
                assert_eq!(values.len(), 3);
            }
            _ => panic!("expected Bar"),
        }
    }

    #[test]
    fn map_interaction_cost_map() {
        let payload = json!({
            "channel": "InteractionCostMap",
            "screen_regions": ["top-left", "center", "bottom-right"],
            "actions": ["click", "drag"],
            "costs": [2.5, 1.0, 3.0, 1.5, 0.5, 2.0]
        });
        let binding = map_game_channel(&payload).expect("should map");
        assert!(matches!(binding, DataBinding::Heatmap { .. }));
    }

    #[test]
    fn map_generation_preview() {
        let payload = json!({
            "channel": "GenerationPreview",
            "x": [1.0, 2.0, 3.0],
            "y": [4.0, 5.0, 6.0],
            "labels": ["a", "b", "c"]
        });
        let binding = map_game_channel(&payload).expect("should map");
        assert!(matches!(binding, DataBinding::Scatter { .. }));
    }

    #[test]
    fn map_accessibility_report() {
        let payload = json!({
            "channel": "AccessibilityReport",
            "grid_x": [0.0, 1.0],
            "grid_y": [0.0, 1.0],
            "values": [0.9, 0.7, 0.85, 0.95]
        });
        let binding = map_game_channel(&payload).expect("should map");
        assert!(matches!(binding, DataBinding::FieldMap { .. }));
    }

    #[test]
    fn map_ui_analysis() {
        let payload = json!({
            "channel": "UiAnalysis",
            "grid_x": [0.0, 1.0, 2.0],
            "grid_y": [0.0, 1.0],
            "values": [0.1, 0.2, 0.3, 0.4, 0.5, 0.6]
        });
        let binding = map_game_channel(&payload).expect("should map");
        assert!(matches!(binding, DataBinding::FieldMap { .. }));
    }

    #[test]
    fn unknown_channel_returns_none() {
        let payload = json!({"channel": "SomethingNew", "data": []});
        assert!(map_game_channel(&payload).is_none());
    }

    #[test]
    fn missing_channel_field_returns_none() {
        let payload = json!({"data": [1, 2, 3]});
        assert!(map_game_channel(&payload).is_none());
    }

    #[test]
    fn empty_data_returns_none() {
        let payload = json!({
            "channel": "EngagementCurve",
            "timestamps": [],
            "engagement": []
        });
        assert!(map_game_channel(&payload).is_none());
    }

    #[test]
    fn custom_id_and_label() {
        let payload = json!({
            "channel": "EngagementCurve",
            "id": "my-engagement",
            "label": "Player Engagement",
            "timestamps": [0.0, 1.0],
            "engagement": [0.5, 0.9]
        });
        let binding = map_game_channel(&payload).expect("should map");
        match binding {
            DataBinding::TimeSeries { id, label, .. } => {
                assert_eq!(id, "my-engagement");
                assert_eq!(label, "Player Engagement");
            }
            _ => panic!("expected TimeSeries"),
        }
    }

    #[test]
    fn snake_case_channel_names_work() {
        let payload = json!({
            "channel": "interaction_cost_map",
            "screen_regions": ["center"],
            "actions": ["click"],
            "costs": [1.5]
        });
        assert!(map_game_channel(&payload).is_some());
    }

    #[test]
    fn all_channel_variants_parse() {
        assert!(GameChannel::from_str("EngagementCurve").is_some());
        assert!(GameChannel::from_str("DifficultyProfile").is_some());
        assert!(GameChannel::from_str("FlowTimeline").is_some());
        assert!(GameChannel::from_str("InteractionCostMap").is_some());
        assert!(GameChannel::from_str("GenerationPreview").is_some());
        assert!(GameChannel::from_str("AccessibilityReport").is_some());
        assert!(GameChannel::from_str("UiAnalysis").is_some());
    }

    #[test]
    fn engagement_curve_empty_y_returns_none() {
        let payload = json!({
            "channel": "EngagementCurve",
            "timestamps": [0.0, 1.0],
            "engagement": []
        });
        assert!(map_game_channel(&payload).is_none());
    }

    #[test]
    fn scatter_point_labels_optional() {
        let payload = json!({
            "channel": "GenerationPreview",
            "x": [1.0, 2.0],
            "y": [3.0, 4.0]
        });
        let binding = map_game_channel(&payload).expect("should map");
        assert!(matches!(binding, DataBinding::Scatter { .. }));
    }
}
