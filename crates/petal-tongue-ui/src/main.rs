// SPDX-License-Identifier: AGPL-3.0-only
//! Main entry point for petalTongue desktop UI

use clap::Parser;
use petal_tongue_core::{
    Instance, InstanceId, InstanceRegistry, RenderingCapabilities, constants::PRIMAL_NAME,
};
use petal_tongue_ui::PetalTongueApp;
use petal_tongue_ui::display::prompt::prompt_for_display_server;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "petal-tongue")]
#[command(about = "petalTongue - The Human Interface for ecoPrimals", long_about = None)]
#[command(version)]
struct Cli {
    /// Subcommand (defaults to 'ui' if not specified)
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to scenario JSON file
    #[arg(long, global = true)]
    scenario: Option<PathBuf>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Launch the graphical UI (default)
    Ui {
        /// Path to scenario JSON file
        #[arg(long)]
        scenario: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI args
    let cli = Cli::parse();
    let scenario_path = match &cli.command {
        Some(Commands::Ui { scenario }) => scenario.clone().or(cli.scenario.clone()),
        None => cli.scenario.clone(),
    };

    // ===== Phase 1: Instance Management Integration =====
    // Create unique instance ID for this petalTongue instance
    let instance_id = InstanceId::new();
    let id_str = instance_id.as_str();
    tracing::info!("🌸 Starting petalTongue instance: {}", id_str);

    // Create instance metadata
    let instance = Instance::new(instance_id.clone(), Some(PRIMAL_NAME.to_string()))?;

    // Load/create instance registry
    let mut registry = InstanceRegistry::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load registry: {}, creating new", e);
        InstanceRegistry::new()
    });

    // Register this instance
    if let Err(e) = registry.register(instance) {
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

    // ===== Phase 2.5: Device Capability Detection =====
    tracing::info!("🎨 Detecting device capabilities...");
    let rendering_caps = RenderingCapabilities::detect();
    tracing::info!(
        "✅ Device type: {} | UI complexity: {:?}",
        rendering_caps.device_type,
        rendering_caps.ui_complexity
    );
    tracing::info!(
        "   Screen: {:?} | Modalities: {}",
        rendering_caps.screen_size,
        rendering_caps.modalities.len()
    );
    // ===== End Phase 2.5 =====

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
        let rt = tokio::runtime::Runtime::new()?;
        match rt.block_on(prompt_for_display_server()) {
            Ok(true) => tracing::info!("✅ Display server now available"),
            Ok(false) => tracing::info!("📦 Continuing without display server"),
            Err(e) => tracing::warn!("⚠️  Prompt error: {}", e),
        }
    } else {
        tracing::info!("✅ Display server detected");
    }

    // Try to run with eframe
    let result = run_with_eframe(scenario_path, rendering_caps);

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
fn run_with_eframe(
    scenario_path: Option<PathBuf>,
    rendering_caps: RenderingCapabilities,
) -> Result<(), eframe::Error> {
    // v1.2.0: Conditional diagnostic logging (set PETALTONGUE_DIAG=1 to enable)
    let diagnostic_enabled = std::env::var("PETALTONGUE_DIAG").is_ok();

    if diagnostic_enabled {
        tracing::info!("🎬 DIAGNOSTIC: Entered run_with_eframe()");
        tracing::info!("🔍 DIAGNOSTIC: DISPLAY={:?}", std::env::var("DISPLAY"));
        tracing::info!(
            "🔍 DIAGNOSTIC: WAYLAND_DISPLAY={:?}",
            std::env::var("WAYLAND_DISPLAY")
        );
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 900.0])
            .with_title(format!(
                "🌸 {} - Universal Representation System",
                PRIMAL_NAME
            ))
            .with_visible(true) // FIX: Explicitly show window (critical for headless+remote setups!)
            .with_active(true), // 🖥️ REMOTE DESKTOP FIX: Request active/focused state
        // 🖥️ CRITICAL: Always request input focus (for remote desktop)
        centered: true,
        ..Default::default()
    };

    if diagnostic_enabled {
        tracing::info!("🎬 DIAGNOSTIC: About to call eframe::run_native");
        tracing::info!("🎬 DIAGNOSTIC: This will block until window closes");
    }

    let result = eframe::run_native(
        PRIMAL_NAME,
        options,
        Box::new(move |cc| {
            if diagnostic_enabled {
                tracing::info!("🎨 DIAGNOSTIC: Inside app creation callback");
                tracing::info!("🎨 DIAGNOSTIC: Creating PetalTongueApp...");
            }
            let app = PetalTongueApp::new(cc, scenario_path, rendering_caps.clone())?;
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
