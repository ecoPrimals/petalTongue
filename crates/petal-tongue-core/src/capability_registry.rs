// SPDX-License-Identifier: AGPL-3.0-or-later
//! Capability registry loader for `config/capability_registry.toml`.
//!
//! Parses capability domain metadata and `[network.fallbacks]` port hints.
//! Runtime discovery remains authoritative; registry values are fallbacks only.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Environment variable overriding the capability registry TOML path.
pub const ENV_CAPABILITY_REGISTRY: &str = "PETALTONGUE_CAPABILITY_REGISTRY";

/// Errors loading or parsing the capability registry.
#[derive(Debug, Error)]
pub enum CapabilityRegistryError {
    /// TOML parse failure or invalid structure.
    #[error("Failed to parse capability registry: {0}")]
    Parse(String),

    /// File I/O failure.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Config-driven capability registry with network fallback resolution.
///
/// Loaded from `config/capability_registry.toml`. Port and endpoint values
/// under `[network.fallbacks]` are hints for when capability discovery is
/// unavailable — not authoritative runtime endpoints.
#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    fallback_ports: HashMap<String, u16>,
    capability_ports: HashMap<String, u16>,
    fallback_strings: HashMap<String, String>,
    discovery_ports: Option<Vec<u16>>,
}

impl CapabilityRegistry {
    /// Load registry from the default path ([`default_registry_path`]).
    ///
    /// # Errors
    ///
    /// Returns an error if the registry file exists but cannot be read or parsed.
    pub fn load_default() -> Result<Self, CapabilityRegistryError> {
        Self::from_toml(Self::default_registry_path())
    }

    /// Load registry from a TOML file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or the TOML is invalid.
    pub fn from_toml(path: impl AsRef<Path>) -> Result<Self, CapabilityRegistryError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::parse_toml(&content)
    }

    /// Parse registry contents from TOML text (useful for tests).
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML is invalid.
    pub fn parse_toml(content: &str) -> Result<Self, CapabilityRegistryError> {
        let table: toml::Table = content
            .parse()
            .map_err(|e: toml::de::Error| CapabilityRegistryError::Parse(e.to_string()))?;

        let mut registry = Self::default();
        if let Some(network) = table.get("network").and_then(toml::Value::as_table) {
            if let Some(fallbacks) = network.get("fallbacks").and_then(toml::Value::as_table) {
                registry.ingest_fallback_table(fallbacks);
            }
            if let Some(cap_ports) = network
                .get("capability_ports")
                .and_then(toml::Value::as_table)
            {
                registry.ingest_capability_ports(cap_ports);
            }
        }
        Ok(registry)
    }

    /// Resolve a named fallback port from `[network.fallbacks]`.
    #[must_use]
    pub fn fallback_port(&self, key: &str) -> Option<u16> {
        self.fallback_ports.get(key).copied()
    }

    /// Resolve a capability-keyed fallback port from `[network.capability_ports]`.
    #[must_use]
    pub fn capability_port(&self, capability: &str) -> Option<u16> {
        self.capability_ports.get(capability).copied()
    }

    /// Resolve a named fallback string from `[network.fallbacks]`.
    #[must_use]
    pub fn fallback_string(&self, key: &str) -> Option<&str> {
        self.fallback_strings.get(key).map(String::as_str)
    }

    /// Discovery port list from `[network.fallbacks].discovery_ports`.
    #[must_use]
    pub fn fallback_discovery_ports(&self) -> Option<&[u16]> {
        self.discovery_ports.as_deref()
    }

    /// Default path: env override or `config/capability_registry.toml`.
    #[must_use]
    pub fn default_registry_path() -> PathBuf {
        std::env::var(ENV_CAPABILITY_REGISTRY).map_or_else(
            |_| PathBuf::from("config/capability_registry.toml"),
            PathBuf::from,
        )
    }

    fn ingest_fallback_table(&mut self, table: &toml::Table) {
        for (key, value) in table {
            if key == "discovery_ports" {
                if let Some(ports) = parse_port_list(value) {
                    self.discovery_ports = Some(ports);
                }
                continue;
            }
            if let Some(port) = parse_port_value(value) {
                self.fallback_ports.insert(key.clone(), port);
            } else if let Some(text) = value.as_str() {
                self.fallback_strings.insert(key.clone(), text.to_owned());
            }
        }
    }

    fn ingest_capability_ports(&mut self, table: &toml::Table) {
        for (capability, value) in table {
            if let Some(port) = parse_port_value(value) {
                self.capability_ports.insert(capability.clone(), port);
            }
        }
    }
}

fn parse_port_value(value: &toml::Value) -> Option<u16> {
    value.as_integer().and_then(|i| u16::try_from(i).ok())
}

fn parse_port_list(value: &toml::Value) -> Option<Vec<u16>> {
    value.as_array().map(|arr| {
        arr.iter()
            .filter_map(parse_port_value)
            .collect::<Vec<u16>>()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[network.fallbacks]
web_port = 3100
headless_port = 8180
discovery_ports = [8180, 3100]
gpu_compute_endpoint = "tarpc://localhost:9100"

[network.capability_ports]
visualization = 3100
discovery = 8180
"#;

    #[test]
    fn parse_network_fallbacks() {
        let registry = CapabilityRegistry::parse_toml(SAMPLE).expect("parse");
        assert_eq!(registry.fallback_port("web_port"), Some(3100));
        assert_eq!(registry.fallback_port("headless_port"), Some(8180));
        assert_eq!(
            registry.fallback_string("gpu_compute_endpoint"),
            Some("tarpc://localhost:9100")
        );
        assert_eq!(
            registry.fallback_discovery_ports(),
            Some([8180_u16, 3100].as_slice())
        );
        assert_eq!(registry.capability_port("visualization"), Some(3100));
        assert_eq!(registry.capability_port("discovery"), Some(8180));
    }

    #[test]
    fn empty_registry_has_no_fallbacks() {
        let registry = CapabilityRegistry::default();
        assert_eq!(registry.fallback_port("web_port"), None);
        assert!(registry.fallback_discovery_ports().is_none());
    }
}
