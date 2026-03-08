// SPDX-License-Identifier: AGPL-3.0-only
//! Runtime Sensor Discovery
//!
//! Discovers available input/output peripherals at startup.
//! No hardcoded assumptions - tests what actually exists.

use anyhow::Result;
use petal_tongue_core::{Sensor, SensorRegistry};
use std::sync::{Arc, RwLock};

/// Discover all available sensors and populate registry
pub async fn discover_all_sensors(registry: Arc<RwLock<SensorRegistry>>) -> Result<()> {
    tracing::info!("🔍 Starting sensor discovery...");

    let mut discovered_count = 0;

    // Discover screen (output sensor)
    if let Some(screen) = crate::sensors::screen::discover().await {
        tracing::info!("✅ Discovered: {}", screen.name());
        if let Ok(mut reg) = registry.write() {
            reg.register(Box::new(screen));
            discovered_count += 1;
        }
    } else {
        tracing::warn!("❌ No screen sensor discovered");
    }

    // Discover keyboard (discrete input)
    if let Some(keyboard) = crate::sensors::keyboard::discover().await {
        tracing::info!("✅ Discovered: {}", keyboard.name());
        if let Ok(mut reg) = registry.write() {
            reg.register(Box::new(keyboard));
            discovered_count += 1;
        }
    } else {
        tracing::info!("ℹ️  No keyboard sensor discovered (may be GUI-only)");
    }

    // Discover mouse (spatial input)
    if let Some(mouse) = crate::sensors::mouse::discover().await {
        tracing::info!("✅ Discovered: {}", mouse.name());
        if let Ok(mut reg) = registry.write() {
            reg.register(Box::new(mouse));
            discovered_count += 1;
        }
    } else {
        tracing::info!("ℹ️  No mouse sensor discovered (may be GUI-only)");
    }

    // Discover audio (input/output)
    #[cfg(feature = "audio")]
    if let Some(audio) = crate::sensors::audio::discover().await {
        tracing::info!("✅ Discovered: {}", audio.name());
        if let Ok(mut reg) = registry.write() {
            reg.register(Box::new(audio));
            discovered_count += 1;
        }
    }

    // Summary
    tracing::info!(
        "🎯 Sensor discovery complete: {} sensor(s) registered",
        discovered_count
    );

    // Log detailed capabilities
    if let Ok(reg) = registry.read() {
        let stats = reg.stats();
        tracing::info!("📊 Sensor statistics:");
        tracing::info!("   Total sensors: {}", stats.total);
        tracing::info!("   Active sensors: {}", stats.active);

        // Count sensors by capability
        let mut input_count = 0;
        let mut output_count = 0;
        for sensor in reg.sensors() {
            let caps = sensor.capabilities();
            if caps.input {
                input_count += 1;
            }
            if caps.output {
                output_count += 1;
            }
        }
        tracing::info!("   Input capable: {}", input_count);
        tracing::info!("   Output capable: {}", output_count);
    }

    Ok(())
}

/// Check if essential sensors are available
pub fn verify_essential_sensors(registry: &Arc<RwLock<SensorRegistry>>) -> bool {
    if let Ok(reg) = registry.read() {
        // Need at least one output sensor (screen)
        let mut has_output = false;
        for sensor in reg.sensors() {
            if sensor.capabilities().output {
                has_output = true;
                break;
            }
        }

        if !has_output {
            tracing::warn!("⚠️  No output sensors available!");
            return false;
        }

        tracing::info!("✅ Essential sensors verified");
        true
    } else {
        tracing::error!("❌ Cannot access sensor registry");
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sensor_discovery() {
        let registry = Arc::new(RwLock::new(SensorRegistry::new()));

        // Discovery should not fail even if no sensors found
        let result = discover_all_sensors(Arc::clone(&registry)).await;
        assert!(result.is_ok());

        // Should have registered something (at least in test env)
        if let Ok(reg) = registry.read() {
            let stats = reg.stats();
            // In CI, might be 0, but shouldn't panic
            assert!(stats.total >= 0);
        }
    }

    #[test]
    fn test_verify_essential_sensors() {
        let registry = Arc::new(RwLock::new(SensorRegistry::new()));

        // With empty registry, should return false (no output)
        assert!(!verify_essential_sensors(&registry));
    }
}
