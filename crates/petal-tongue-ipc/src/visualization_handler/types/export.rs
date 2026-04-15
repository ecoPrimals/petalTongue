// SPDX-License-Identifier: AGPL-3.0-or-later
//! Types for `visualization.export`.

use serde::{Deserialize, Serialize};

/// Request for `visualization.export`: export a session to a format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    /// Session ID to export.
    pub session_id: String,
    /// Output format: "svg", "html", "audio", "description", "braille", "terminal", "gpu".
    pub format: String,
}

/// Response for `visualization.export`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// Format that was used.
    pub format: String,
    /// Exported content (SVG string, JSON, or description text).
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_request_response_roundtrip() {
        let req = ExportRequest {
            session_id: "s1".into(),
            format: "svg".into(),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let _: ExportRequest = serde_json::from_str(&json).expect("deserialize");

        let resp = ExportResponse {
            session_id: "s1".into(),
            format: "svg".into(),
            content: "<svg></svg>".into(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: ExportResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.content, "<svg></svg>");
    }
}
