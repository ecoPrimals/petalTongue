// SPDX-License-Identifier: AGPL-3.0-or-later
//! Types for `visualization.dismiss`.

use serde::{Deserialize, Serialize};

/// Request for `visualization.dismiss`: remove a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissRequest {
    /// Session ID to dismiss.
    pub session_id: String,
}

/// Response for `visualization.dismiss`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// Whether the session was removed.
    pub dismissed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dismiss_request_response_roundtrip() {
        let req = DismissRequest {
            session_id: "s1".into(),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let _: DismissRequest = serde_json::from_str(&json).expect("deserialize");

        let resp = DismissResponse {
            session_id: "s1".into(),
            dismissed: true,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: DismissResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(restored.dismissed);
    }
}
