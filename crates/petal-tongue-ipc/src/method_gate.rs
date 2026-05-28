// SPDX-License-Identifier: AGPL-3.0-or-later
//! JH-0 MethodGate: pre-dispatch authorization for JSON-RPC methods.
//!
//! Every incoming JSON-RPC call passes through [`MethodGate::check`] before
//! reaching the handler dispatch table. Methods are classified as [`Public`]
//! (always allowed) or [`Protected`] (require a capability token when
//! enforcement is active).
//!
//! Default mode is [`Permissive`] (logs violations but allows all calls).
//! Switch to [`Enforced`] via `PETALTONGUE_AUTH_MODE=enforced`.
//!
//! [`Public`]: MethodAccess::Public
//! [`Protected`]: MethodAccess::Protected
//! [`Permissive`]: EnforcementMode::Permissive
//! [`Enforced`]: EnforcementMode::Enforced

use crate::json_rpc::{JsonRpcResponse, error_codes};
use petal_tongue_core::constants;
use serde_json::json;

/// JSON-RPC error code: caller identity not established.
pub const UNAUTHORIZED: i32 = error_codes::SERVER_ERROR_START; // -32000

/// JSON-RPC error code: protected method, no valid token.
pub const PERMISSION_DENIED: i32 = -32001;

/// Method access classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodAccess {
    /// Always allowed without authentication.
    Public,
    /// Requires a capability token when enforcement is active.
    Protected,
}

/// Enforcement posture for the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementMode {
    /// Log violations but allow all calls (default, backward-compatible).
    Permissive,
    /// Reject protected calls without a valid token.
    Enforced,
}

impl EnforcementMode {
    /// Read from `PETALTONGUE_AUTH_MODE` (or `AUTH_MODE` fallback).
    #[must_use]
    pub fn from_env() -> Self {
        let val = std::env::var(constants::PETALTONGUE_AUTH_MODE)
            .or_else(|_| std::env::var(constants::AUTH_MODE))
            .unwrap_or_default();
        if val.eq_ignore_ascii_case("enforced") {
            Self::Enforced
        } else {
            Self::Permissive
        }
    }

    /// String representation for IPC responses.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Permissive => "permissive",
            Self::Enforced => "enforced",
        }
    }
}

/// Per-connection caller metadata.
#[derive(Debug, Clone)]
pub struct CallerContext {
    /// Connection origin.
    pub origin: ConnectionOrigin,
    /// Bearer token extracted from request params (if any).
    pub bearer_token: Option<String>,
}

impl CallerContext {
    /// Construct context for a Unix domain socket connection.
    #[must_use]
    pub const fn unix() -> Self {
        Self {
            origin: ConnectionOrigin::Unix,
            bearer_token: None,
        }
    }

    /// Construct context for a TCP connection.
    #[must_use]
    pub const fn tcp(addr: std::net::SocketAddr) -> Self {
        let origin = if addr.ip().is_loopback() {
            ConnectionOrigin::Loopback
        } else {
            ConnectionOrigin::Remote
        };
        Self {
            origin,
            bearer_token: None,
        }
    }

    /// Extract `_bearer_token` from request params (if present).
    #[must_use]
    pub fn with_token_from_params(mut self, params: &serde_json::Value) -> Self {
        if let Some(token) = params
            .get("_bearer_token")
            .and_then(|v| v.as_str())
            .map(String::from)
        {
            self.bearer_token = Some(token);
        }
        self
    }
}

/// Where the connection originated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionOrigin {
    /// Local Unix domain socket (trusted).
    Unix,
    /// TCP from loopback address (127.0.0.1 / ::1).
    Loopback,
    /// TCP from a non-loopback address.
    Remote,
}

impl ConnectionOrigin {
    /// String representation for IPC responses.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Unix => "unix",
            Self::Loopback => "loopback",
            Self::Remote => "remote",
        }
    }
}

/// Pre-dispatch authorization gate.
#[derive(Debug)]
pub struct MethodGate {
    mode: EnforcementMode,
}

impl MethodGate {
    /// Create a gate with the given enforcement mode.
    #[must_use]
    pub const fn new(mode: EnforcementMode) -> Self {
        Self { mode }
    }

