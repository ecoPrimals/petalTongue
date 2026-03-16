// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sensor registry: discovery and management of sensors.

use std::time::Instant;

use super::types::{Sensor, SensorCapability, SensorEvent, SensorType};

/// Sensor registry - discovers and manages all sensors
pub struct SensorRegistry {
    sensors: Vec<Box<dyn Sensor>>,
    last_poll: Option<Instant>,
}

impl SensorRegistry {
    /// Create new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            sensors: Vec::new(),
            last_poll: None,
        }
    }

    /// Register a sensor
    pub fn register(&mut self, sensor: Box<dyn Sensor>) {
        self.sensors.push(sensor);
    }

    /// Get all sensors
    #[must_use]
    pub fn sensors(&self) -> &[Box<dyn Sensor>] {
        &self.sensors
    }

    /// Get sensors by type
    #[must_use]
    pub fn sensors_by_type(&self, sensor_type: SensorType) -> Vec<&dyn Sensor> {
        self.sensors
            .iter()
            .filter(|s| s.capabilities().sensor_type == sensor_type)
            .map(std::convert::AsRef::as_ref)
            .collect()
    }

    /// Check if we have a sensor with specific capability
    #[must_use]
    pub fn has_capability(&self, capability: SensorCapability) -> bool {
        self.sensors
            .iter()
            .any(|s| s.capabilities().has_capability(capability))
    }

    /// Poll all sensors for events
    ///
    /// # Errors
    ///
    /// Does not return errors; individual sensor poll failures are logged and skipped.
    pub async fn poll_all(&mut self) -> anyhow::Result<Vec<SensorEvent>> {
        let mut all_events = Vec::new();

        for sensor in &mut self.sensors {
            if sensor.is_available() {
                match sensor.poll_events().await {
                    Ok(events) => all_events.extend(events),
                    Err(e) => {
                        tracing::warn!("Error polling sensor {}: {}", sensor.name(), e);
                    }
                }
            }
        }

        self.last_poll = Some(Instant::now());
        Ok(all_events)
    }

    /// Get count of active sensors
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.sensors.iter().filter(|s| s.is_available()).count()
    }

    /// Get sensor statistics
    #[must_use]
    pub fn stats(&self) -> SensorStats {
        let total = self.sensors.len();
        let active = self.active_count();
        let has_input = self.has_capability(SensorCapability::Input);
        let has_output = self.has_capability(SensorCapability::Output);
        let has_bidirectional = self.has_capability(SensorCapability::Bidirectional);

        SensorStats {
            total,
            active,
            has_input,
            has_output,
            has_bidirectional,
            last_poll: self.last_poll,
        }
    }
}

impl Default for SensorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Sensor statistics for runtime tracking
#[expect(
    missing_docs,
    reason = "field names are self-documenting for runtime sensor statistics"
)]
#[derive(Debug, Clone)]
pub struct SensorStats {
    pub total: usize,
    pub active: usize,
    pub has_input: bool,
    pub has_output: bool,
    pub has_bidirectional: bool,
    pub last_poll: Option<Instant>,
}

#[cfg(test)]
pub mod mock_sensor {
    use crate::sensor::Sensor;
    use crate::sensor::types::{SensorCapabilities, SensorEvent};
    use async_trait::async_trait;
    use std::time::Instant;

    /// Mock sensor for testing
    pub struct MockSensor {
        name: String,
        capabilities: SensorCapabilities,
        available: bool,
    }

    impl MockSensor {
        pub fn new(name: &str, capabilities: SensorCapabilities) -> Self {
            Self {
                name: name.to_string(),
                capabilities,
                available: true,
            }
        }
    }

    #[async_trait]
    impl Sensor for MockSensor {
        fn capabilities(&self) -> &SensorCapabilities {
            &self.capabilities
        }

        fn is_available(&self) -> bool {
            self.available
        }

        async fn poll_events(&mut self) -> anyhow::Result<Vec<SensorEvent>> {
            Ok(Vec::new())
        }

        fn last_activity(&self) -> Option<Instant> {
            None
        }

        fn name(&self) -> &str {
            &self.name
        }
    }
}
