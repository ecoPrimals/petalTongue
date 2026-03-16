// SPDX-License-Identifier: AGPL-3.0-or-later
//! Visualization capability definitions
//!
//! These capabilities define what data a primal must provide to support
//! visualization in petalTongue.

/// Visualization capabilities that primals can provide
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisualizationCapability {
    /// Provides list of discovered primals
    ///
    /// Required endpoint: `GET /api/v1/primals`
    PrimalProvider,

    /// Provides topology/connections between primals
    ///
    /// Required endpoint: `GET /api/v1/topology`
    TopologyProvider,

    /// Provides real-time telemetry stream
    ///
    /// Required endpoint: `WS /api/v1/telemetry`
    TelemetryProvider,

    /// Provides health monitoring
    ///
    /// Required endpoint: `GET /api/v1/health`
    HealthProvider,
}

impl VisualizationCapability {
    /// Get the capability string for discovery
    ///
    /// This is what we query for in mDNS or capability registries.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::PrimalProvider => "visualization.primal-provider",
            Self::TopologyProvider => "visualization.topology-provider",
            Self::TelemetryProvider => "visualization.telemetry-provider",
            Self::HealthProvider => "visualization.health-provider",
        }
    }

    /// Get all visualization capabilities
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::PrimalProvider,
            Self::TopologyProvider,
            Self::TelemetryProvider,
            Self::HealthProvider,
        ]
    }
}

impl std::fmt::Display for VisualizationCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_strings() {
        assert_eq!(
            VisualizationCapability::PrimalProvider.as_str(),
            "visualization.primal-provider"
        );
        assert_eq!(
            VisualizationCapability::TopologyProvider.as_str(),
            "visualization.topology-provider"
        );
    }

    #[test]
    fn test_all_capabilities() {
        let caps = VisualizationCapability::all();
        assert_eq!(caps.len(), 4);
    }

    #[test]
    fn test_display() {
        let cap = VisualizationCapability::PrimalProvider;
        assert_eq!(format!("{cap}"), "visualization.primal-provider");
    }

    #[test]
    fn test_telemetry_and_health_capability_strings() {
        assert_eq!(
            VisualizationCapability::TelemetryProvider.as_str(),
            "visualization.telemetry-provider"
        );
        assert_eq!(
            VisualizationCapability::HealthProvider.as_str(),
            "visualization.health-provider"
        );
    }
}
