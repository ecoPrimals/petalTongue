//! Concrete sensor implementations
//!
//! Platform-specific implementations of the Sensor trait.

pub mod screen;
pub mod keyboard;
pub mod mouse;
pub mod audio;

pub use screen::ScreenSensor;
pub use keyboard::KeyboardSensor;
pub use mouse::MouseSensor;
pub use audio::AudioSensor;

use anyhow::Result;
use petal_tongue_core::{SensorRegistry, SensorType};

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

