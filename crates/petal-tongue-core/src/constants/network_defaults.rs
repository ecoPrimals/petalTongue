// SPDX-License-Identifier: AGPL-3.0-or-later
//! Runtime network defaults: capability registry > env > compile-time const.

use crate::capability_registry::CapabilityRegistry;

use super::env_vars as env;
use super::network::{
    ALTERNATIVE_RUN_DIR, DEFAULT_BIND_HOST, DEFAULT_DISCOVERY_PORTS, DEFAULT_DISPLAY_BACKEND_PORT,
    DEFAULT_GPU_COMPUTE_ENDPOINT, DEFAULT_HEADLESS_PORT, DEFAULT_LOOPBACK_HOST,
    DEFAULT_SANDBOX_DISCOVERY_PORT, DEFAULT_SANDBOX_SECURITY_PORT, DEFAULT_WEB_PORT,
    DEFAULT_WEBSOCKET_PORT, ECOSYSTEM_TCP_FALLBACK_PORT, LEGACY_TMP_PREFIX,
};

/// Runtime-resolved network defaults with capability-registry awareness.
///
/// Compile-time [`DEFAULT_*`] constants remain the ultimate fallback.
/// Prefer [`Self::resolve()`] in runtime code over reading constants directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkDefaults {
    /// Web server port.
    pub web_port: u16,
    /// Headless API port.
    pub headless_port: u16,
    /// WebSocket streaming port.
    pub websocket_port: u16,
    /// Sandbox security endpoint port.
    pub sandbox_security_port: u16,
    /// Sandbox discovery endpoint port.
    pub sandbox_discovery_port: u16,
    /// Display backend / GPU compute port.
    pub display_backend_port: u16,
    /// Ecosystem TCP fallback port.
    pub ecosystem_tcp_fallback_port: u16,
    /// HTTP discovery probe ports.
    pub discovery_ports: Vec<u16>,
    /// Loopback host for local-only connections.
    pub loopback_host: String,
    /// Bind host for servers listening on all interfaces.
    pub bind_host: String,
    /// GPU compute endpoint URL fallback.
    pub gpu_compute_endpoint: String,
    /// Legacy `/tmp` socket path prefix.
    pub legacy_tmp_prefix: String,
    /// Alternative ecosystem runtime directory.
    pub alternative_run_dir: String,
}

impl Default for NetworkDefaults {
    fn default() -> Self {
        Self {
            web_port: DEFAULT_WEB_PORT,
            headless_port: DEFAULT_HEADLESS_PORT,
            websocket_port: DEFAULT_WEBSOCKET_PORT,
            sandbox_security_port: DEFAULT_SANDBOX_SECURITY_PORT,
            sandbox_discovery_port: DEFAULT_SANDBOX_DISCOVERY_PORT,
            display_backend_port: DEFAULT_DISPLAY_BACKEND_PORT,
            ecosystem_tcp_fallback_port: ECOSYSTEM_TCP_FALLBACK_PORT,
            discovery_ports: DEFAULT_DISCOVERY_PORTS.to_vec(),
            loopback_host: DEFAULT_LOOPBACK_HOST.to_owned(),
            bind_host: DEFAULT_BIND_HOST.to_owned(),
            gpu_compute_endpoint: DEFAULT_GPU_COMPUTE_ENDPOINT.to_owned(),
            legacy_tmp_prefix: LEGACY_TMP_PREFIX.to_owned(),
            alternative_run_dir: ALTERNATIVE_RUN_DIR.to_owned(),
        }
    }
}

impl NetworkDefaults {
    /// Resolve defaults with priority: registry > env > const.
    #[must_use]
    pub fn resolve() -> Self {
        let base = Self::default();
        let with_env = Self::apply_env(base);
        let registry = CapabilityRegistry::load_default().unwrap_or_default();
        Self::apply_registry(with_env, &registry)
    }

    /// Build defaults from environment variables with const fallback.
    #[must_use]
    pub fn from_env() -> Self {
        Self::apply_env(Self::default())
    }

    /// Build defaults from a capability registry with const fallback.
    #[must_use]
    pub fn from_capability_registry(registry: &CapabilityRegistry) -> Self {
        Self::apply_registry(Self::default(), registry)
    }

    fn apply_env(mut defaults: Self) -> Self {
        if let Some(port) = env_port(env::PETALTONGUE_WEB_PORT) {
            defaults.web_port = port;
        }
        if let Some(port) = env_port(env::PETALTONGUE_HEADLESS_PORT) {
            defaults.headless_port = port;
        }
        if let Some(port) = env_port("WEBSOCKET_PORT") {
            defaults.websocket_port = port;
        }
        if let Some(port) = env_port(env::PETALTONGUE_SANDBOX_SECURITY_PORT) {
            defaults.sandbox_security_port = port;
        }
        if let Some(port) = env_port("PETALTONGUE_TCP_PORT") {
            defaults.ecosystem_tcp_fallback_port = port;
        }
        if let Some(port) = env_port(env::DISPLAY_BACKEND_PORT) {
            defaults.display_backend_port = port;
        }
        if let Some(ports) = env_discovery_ports() {
            defaults.discovery_ports = ports;
        }
        if let Some(endpoint) = env_gpu_compute_endpoint() {
            defaults.gpu_compute_endpoint = endpoint;
        }
        defaults
    }

