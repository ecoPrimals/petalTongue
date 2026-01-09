//! Main entry point for petalTongue desktop UI

use petal_tongue_core::{Instance, InstanceId, InstanceRegistry};
use petal_tongue_ui::PetalTongueApp;
use petal_tongue_ui::display::prompt::prompt_for_display_server;

fn main() -> anyhow::Result<()> {
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

    tracing::info!("🧹 Running garbage collection...");

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

    tracing::info!("📝 Registry saved");

    tracing::debug!(
        "Instance ID will be passed to app: {}",
        instance_id.as_str()
    );
    // ===== End Phase 1 Integration =====

    tracing::info!("🔄 Phase 1 complete, starting Phase 2...");
    tracing::info!("🌸 Starting petalTongue Universal Representation System");

    // ===== Phase 2: Pure Rust Display System Integration =====
    // Check if external display is available, prompt if not
    tracing::info!("🎨 Checking display availability...");

    let has_display = std::env::var("DISPLAY").is_ok()
        || std::env::var("WAYLAND_DISPLAY").is_ok()
        || cfg!(target_os = "windows")
        || cfg!(target_os = "macos");

    if !has_display {
        tracing::info!("🪟 No display server detected");
        tracing::info!("   Pure Rust display backends:");
        tracing::info!("   - TerminalGUI (ASCII art topology)");
        tracing::info!("   - SVGGUI (vector export)");
        tracing::info!("   - PNGGUI (raster export)");
        tracing::info!("   - Toadstool WASM (if available)");

        // Prompt user about display server
        match prompt_for_display_server() {
            Ok(true) => tracing::info!("✅ Display server now available"),
            Ok(false) => tracing::info!("📦 Continuing without display server"),
            Err(e) => tracing::warn!("⚠️  Prompt error: {}", e),
        }
    } else {
        tracing::info!("✅ Display server detected");
    }

    // Try to run with eframe
    let result = run_with_eframe();

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

    result.map_err(|e| anyhow::anyhow!("eframe error: {:?}", e))
}

/// Run with traditional eframe
fn run_with_eframe() -> Result<(), eframe::Error> {
    // v1.2.0: Conditional diagnostic logging (set PETALTONGUE_DIAG=1 to enable)
    let diagnostic_enabled = std::env::var("PETALTONGUE_DIAG").is_ok();
    
    if diagnostic_enabled {
        tracing::info!("🎬 DIAGNOSTIC: Entered run_with_eframe()");
        tracing::info!("🔍 DIAGNOSTIC: DISPLAY={:?}", std::env::var("DISPLAY"));
        tracing::info!("🔍 DIAGNOSTIC: WAYLAND_DISPLAY={:?}", std::env::var("WAYLAND_DISPLAY"));
    }
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("🌸 petalTongue - Universal Representation System")
            .with_visible(true), // FIX: Explicitly show window (critical for headless+remote setups!)
        ..Default::default()
    };

    if diagnostic_enabled {
        tracing::info!("🎬 DIAGNOSTIC: About to call eframe::run_native");
        tracing::info!("🎬 DIAGNOSTIC: This will block until window closes");
    }
    
    let result = eframe::run_native(
        "petalTongue",
        options,
        Box::new(move |cc| {
            if diagnostic_enabled {
                tracing::info!("🎨 DIAGNOSTIC: Inside app creation callback");
                tracing::info!("🎨 DIAGNOSTIC: Creating PetalTongueApp...");
            }
            let app = PetalTongueApp::new(cc);
            if diagnostic_enabled {
                tracing::info!("🎨 DIAGNOSTIC: PetalTongueApp created successfully");
            }
            Ok(Box::new(app))
        }),
    );
    
    if diagnostic_enabled {
        tracing::info!("🎬 DIAGNOSTIC: eframe::run_native returned: {:?}", result);
    }
    result
}