    /// Create a gate from the `PETALTONGUE_AUTH_MODE` environment variable.
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(EnforcementMode::from_env())
    }

    /// Current enforcement mode.
    #[must_use]
    pub const fn mode(&self) -> EnforcementMode {
        self.mode
    }

    /// Check whether a method call should proceed.
    ///
    /// Returns `None` if the call is allowed, or `Some(error_response)` if it
    /// should be rejected.
    #[must_use]
    pub fn check(
        &self,
        method: &str,
        request_id: &serde_json::Value,
        ctx: &CallerContext,
    ) -> Option<JsonRpcResponse> {
        if classify_method(method) == MethodAccess::Public {
            return None;
        }

        if ctx.bearer_token.is_some() {
            return None;
        }

        match self.mode {
            EnforcementMode::Permissive => {
                tracing::warn!(
                    method,
                    origin = ctx.origin.as_str(),
                    "MethodGate: protected method called without token (permissive — allowing)"
                );
                None
            }
            EnforcementMode::Enforced => {
                tracing::warn!(
                    method,
                    origin = ctx.origin.as_str(),
                    "MethodGate: rejecting unauthenticated call to protected method"
                );
                Some(JsonRpcResponse::error_with_data(
                    request_id.clone(),
                    PERMISSION_DENIED,
                    format!("Permission denied: {method} requires authentication"),
                    json!({ "method": method }),
                ))
            }
        }
    }

    /// Handle `auth.check` — is the caller authenticated?
    #[must_use]
    pub fn handle_auth_check(&self, id: serde_json::Value, ctx: &CallerContext) -> JsonRpcResponse {
        JsonRpcResponse::success(
            id,
            json!({
                "authenticated": ctx.bearer_token.is_some(),
                "mode": self.mode.as_str(),
            }),
        )
    }

    /// Handle `auth.mode` — current enforcement mode.
    #[must_use]
    pub fn handle_auth_mode(&self, id: serde_json::Value) -> JsonRpcResponse {
        JsonRpcResponse::success(id, json!({ "mode": self.mode.as_str() }))
    }

    /// Handle `auth.peer_info` — connection origin introspection.
    #[must_use]
    pub fn handle_auth_peer_info(
        &self,
        id: serde_json::Value,
        ctx: &CallerContext,
    ) -> JsonRpcResponse {
        JsonRpcResponse::success(
            id,
            json!({
                "origin": ctx.origin.as_str(),
                "has_token": ctx.bearer_token.is_some(),
            }),
        )
    }
}

