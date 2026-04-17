// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP client used when falling back from tarpc/JSON-RPC.

use petal_tongue_ipc::{LocalHttpClient, TarpcClientError, TarpcResult};

use super::parse::{parse_capabilities_from_json, parse_health_from_json};

/// HTTP client for primal-to-primal communication (plain HTTP, no TLS).
///
/// TLS is delegated to Songbird via tower atomic IPC.
/// Falls back to HTTP when tarpc/JSON-RPC are unavailable.
#[derive(Clone)]
pub struct HttpsClient {
    pub(crate) base_url: String,
    pub(crate) client: LocalHttpClient,
}

impl HttpsClient {
    /// Try common health endpoint paths
    const HEALTH_PATHS: &[&str] = &["/health", "/api/v1/health"];
    /// Try common capabilities endpoint paths
    const CAPABILITIES_PATHS: &[&str] = &["/api/v1/capabilities", "/capabilities"];

    async fn fetch_json(&self, path: &str) -> TarpcResult<serde_json::Value> {
        let url = format!("{}{path}", self.base_url);
        let resp = self
            .client
            .get(&url)
            .await
            .map_err(|e| TarpcClientError::Connection(e.to_string()))?;
        if !resp.is_success() {
            return Err(TarpcClientError::Connection(format!(
                "HTTP {}: {}",
                resp.status(),
                url
            )));
        }
        resp.json()
            .map_err(|e| TarpcClientError::Serialization(e.to_string()))
    }

    pub(crate) async fn health(&self) -> TarpcResult<petal_tongue_ipc::HealthStatus> {
        for path in Self::HEALTH_PATHS {
            if let Ok(value) = self.fetch_json(path).await {
                return Ok(parse_health_from_json(&value));
            }
        }
        Err(TarpcClientError::Connection(format!(
            "No health endpoint responded at {}",
            self.base_url
        )))
    }

    pub(crate) async fn get_capabilities(&self) -> TarpcResult<Vec<String>> {
        for path in Self::CAPABILITIES_PATHS {
            if let Ok(value) = self.fetch_json(path).await {
                return parse_capabilities_from_json(&value);
            }
        }
        Err(TarpcClientError::Configuration(
            "No capabilities endpoint responded".to_string(),
        ))
    }
}