    fn apply_registry(mut defaults: Self, registry: &CapabilityRegistry) -> Self {
        if let Some(port) = registry
            .fallback_port("web_port")
            .or_else(|| registry.capability_port("visualization"))
        {
            defaults.web_port = port;
        }
        if let Some(port) = registry
            .fallback_port("headless_port")
            .or_else(|| registry.capability_port("discovery"))
        {
            defaults.headless_port = port;
        }
        if let Some(port) = registry.fallback_port("websocket_port") {
            defaults.websocket_port = port;
        }
        if let Some(port) = registry
            .fallback_port("sandbox_security_port")
            .or_else(|| registry.capability_port("security"))
        {
            defaults.sandbox_security_port = port;
        }
        if let Some(port) = registry.fallback_port("sandbox_discovery_port") {
            defaults.sandbox_discovery_port = port;
        }
        if let Some(port) = registry
            .fallback_port("display_backend_port")
            .or_else(|| registry.capability_port("compute"))
        {
            defaults.display_backend_port = port;
        }
        if let Some(port) = registry.fallback_port("tcp_fallback_port") {
            defaults.ecosystem_tcp_fallback_port = port;
        }
        if let Some(ports) = registry.fallback_discovery_ports() {
            defaults.discovery_ports = ports.to_vec();
        }
        if let Some(host) = registry.fallback_string("loopback_host") {
            host.clone_into(&mut defaults.loopback_host);
        }
        if let Some(host) = registry.fallback_string("bind_host") {
            host.clone_into(&mut defaults.bind_host);
        }
        if let Some(endpoint) = registry.fallback_string("gpu_compute_endpoint") {
            endpoint.clone_into(&mut defaults.gpu_compute_endpoint);
        }
        if let Some(prefix) = registry.fallback_string("legacy_tmp_prefix") {
            prefix.clone_into(&mut defaults.legacy_tmp_prefix);
        }
        if let Some(dir) = registry.fallback_string("alternative_run_dir") {
            dir.clone_into(&mut defaults.alternative_run_dir);
        }
        defaults
    }
}

/// Cached [`NetworkDefaults::resolve()`] for hot-path helpers.
pub(super) fn resolved_network_defaults() -> &'static NetworkDefaults {
    static RESOLVED: std::sync::OnceLock<NetworkDefaults> = std::sync::OnceLock::new();
    RESOLVED.get_or_init(NetworkDefaults::resolve)
}

fn env_port(key: &str) -> Option<u16> {
    std::env::var(key).ok().and_then(|s| s.parse::<u16>().ok())
}

fn env_discovery_ports() -> Option<Vec<u16>> {
    std::env::var(env::PETALTONGUE_DISCOVERY_PORTS)
        .or_else(|_| std::env::var(env::DISCOVERY_PORTS))
        .ok()
        .map(|s| {
            s.split(',')
                .filter_map(|p| p.trim().parse::<u16>().ok())
                .collect::<Vec<u16>>()
        })
        .filter(|ports| !ports.is_empty())
}

/// GPU compute endpoint from env var chain (call-time override).
pub(super) fn env_gpu_compute_endpoint() -> Option<String> {
    std::env::var(env::PETALTONGUE_GPU_COMPUTE_ENDPOINT)
        .ok()
        .or_else(|| std::env::var(env::GPU_RENDERING_ENDPOINT).ok())
        .or_else(|| std::env::var(env::COMPUTE_PROVIDER_ENDPOINT).ok())
        .or_else(|| std::env::var(env::GPU_COMPUTE_ENDPOINT).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability_registry::CapabilityRegistry;
    use crate::test_fixtures::env_test_helpers;

    #[test]
    fn network_defaults_default_matches_consts() {
        let defaults = NetworkDefaults::default();
        assert_eq!(defaults.web_port, DEFAULT_WEB_PORT);
        assert_eq!(defaults.headless_port, DEFAULT_HEADLESS_PORT);
        assert_eq!(defaults.websocket_port, DEFAULT_WEBSOCKET_PORT);
        assert_eq!(defaults.display_backend_port, DEFAULT_DISPLAY_BACKEND_PORT);
        assert_eq!(defaults.gpu_compute_endpoint, DEFAULT_GPU_COMPUTE_ENDPOINT);
        assert_eq!(defaults.discovery_ports, DEFAULT_DISCOVERY_PORTS);
    }

    #[test]
    fn from_capability_registry_resolves_ports() {
        let toml = r#"
[network.fallbacks]
web_port = 4100
headless_port = 9100
gpu_compute_endpoint = "tarpc://localhost:9100"
"#;
        let registry = CapabilityRegistry::parse_toml(toml).expect("parse");
        let defaults = NetworkDefaults::from_capability_registry(&registry);
        assert_eq!(defaults.web_port, 4100);
        assert_eq!(defaults.headless_port, 9100);
        assert_eq!(defaults.gpu_compute_endpoint, "tarpc://localhost:9100");
    }

    #[test]
    fn resolve_registry_overrides_env() {
        let dir =
            std::env::temp_dir().join(format!("petaltongue-cap-registry-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("capability_registry.toml");
        std::fs::write(
            &path,
            r"
[network.fallbacks]
web_port = 4200
",
        )
        .expect("write temp registry");

        env_test_helpers::with_env_vars(
            &[
                (
                    crate::capability_registry::ENV_CAPABILITY_REGISTRY,
                    Some(path.to_str().expect("path")),
                ),
                ("PETALTONGUE_WEB_PORT", Some("4300")),
            ],
            || {
                let resolved = NetworkDefaults::resolve();
                assert_eq!(
                    resolved.web_port, 4200,
                    "registry should override env and const"
                );
            },
        );

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(dir);
    }

    #[test]
    fn from_env_overrides_const_fallback() {
        env_test_helpers::with_env_var("PETALTONGUE_WEB_PORT", "4400", || {
            let defaults = NetworkDefaults::from_env();
            assert_eq!(defaults.web_port, 4400);
        });
    }
}
