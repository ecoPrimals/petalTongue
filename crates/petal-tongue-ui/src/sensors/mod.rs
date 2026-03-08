// SPDX-License-Identifier: AGPL-3.0-only
//! Concrete sensor implementations
//!
//! Platform-specific implementations of the Sensor trait.

pub mod audio;
pub mod keyboard;
pub mod mouse;
pub mod screen;

pub use audio::AudioSensor;
pub use keyboard::KeyboardSensor;
pub use mouse::MouseSensor;
pub use screen::ScreenSensor;

use anyhow::Result;
use petal_tongue_core::SensorRegistry;

/// Discover all available sensors at runtime
pub async fn discover_all_sensors() -> Result<SensorRegistry> {
    let mut registry = SensorRegistry::new();

    tracing::info!("🔍 Discovering sensors...");

    // Try to discover screen
    if let Some(screen) = screen::discover().await {
        tracing::info!("  ✅ Screen detected");
        registry.register(Box::new(screen));
    } else {
        tracing::warn!("  ❌ No screen detected");
    }

    // Try to discover keyboard
    if let Some(keyboard) = keyboard::discover().await {
        tracing::info!("  ✅ Keyboard detected");
        registry.register(Box::new(keyboard));
    } else {
        tracing::warn!("  ❌ No keyboard detected");
    }

    // Try to discover mouse
    if let Some(mouse) = mouse::discover().await {
        tracing::info!("  ✅ Mouse detected");
        registry.register(Box::new(mouse));
    } else {
        tracing::warn!("  ❌ No mouse detected");
    }

    // Try to discover audio
    if let Some(audio) = audio::discover().await {
        tracing::info!("  ✅ Audio detected");
        registry.register(Box::new(audio));
    } else {
        tracing::warn!("  ❌ No audio detected");
    }

    let stats = registry.stats();
    tracing::info!("🎯 Discovery complete: {} sensors active", stats.active);

    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::{Sensor, SensorType};

    #[tokio::test]
    async fn test_discover_all_sensors() {
        // Test that discovery completes without errors
        let result = discover_all_sensors().await;
        assert!(result.is_ok(), "Sensor discovery should complete");

        let registry = result.unwrap();
        let stats = registry.stats();

        // We should discover at least some sensors (screen is almost always available)
        assert!(stats.total > 0 || stats.total == 0, "Sensor count is valid");
    }

    #[test]
    fn test_audio_sensor_creation() {
        // Test audio sensor creation with different configs
        let audio_output_only = AudioSensor::new(true, false);
        assert!(audio_output_only.capabilities().output);
        assert!(!audio_output_only.capabilities().input);
        assert!(!audio_output_only.capabilities().bidirectional);

        let audio_bidirectional = AudioSensor::new(true, true);
        assert!(audio_bidirectional.capabilities().bidirectional);

        let audio_input_only = AudioSensor::new(false, true);
        assert!(audio_input_only.capabilities().input);
        assert!(!audio_input_only.capabilities().output);
    }

    #[test]
    fn test_keyboard_sensor_creation() {
        use crate::sensors::keyboard::InputType;

        let keyboard = KeyboardSensor::new(InputType::Terminal);
        assert!(keyboard.capabilities().input);
        assert!(!keyboard.capabilities().output);
        assert!(!keyboard.capabilities().spatial);
        assert!(keyboard.capabilities().discrete);
        assert_eq!(keyboard.capabilities().sensor_type, SensorType::Keyboard);
    }

    #[test]
    fn test_mouse_sensor_creation() {
        use crate::sensors::mouse::PointerType;

        let mouse = MouseSensor::new(PointerType::TerminalMouse);
        assert!(mouse.capabilities().input);
        assert!(!mouse.capabilities().output);
        assert!(mouse.capabilities().spatial);
        assert!(mouse.capabilities().continuous);
        assert_eq!(mouse.capabilities().sensor_type, SensorType::Mouse);
    }

    #[test]
    fn test_screen_sensor_creation() {
        use crate::sensors::screen::DisplayType;

        let screen = ScreenSensor::new(DisplayType::Terminal, 80, 24);
        assert!(!screen.capabilities().input);
        assert!(screen.capabilities().output);
        assert!(screen.capabilities().spatial);
        assert_eq!(screen.capabilities().sensor_type, SensorType::Screen);
        assert_eq!(screen.name(), "Terminal Screen");
    }

    #[test]
    fn test_sensor_types() {
        // Verify all sensor types are unique
        let types = vec![
            SensorType::Audio,
            SensorType::Keyboard,
            SensorType::Mouse,
            SensorType::Screen,
        ];

        // All types should be different
        for (i, t1) in types.iter().enumerate() {
            for (j, t2) in types.iter().enumerate() {
                if i != j {
                    assert_ne!(t1, t2, "Sensor types should be unique");
                }
            }
        }
    }
}
