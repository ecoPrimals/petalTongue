// SPDX-License-Identifier: AGPL-3.0-only
//! Proprioception (SAME DAVE) data structures
//!
//! SAME DAVE is neuroanatomy, not AI — the channel model maps specialized
//! unidirectional pathways analogous to the spinal cord's dorsal/ventral roots.
//! Sensory Afferent pathways carry input TO the proprioception core. Motor
//! Efferent pathways carry commands FROM the core to effectors. Classification
//! nodes along each channel act like nodes of Ranvier, enabling saltatory
//! signal routing.
//!
//! The types in this module are **snapshots** of channel and system state,
//! populated by the Neural API or local introspection and visualized by the UI.

use crate::channel::ChannelSnapshot;
use serde::{Deserialize, Serialize};

/// Complete proprioception data from Neural API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProprioceptionData {
    /// When this proprioception snapshot was taken
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Family ID this system belongs to
    pub family_id: String,

    /// Overall health assessment
    pub health: HealthData,

    /// System's confidence in its state (0.0-100.0)
    pub confidence: f32,

    /// Sensory: What the system perceives (afferent channel state)
    pub sensory: SensoryData,

    /// Awareness: What the system knows about its ecosystem
    pub self_awareness: SelfAwarenessData,

    /// Motor: What the system can do (efferent channel state)
    pub motor: MotorData,

    /// Afferent (sensory) channel health snapshots
    #[serde(default)]
    pub afferent_channels: Vec<ChannelSnapshot>,

    /// Efferent (motor) channel health snapshots
    #[serde(default)]
    pub efferent_channels: Vec<ChannelSnapshot>,
}

/// Health assessment data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthData {
    /// Health percentage (0.0-100.0)
    pub percentage: f32,

    /// Qualitative health status
    pub status: HealthStatus,
}

/// Qualitative health status levels
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// System is fully operational (100%)
    Healthy,

    /// System is partially operational (50-99%)
    Degraded,

    /// System is critically impaired (<50%)
    Critical,
}

impl HealthStatus {
    /// Get color for UI rendering
    #[must_use]
    pub const fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Healthy => (34, 197, 94),  // green-500
            Self::Degraded => (234, 179, 8), // yellow-500
            Self::Critical => (239, 68, 68), // red-500
        }
    }

    /// Get emoji representation
    #[must_use]
    pub const fn emoji(&self) -> &'static str {
        match self {
            Self::Healthy => "💚",
            Self::Degraded => "💛",
            Self::Critical => "❤️",
        }
    }
}

/// Sensory (afferent) snapshot: What the system perceives via input channels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensoryData {
    /// Number of active Unix sockets detected
    pub active_sockets: u32,

    /// When the last socket scan occurred
    pub last_scan: chrono::DateTime<chrono::Utc>,
}

/// Self-awareness data: What the system knows about its ecosystem.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelfAwarenessData {
    /// Number of primals the system knows about
    pub knows_about: u32,

    /// Can coordinate multiple primals
    pub can_coordinate: bool,

    /// Has security capabilities (`BearDog`)
    pub has_security: bool,

    /// Has discovery capabilities (Songbird)
    pub has_discovery: bool,

    /// Has compute capabilities (`ToadStool`)
    pub has_compute: bool,
}

/// Motor (efferent) snapshot: What the system can do via output channels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MotorData {
    /// Can deploy new primals
    pub can_deploy: bool,

    /// Can execute graphs
    pub can_execute_graphs: bool,

    /// Can coordinate primals
    pub can_coordinate_primals: bool,
}

