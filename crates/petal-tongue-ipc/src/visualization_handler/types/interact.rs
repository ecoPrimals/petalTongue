// SPDX-License-Identifier: AGPL-3.0-or-later
//! Types for `visualization.interact.*` and perspective state.

use serde::{Deserialize, Serialize};

/// Request for `visualization.interact.apply`: apply an interaction intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionApplyRequest {
    /// Intent: "select", "focus", "inspect", "filter", "navigate".
    pub intent: String,
    /// Target identifiers to apply the intent to.
    pub targets: Vec<String>,
    /// Optional grammar ID for scoped interactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grammar_id: Option<String>,
}

/// Response for `visualization.interact.apply`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionApplyResponse {
    /// Whether the interaction was accepted.
    pub accepted: bool,
    /// Number of targets resolved.
    pub targets_resolved: usize,
    /// Number of callback dispatches queued for push delivery (PT-06).
    #[serde(default)]
    pub pending_callbacks: usize,
}

/// A visualization perspective (view configuration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perspective {
    /// Perspective identifier.
    pub id: String,
    /// Modalities active in this perspective.
    pub modalities: Vec<String>,
    /// Current selection.
    pub selection: Vec<String>,
    /// Sync mode (e.g. "`shared_selection`").
    pub sync_mode: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interaction_apply_request_response_roundtrip() {
        let req = InteractionApplyRequest {
            intent: "select".into(),
            targets: vec!["t1".into()],
            grammar_id: Some("g1".into()),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let restored: InteractionApplyRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.grammar_id, Some("g1".into()));

        let resp = InteractionApplyResponse {
            accepted: true,
            targets_resolved: 1,
            pending_callbacks: 0,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: InteractionApplyResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.targets_resolved, 1);
        assert_eq!(restored.pending_callbacks, 0);
    }

    #[test]
    fn perspective_roundtrip() {
        let p = Perspective {
            id: "p1".into(),
            modalities: vec!["svg".into()],
            selection: vec![],
            sync_mode: "shared_selection".into(),
        };
        let json = serde_json::to_string(&p).expect("serialize");
        let restored: Perspective = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.id, "p1");
    }
}