/// Classify a JSON-RPC method as Public or Protected.
///
/// Public methods are always allowed regardless of enforcement mode:
/// - `health.*` (prefix)
/// - `identity.get`
/// - `capabilities.list`, `capability.list`
/// - `lifecycle.status`
/// - `auth.check`, `auth.mode`, `auth.peer_info`
#[must_use]
pub fn classify_method(method: &str) -> MethodAccess {
    if method.starts_with("health.") || method == "health" || method == "ping" {
        return MethodAccess::Public;
    }
    match method {
        "status"
        | "check"
        | "identity.get"
        | "capabilities.list"
        | "capability.list"
        | "primal.capabilities"
        | "primal.announce"
        | "capability.announce"
        | "btsp.capabilities"
        | "lifecycle.status"
        | "auth.check"
        | "auth.mode"
        | "auth.peer_info" => MethodAccess::Public,
        _ => MethodAccess::Protected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_classify_health_methods_are_public() {
        assert_eq!(classify_method("health.check"), MethodAccess::Public);
        assert_eq!(classify_method("health.liveness"), MethodAccess::Public);
        assert_eq!(classify_method("health.readiness"), MethodAccess::Public);
        assert_eq!(classify_method("health.get"), MethodAccess::Public);
        assert_eq!(classify_method("health"), MethodAccess::Public);
        assert_eq!(classify_method("ping"), MethodAccess::Public);
        assert_eq!(classify_method("status"), MethodAccess::Public);
        assert_eq!(classify_method("check"), MethodAccess::Public);
    }

    #[test]
    fn test_classify_identity_lifecycle_public() {
        assert_eq!(classify_method("identity.get"), MethodAccess::Public);
        assert_eq!(classify_method("lifecycle.status"), MethodAccess::Public);
    }

    #[test]
    fn test_classify_capabilities_public() {
        assert_eq!(classify_method("capabilities.list"), MethodAccess::Public);
        assert_eq!(classify_method("capability.list"), MethodAccess::Public);
        assert_eq!(classify_method("primal.capabilities"), MethodAccess::Public);
    }

    #[test]
    fn test_classify_auth_methods_public() {
        assert_eq!(classify_method("auth.check"), MethodAccess::Public);
        assert_eq!(classify_method("auth.mode"), MethodAccess::Public);
        assert_eq!(classify_method("auth.peer_info"), MethodAccess::Public);
    }

    #[test]
    fn test_classify_visualization_protected() {
        assert_eq!(
            classify_method("visualization.render"),
            MethodAccess::Protected
        );
        assert_eq!(
            classify_method("visualization.render.graph"),
            MethodAccess::Protected
        );
        assert_eq!(
            classify_method("visualization.export"),
            MethodAccess::Protected
        );
        assert_eq!(
            classify_method("visualization.scene.verify"),
            MethodAccess::Protected
        );
    }

    #[test]
    fn test_classify_interaction_protected() {
        assert_eq!(
            classify_method("interaction.subscribe"),
            MethodAccess::Protected
        );
        assert_eq!(classify_method("interaction.poll"), MethodAccess::Protected);
    }

    #[test]
    fn test_classify_ui_motor_audio_protected() {
        assert_eq!(classify_method("ui.render"), MethodAccess::Protected);
        assert_eq!(classify_method("motor.set_panel"), MethodAccess::Protected);
        assert_eq!(classify_method("audio.synthesize"), MethodAccess::Protected);
    }

    #[test]
    fn test_classify_unknown_protected() {
        assert_eq!(classify_method("does.not.exist"), MethodAccess::Protected);
    }

    #[test]
    fn test_gate_public_always_passes() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let ctx = CallerContext::unix();
        assert!(gate.check("health.check", &json!(1), &ctx).is_none());
        assert!(gate.check("identity.get", &json!(1), &ctx).is_none());
        assert!(gate.check("auth.mode", &json!(1), &ctx).is_none());
    }

    #[test]
    fn test_gate_protected_with_token_passes() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let ctx = CallerContext {
            origin: ConnectionOrigin::Unix,
            bearer_token: Some("test-token".to_owned()),
        };
        assert!(
            gate.check("visualization.render", &json!(1), &ctx)
                .is_none()
        );
    }

    #[test]
    fn test_gate_protected_no_token_permissive_allows() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let ctx = CallerContext::unix();
        assert!(
            gate.check("visualization.render", &json!(1), &ctx)
                .is_none()
        );
    }

    #[test]
    fn test_gate_protected_no_token_enforced_rejects() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let ctx = CallerContext::unix();
        let resp = gate
            .check("visualization.render", &json!(1), &ctx)
            .expect("should reject");
        let err = resp.error.as_ref().expect("error");
        assert_eq!(err.code, PERMISSION_DENIED);
        assert!(err.message.contains("visualization.render"));
    }

    #[test]
    fn test_gate_error_code_values() {
        assert_eq!(UNAUTHORIZED, -32000);
        assert_eq!(PERMISSION_DENIED, -32001);
    }

    #[test]
    fn test_enforcement_mode_from_env_default() {
        let mode = EnforcementMode::from_env();
        assert_eq!(mode, EnforcementMode::Permissive);
    }

    #[test]
    fn test_enforcement_mode_as_str() {
        assert_eq!(EnforcementMode::Permissive.as_str(), "permissive");
        assert_eq!(EnforcementMode::Enforced.as_str(), "enforced");
    }

    #[test]
    fn test_caller_context_unix() {
        let ctx = CallerContext::unix();
        assert_eq!(ctx.origin, ConnectionOrigin::Unix);
        assert!(ctx.bearer_token.is_none());
    }

    #[test]
    fn test_caller_context_tcp_loopback() {
        let addr: std::net::SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let ctx = CallerContext::tcp(addr);
        assert_eq!(ctx.origin, ConnectionOrigin::Loopback);
    }

    #[test]
    fn test_caller_context_tcp_remote() {
        let addr: std::net::SocketAddr = "192.168.1.50:12345".parse().unwrap();
        let ctx = CallerContext::tcp(addr);
        assert_eq!(ctx.origin, ConnectionOrigin::Remote);
    }

    #[test]
    fn test_caller_context_with_token_from_params() {
        let ctx = CallerContext::unix()
            .with_token_from_params(&json!({"_bearer_token": "abc123", "other": "data"}));
        assert_eq!(ctx.bearer_token.as_deref(), Some("abc123"));
    }

    #[test]
    fn test_caller_context_with_no_token_in_params() {
        let ctx = CallerContext::unix().with_token_from_params(&json!({"key": "value"}));
        assert!(ctx.bearer_token.is_none());
    }

    #[test]
    fn test_auth_check_unauthenticated() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let ctx = CallerContext::unix();
        let resp = gate.handle_auth_check(json!(1), &ctx);
        let r = resp.result.unwrap();
        assert_eq!(r["authenticated"], false);
        assert_eq!(r["mode"], "permissive");
    }

    #[test]
    fn test_auth_check_authenticated() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let ctx = CallerContext {
            origin: ConnectionOrigin::Unix,
            bearer_token: Some("tok".to_owned()),
        };
        let resp = gate.handle_auth_check(json!(1), &ctx);
        let r = resp.result.unwrap();
        assert_eq!(r["authenticated"], true);
        assert_eq!(r["mode"], "enforced");
    }

    #[test]
    fn test_auth_mode_handler() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let resp = gate.handle_auth_mode(json!(1));
        let r = resp.result.unwrap();
        assert_eq!(r["mode"], "enforced");
    }

    #[test]
    fn test_auth_peer_info_handler() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let ctx = CallerContext {
            origin: ConnectionOrigin::Remote,
            bearer_token: Some("tok".to_owned()),
        };
        let resp = gate.handle_auth_peer_info(json!(1), &ctx);
        let r = resp.result.unwrap();
        assert_eq!(r["origin"], "remote");
        assert_eq!(r["has_token"], true);
    }

    #[test]
    fn test_connection_origin_as_str() {
        assert_eq!(ConnectionOrigin::Unix.as_str(), "unix");
        assert_eq!(ConnectionOrigin::Loopback.as_str(), "loopback");
        assert_eq!(ConnectionOrigin::Remote.as_str(), "remote");
    }
}