impl ProprioceptionData {
    /// Create a default/empty proprioception data (for graceful fallback)
    #[must_use]
    pub fn empty(family_id: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            timestamp: now,
            family_id: family_id.into(),
            health: HealthData {
                percentage: 0.0,
                status: HealthStatus::Critical,
            },
            confidence: 0.0,
            sensory: SensoryData {
                active_sockets: 0,
                last_scan: now,
            },
            self_awareness: SelfAwarenessData {
                knows_about: 0,
                can_coordinate: false,
                has_security: false,
                has_discovery: false,
                has_compute: false,
            },
            motor: MotorData {
                can_deploy: false,
                can_execute_graphs: false,
                can_coordinate_primals: false,
            },
            afferent_channels: Vec::new(),
            efferent_channels: Vec::new(),
        }
    }

    /// Check if the system is healthy (>= 80%)
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        self.health.percentage >= 80.0
    }

    /// Check if the system is confident (>= 80%)
    #[must_use]
    pub const fn is_confident(&self) -> bool {
        self.confidence >= 80.0
    }

    /// Get a human-readable summary string
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{} {} | {} primals | {:.0}% confident",
            self.health.status.emoji(),
            self.health.status,
            self.self_awareness.knows_about,
            self.confidence
        )
    }

    /// Get time since this proprioception was captured
    #[must_use]
    pub fn age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.timestamp
    }

    /// Check if this data is stale (> 30 seconds old)
    #[must_use]
    pub fn is_stale(&self) -> bool {
        self.age().num_seconds() > 30
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "Healthy"),
            Self::Degraded => write!(f, "Degraded"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_proprioception() {
        let data = ProprioceptionData::empty("test-family");
        assert_eq!(data.family_id, "test-family");
        assert_eq!(data.health.status, HealthStatus::Critical);
        assert!(!data.is_healthy());
        assert!(!data.is_confident());
    }

    #[test]
    fn test_health_status_colors() {
        assert_eq!(HealthStatus::Healthy.color_rgb(), (34, 197, 94));
        assert_eq!(HealthStatus::Degraded.color_rgb(), (234, 179, 8));
        assert_eq!(HealthStatus::Critical.color_rgb(), (239, 68, 68));
    }

    #[test]
    fn test_health_status_emoji() {
        assert_eq!(HealthStatus::Healthy.emoji(), "💚");
        assert_eq!(HealthStatus::Degraded.emoji(), "💛");
        assert_eq!(HealthStatus::Critical.emoji(), "❤️");
    }

    #[test]
    fn test_proprioception_is_healthy() {
        let mut data = ProprioceptionData::empty("test");

        data.health.percentage = 85.0;
        assert!(data.is_healthy());

        data.health.percentage = 79.9;
        assert!(!data.is_healthy());
    }

    #[test]
    fn test_proprioception_is_confident() {
        let mut data = ProprioceptionData::empty("test");

        data.confidence = 90.0;
        assert!(data.is_confident());

        data.confidence = 75.0;
        assert!(!data.is_confident());
    }

    #[test]
    fn test_proprioception_summary() {
        let mut data = ProprioceptionData::empty("test");
        data.health.status = HealthStatus::Healthy;
        data.self_awareness.knows_about = 3;
        data.confidence = 95.5;

        let summary = data.summary();
        assert!(summary.contains("💚"));
        assert!(summary.contains("Healthy"));
        assert!(summary.contains("3 primals"));
        assert!(summary.contains("96% confident"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let data = ProprioceptionData::empty("test");
        let json = serde_json::to_string(&data).expect("serialize");
        let decoded: ProprioceptionData = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.family_id, "test");
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "Healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "Degraded");
        assert_eq!(HealthStatus::Critical.to_string(), "Critical");
    }

    #[test]
    fn test_proprioception_is_stale_fresh() {
        let data = ProprioceptionData::empty("test");
        assert!(!data.is_stale());
    }

    #[test]
    fn test_proprioception_age() {
        let data = ProprioceptionData::empty("test");
        let age = data.age();
        assert!(age.num_seconds() >= 0);
    }

    #[test]
    fn test_sensory_data_fields() {
        let mut data = ProprioceptionData::empty("test");
        data.sensory.active_sockets = 5;
        assert_eq!(data.sensory.active_sockets, 5);
    }

    #[test]
    fn test_motor_data_fields() {
        let mut data = ProprioceptionData::empty("test");
        data.motor.can_deploy = true;
        data.motor.can_execute_graphs = true;
        assert!(data.motor.can_deploy);
        assert!(data.motor.can_execute_graphs);
    }

    #[test]
    fn test_self_awareness_data_fields() {
        let mut data = ProprioceptionData::empty("test");
        data.self_awareness.knows_about = 10;
        data.self_awareness.can_coordinate = true;
        assert_eq!(data.self_awareness.knows_about, 10);
        assert!(data.self_awareness.can_coordinate);
    }

    #[test]
    fn test_health_data_construction() {
        let health = HealthData {
            percentage: 75.5,
            status: HealthStatus::Degraded,
        };
        assert!((health.percentage - 75.5).abs() < f32::EPSILON);
        assert_eq!(health.status, HealthStatus::Degraded);
    }

    #[test]
    fn test_sensory_data_construction() {
        let now = chrono::Utc::now();
        let sensory = SensoryData {
            active_sockets: 3,
            last_scan: now,
        };
        assert_eq!(sensory.active_sockets, 3);
    }

    #[test]
    fn test_motor_data_construction() {
        let motor = MotorData {
            can_deploy: true,
            can_execute_graphs: true,
            can_coordinate_primals: true,
        };
        assert!(motor.can_deploy);
        assert!(motor.can_execute_graphs);
        assert!(motor.can_coordinate_primals);
    }

    #[test]
    fn test_self_awareness_construction() {
        let sa = SelfAwarenessData {
            knows_about: 5,
            can_coordinate: true,
            has_security: true,
            has_discovery: false,
            has_compute: true,
        };
        assert_eq!(sa.knows_about, 5);
        assert!(sa.has_security);
        assert!(!sa.has_discovery);
    }

    #[test]
    fn test_proprioception_empty_family_id_string() {
        let data = ProprioceptionData::empty(String::from("dynamic-id"));
        assert_eq!(data.family_id, "dynamic-id");
    }

    #[test]
    fn test_health_status_serde_lowercase() {
        let json = r#""healthy""#;
        let status: HealthStatus = serde_json::from_str(json).expect("parse");
        assert_eq!(status, HealthStatus::Healthy);
    }
}
