// SPDX-License-Identifier: AGPL-3.0-or-later

/// Describes what a sensor can do
#[derive(Debug, Clone)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "capability flags are naturally boolean"
)]
pub struct SensorCapabilities {
    /// Type of sensor
    pub sensor_type: SensorType,

    /// Can receive input
    pub input: bool,

    /// Can provide output
    pub output: bool,

    /// Provides spatial data (x, y coordinates)
    pub spatial: bool,

    /// Provides temporal data (timing, rhythm)
    pub temporal: bool,

    /// Continuous values (analog)
    pub continuous: bool,

    /// Discrete events (digital)
    pub discrete: bool,

    /// Bidirectional (input AND output)
    pub bidirectional: bool,
}

impl SensorCapabilities {
    /// Check if sensor has a specific capability
    #[must_use]
    pub const fn has_capability(&self, capability: SensorCapability) -> bool {
        match capability {
            SensorCapability::Input => self.input,
            SensorCapability::Output => self.output,
            SensorCapability::Spatial => self.spatial,
            SensorCapability::Temporal => self.temporal,
            SensorCapability::Continuous => self.continuous,
            SensorCapability::Discrete => self.discrete,
            SensorCapability::Bidirectional => self.bidirectional,
        }
    }
}

/// Specific capabilities to query for - sensor characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorCapability {
    /// Accepts input from user/environment
    Input,
    /// Provides output to user/environment
    Output,
    /// Tracks spatial position/movement
    Spatial,
    /// Tracks temporal changes/events
    Temporal,
    /// Provides continuous stream of data
    Continuous,
    /// Provides discrete events/readings
    Discrete,
    /// Supports bidirectional communication
    Bidirectional,
}

/// Types of sensors (discovered at runtime)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensorType {
    /// Display output (terminal, framebuffer, surface)
    Screen,

    /// Discrete input device (keys, buttons)
    Keyboard,

    /// Spatial input device (pointing, clicking)
    Mouse,

    /// Audio input/output (microphone, speaker)
    Audio,

    /// Visual input (camera, image sensor)
    Camera,

    /// Motion detection (accelerometer, gyroscope)
    Motion,

    /// Location awareness (GPS, network location)
    Location,

    /// Biometric sensor (heart rate, temperature, etc.)
    Biometric,

    /// Environmental sensor (temperature, humidity, etc.)
    Environmental,

    /// Network sensor (primal discovery, health)
    Network,

    /// Touchscreen / pressure surface
    Touch,

    /// Eye/gaze tracking device
    EyeTracker,

    /// Binary switch device (sip-and-puff, head switch, BCI binary)
    Switch,

    /// Agentic AI / machine interactor (Squirrel, API client)
    Agent,

    /// Unknown sensor type
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_capability_returns_correct_booleans() {
        let caps = SensorCapabilities {
            sensor_type: SensorType::Mouse,
            input: true,
            output: false,
            spatial: true,
            temporal: false,
            continuous: true,
            discrete: true,
            bidirectional: false,
        };
        assert!(caps.has_capability(SensorCapability::Input));
        assert!(!caps.has_capability(SensorCapability::Output));
        assert!(caps.has_capability(SensorCapability::Spatial));
        assert!(!caps.has_capability(SensorCapability::Temporal));
        assert!(caps.has_capability(SensorCapability::Continuous));
        assert!(caps.has_capability(SensorCapability::Discrete));
        assert!(!caps.has_capability(SensorCapability::Bidirectional));
    }
}
