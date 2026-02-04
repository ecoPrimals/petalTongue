//! Type conversions for scenarios
//!
//! Converts scenario definitions to core types used by the visualization system.

use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, PropertyValue};
use std::collections::HashMap;

use crate::scenario::ecosystem::PrimalDefinition;
use crate::scenario::types::Scenario;

impl Scenario {
    /// Convert scenario primals to PrimalInfo for the graph system
    ///
    /// Maps the scenario's primal definitions to the core PrimalInfo type
    /// used by the visualization and graph systems.
    pub fn to_primal_infos(&self) -> Vec<PrimalInfo> {
        self.ecosystem
            .primals
            .iter()
            .map(|p| p.to_primal_info())
            .collect()
    }
}

impl PrimalDefinition {
    /// Convert a scenario primal to core PrimalInfo
    pub fn to_primal_info(&self) -> PrimalInfo {
        let mut properties = HashMap::new();

        // Map basic properties
        properties.insert(
            "family".to_string(),
            PropertyValue::String(self.family.clone()),
        );
        properties.insert(
            "scenario_type".to_string(),
            PropertyValue::String(self.primal_type.clone()),
        );
        properties.insert(
            "status".to_string(),
            PropertyValue::String(self.status.clone()),
        );
        properties.insert(
            "health".to_string(),
            PropertyValue::Number(f64::from(self.health)),
        );
        properties.insert(
            "confidence".to_string(),
            PropertyValue::Number(f64::from(self.confidence)),
        );

        // Add position
        properties.insert(
            "x".to_string(),
            PropertyValue::Number(f64::from(self.position.x)),
        );
        properties.insert(
            "y".to_string(),
            PropertyValue::Number(f64::from(self.position.y)),
        );

        // Add capabilities as JSON array string
        if !self.capabilities.is_empty() {
            let caps_json = serde_json::to_string(&self.capabilities).unwrap_or_default();
            properties.insert("capabilities".to_string(), PropertyValue::String(caps_json));
        }

        // Add metrics to properties
        if self.metrics.cpu_percent > 0.0 {
            properties.insert(
                "cpu_percent".to_string(),
                PropertyValue::Number(f64::from(self.metrics.cpu_percent)),
            );
        }
        if self.metrics.memory_mb > 0 {
            properties.insert(
                "memory_mb".to_string(),
                PropertyValue::Number(self.metrics.memory_mb as f64),
            );
        }
        if self.metrics.uptime_seconds > 0 {
            properties.insert(
                "uptime_seconds".to_string(),
                PropertyValue::Number(self.metrics.uptime_seconds as f64),
            );
        }
        if self.metrics.requests_per_second > 0 {
            properties.insert(
                "requests_per_second".to_string(),
                PropertyValue::Number(self.metrics.requests_per_second as f64),
            );
        }
        if self.metrics.active_primals > 0 {
            properties.insert(
                "active_primals".to_string(),
                PropertyValue::Number(self.metrics.active_primals as f64),
            );
        }

        // Add proprioception data if available
        if let Some(ref proprio) = self.proprioception {
            // Self-awareness
            properties.insert(
                "knows_about".to_string(),
                PropertyValue::Number(proprio.self_awareness.knows_about as f64),
            );
            properties.insert(
                "can_coordinate".to_string(),
                PropertyValue::Boolean(proprio.self_awareness.can_coordinate),
            );
            properties.insert(
                "has_security".to_string(),
                PropertyValue::Boolean(proprio.self_awareness.has_security),
            );
            properties.insert(
                "has_discovery".to_string(),
                PropertyValue::Boolean(proprio.self_awareness.has_discovery),
            );
            properties.insert(
                "has_compute".to_string(),
                PropertyValue::Boolean(proprio.self_awareness.has_compute),
            );

            // Motor
            properties.insert(
                "can_deploy".to_string(),
                PropertyValue::Boolean(proprio.motor.can_deploy),
            );
            properties.insert(
                "can_execute_graphs".to_string(),
                PropertyValue::Boolean(proprio.motor.can_execute_graphs),
            );
            properties.insert(
                "can_coordinate_primals".to_string(),
                PropertyValue::Boolean(proprio.motor.can_coordinate_primals),
            );

            // Sensory
            properties.insert(
                "active_sockets".to_string(),
                PropertyValue::Number(proprio.sensory.active_sockets as f64),
            );
            if let Some(ref last_scan) = proprio.sensory.last_scan {
                properties.insert(
                    "last_scan".to_string(),
                    PropertyValue::String(last_scan.clone()),
                );
            }
        }

        // Determine health status from scenario health value
        let health = if self.health >= 90 {
            PrimalHealthStatus::Healthy
        } else if self.health >= 50 {
            PrimalHealthStatus::Warning
        } else {
            PrimalHealthStatus::Critical
        };

        PrimalInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            family_id: Some(self.family.clone()), // Use family as family_id
            primal_type: self.primal_type.clone(),
            endpoint: format!("scenario://{}", self.id), // Synthetic endpoint for scenarios
            capabilities: self.capabilities.clone(),
            health,
            trust_level: Some(self.confidence), // Use confidence as trust level
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            endpoints: None,
            metadata: None,
            properties,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::ecosystem::{Position, PrimalMetrics as ScenarioMetrics};

    #[test]
    fn test_primal_definition_to_info() {
        let primal = PrimalDefinition {
            id: "test-1".to_string(),
            name: "Test Primal".to_string(),
            primal_type: "TestType".to_string(),
            family: "TestFamily".to_string(),
            status: "healthy".to_string(),
            health: 100,
            confidence: 95,
            position: Position { x: 100.0, y: 200.0 },
            capabilities: vec!["test-cap".to_string()],
            metrics: ScenarioMetrics {
                cpu_percent: 25.5,
                memory_mb: 512,
                uptime_seconds: 3600,
                ..Default::default()
            },
            proprioception: None,
        };

        let info = primal.to_primal_info();

        assert_eq!(info.id, "test-1");
        assert_eq!(info.name, "Test Primal");
        assert_eq!(info.primal_type, "TestType");
        assert_eq!(info.capabilities, vec!["test-cap".to_string()]);
        assert!(matches!(info.health, PrimalHealthStatus::Healthy));

        // Check properties
        let props = &info.properties;
        assert_eq!(
            props
                .get("cpu_percent")
                .and_then(|v: &PropertyValue| v.as_number()),
            Some(25.5)
        );
    }

    #[test]
    fn test_scenario_to_primal_infos() {
        let scenario = Scenario {
            name: "Test".to_string(),
            description: "Test scenario".to_string(),
            version: "1.0.0".to_string(),
            mode: "test".to_string(),
            ui_config: Default::default(),
            ecosystem: crate::scenario::ecosystem::Ecosystem {
                primals: vec![
                    PrimalDefinition {
                        id: "p1".to_string(),
                        name: "Primal 1".to_string(),
                        primal_type: "Type1".to_string(),
                        family: "Family1".to_string(),
                        status: "healthy".to_string(),
                        health: 100,
                        confidence: 90,
                        position: Position { x: 0.0, y: 0.0 },
                        capabilities: vec![],
                        metrics: Default::default(),
                        proprioception: None,
                    },
                    PrimalDefinition {
                        id: "p2".to_string(),
                        name: "Primal 2".to_string(),
                        primal_type: "Type2".to_string(),
                        family: "Family2".to_string(),
                        status: "healthy".to_string(),
                        health: 95,
                        confidence: 85,
                        position: Position { x: 100.0, y: 100.0 },
                        capabilities: vec![],
                        metrics: Default::default(),
                        proprioception: None,
                    },
                ],
            },
            neural_api: Default::default(),
            sensory_config: Default::default(),
        };

        let infos = scenario.to_primal_infos();
        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].id, "p1");
        assert_eq!(infos[1].id, "p2");
    }
}
