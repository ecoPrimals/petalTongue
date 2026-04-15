// SPDX-License-Identifier: AGPL-3.0-or-later
//! Types for `visualization.session.status`.

use serde::{Deserialize, Serialize};

/// Request for `visualization.session.status`: query session health metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatusRequest {
    /// Session ID to query.
    pub session_id: String,
}

/// Response for `visualization.session.status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatusResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// Whether the session exists.
    pub exists: bool,
    /// Total stream updates received by this session.
    pub frame_count: u64,
    /// Seconds since last update.
    pub last_update_secs: f64,
    /// Whether backpressure is currently active.
    pub backpressure_active: bool,
    /// Number of bindings in the session.
    pub binding_count: usize,
    /// Domain hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_status_request_response_roundtrip() {
        let req = SessionStatusRequest {
            session_id: "s1".into(),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let _: SessionStatusRequest = serde_json::from_str(&json).expect("deserialize");

        let resp = SessionStatusResponse {
            session_id: "s1".into(),
            exists: true,
            frame_count: 100,
            last_update_secs: 0.5,
            backpressure_active: false,
            binding_count: 3,
            domain: Some("health".into()),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: SessionStatusResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.frame_count, 100);
    }
}
