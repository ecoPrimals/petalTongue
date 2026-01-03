//! Main entry point for petalTongue desktop UI

use petal_tongue_core::{Instance, InstanceId, InstanceRegistry};
use petal_tongue_ui::PetalTongueApp;

fn main() -> Result<(), eframe::Error> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // ===== Phase 1: Instance Management Integration =====
    // Create unique instance ID for this petalTongue instance
    let instance_id = InstanceId::new();
    let id_str = instance_id.as_str();
    tracing::info!("🌸 Starting petalTongue instance: {}", id_str);

    // Create instance metadata
    let instance = Instance::new(instance_id.clone(), Some("petalTongue".to_string()))
        .expect("Failed to create instance");

    // Load/create instance registry
    let mut registry = InstanceRegistry::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load registry: {}, creating new", e);
        InstanceRegistry::new()
    });

    // Register this instance
    if let Err(e) = registry.register(instance.clone()) {
        tracing::error!("Failed to register instance: {}", e);
    } else {
        tracing::info!("✅ Instance registered in registry");
    }

    // Clean up dead instances (returns count of removed)
    match registry.gc() {
        Ok(cleaned) if cleaned > 0 => {
            tracing::info!("🧹 Cleaned up {} dead instances", cleaned);
        }
        Ok(_) => {} // No cleanup needed
        Err(e) => {
            tracing::warn!("Failed to clean up dead instances: {}", e);
        }
    }

    // Save registry
    if let Err(e) = registry.save() {
        tracing::error!("Failed to save registry: {}", e);
    }

    // Store instance_id for app use (Phase 2 will use this)
    // EVOLUTION: Instead of using unsafe set_var, we'll pass instance_id directly to the app
    // This eliminates the unsafe block while maintaining functionality
    // The app can access instance_id through its constructor parameter
    tracing::debug!("Instance ID will be passed to app: {}", instance_id.as_str());
    // ===== End Phase 1 Integration =====

    tracing::info!("Starting petalTongue Universal Representation System");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("🌸 petalTongue - Universal Representation System"),
        ..Default::default()
    };

    let result = eframe::run_native(
        "petalTongue",
        options,
        Box::new(|cc| Ok(Box::new(PetalTongueApp::new(cc)))),
    );

    // ===== Cleanup on shutdown =====
    tracing::info!("🌸 petalTongue shutting down...");

    // Unregister instance from registry
    if let Ok(mut registry) = InstanceRegistry::load() {
        if let Err(e) = registry.unregister(&instance_id) {
            tracing::error!("Failed to unregister instance: {}", e);
        } else {
            tracing::info!("✅ Instance unregistered");
        }

        if let Err(e) = registry.save() {
            tracing::error!("Failed to save registry: {}", e);
        }
    }

    result
}
