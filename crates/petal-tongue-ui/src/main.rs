// SPDX-License-Identifier: AGPL-3.0-or-later
//! Main entry point for petalTongue desktop UI

use clap::Parser;
use petal_tongue_core::{
    GraphEngine, Instance, InstanceId, InstanceRegistry, MotorCommand, RenderingCapabilities,
    constants::PRIMAL_NAME,
};
use petal_tongue_ui::PetalTongueApp;
use petal_tongue_ui::display::prompt::prompt_for_display_server;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

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

fn main() -> petal_tongue_ui::error::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI args
    let cli = Cli::parse();
    let scenario_path = match &cli.command {
        Some(Commands::Ui { scenario }) => scenario.clone().or_else(|| cli.scenario.clone()),
        None => cli.scenario.clone(),
    };

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

    if has_display {
        tracing::info!("✅ Display server detected");
    } else {
        tracing::info!("🪟 No display server detected");
        tracing::info!("   Pure Rust display backends:");
        tracing::info!("   - TerminalGUI (ASCII art topology)");
        tracing::info!("   - SVGGUI (vector export)");
        tracing::info!("   - PNGGUI (raster export)");
        tracing::info!("   - Network display (if available)");

        // Prompt user about display server
        let rt = tokio::runtime::Runtime::new()?;
        match rt.block_on(prompt_for_display_server()) {
            Ok(true) => tracing::info!("✅ Display server now available"),
            Ok(false) => tracing::info!("📦 Continuing without display server"),
            Err(e) => tracing::warn!("⚠️  Prompt error: {}", e),
        }
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

    result
        .map_err(|e| petal_tongue_ui::error::BackendError::EframeRunFailed(format!("{e:?}")).into())
}

/// Run with traditional eframe
fn run_with_eframe(
    scenario_path: Option<PathBuf>,
    rendering_caps: RenderingCapabilities,
) -> Result<(), eframe::Error> {
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
                "🌸 {PRIMAL_NAME} - Universal Representation System"
            ))
            .with_visible(true)
            .with_active(true),
        centered: true,
        ..Default::default()
    };

    if diagnostic_enabled {
        tracing::info!("🎬 DIAGNOSTIC: About to call eframe::run_native");
        tracing::info!("🎬 DIAGNOSTIC: This will block until window closes");
    }

    let shared_viz_state = Arc::new(RwLock::new(
        petal_tongue_ipc::visualization_handler::VisualizationState::new(),
    ));

    let result = eframe::run_native(
        PRIMAL_NAME,
        options,
        Box::new(move |cc| {
            if diagnostic_enabled {
                tracing::info!("🎨 DIAGNOSTIC: Creating PetalTongueApp...");
            }
            let mut app = PetalTongueApp::new(cc, scenario_path, rendering_caps.clone())?;
            app.set_visualization_state(Arc::clone(&shared_viz_state));

            let ipc_handles = start_ipc_server(
                &app.graph_handle(),
                app.motor_sender(),
                Arc::clone(app.rendering_awareness()),
                Arc::clone(&shared_viz_state),
            );
            if let Some(handles) = ipc_handles {
                app.set_sensor_stream(handles.sensor_stream);
                app.set_interaction_subscribers(handles.interaction_subscribers);
                if let Some(tx) = handles.callback_tx {
                    app.set_callback_tx(tx);
                }
            }

            register_with_neural_api_background();

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

/// Handles returned by `start_ipc_server` so the UI can broadcast events.
struct IpcHandles {
    sensor_stream: Arc<RwLock<petal_tongue_ipc::SensorStreamRegistry>>,
    interaction_subscribers: Arc<RwLock<petal_tongue_ipc::InteractionSubscriberRegistry>>,
    callback_tx: Option<tokio::sync::mpsc::UnboundedSender<petal_tongue_ipc::CallbackDispatch>>,
}

/// Build the IPC server, extract shared handles, then start it in a background thread.
///
/// Returns `None` only when the server fails to construct (e.g. socket path error).
fn start_ipc_server(
    graph: &Arc<RwLock<GraphEngine>>,
    motor_tx: std::sync::mpsc::Sender<MotorCommand>,
    rendering_awareness: Arc<RwLock<petal_tongue_core::RenderingAwareness>>,
    viz_state: Arc<RwLock<petal_tongue_ipc::visualization_handler::VisualizationState>>,
) -> Option<IpcHandles> {
    let server = match petal_tongue_ipc::UnixSocketServer::new(Arc::clone(graph)) {
        Ok(s) => s
            .with_motor_sender(motor_tx)
            .with_rendering_awareness(rendering_awareness)
            .with_visualization_state(viz_state),
        Err(e) => {
            tracing::warn!("IPC server not started: {e}");
            return None;
        }
    };

    let sensor_stream = server.sensor_stream_handle();
    let interaction_subscribers = server.interaction_subscribers_handle();
    let callback_tx = server.callback_sender();
    let server = Arc::new(server);

    std::thread::Builder::new()
        .name("ipc-server".into())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::error!("Failed to create IPC runtime: {e}");
                    return;
                }
            };
            rt.block_on(async {
                tracing::info!(
                    "🔌 IPC server starting with motor channel + introspection + live viz bridge"
                );
                if let Err(e) = server.start().await {
                    tracing::error!("IPC server error: {e}");
                }
            });
        })
        .ok();

    Some(IpcHandles {
        sensor_stream,
        interaction_subscribers,
        callback_tx,
    })
}

/// Attempt Neural API registration in a background thread.
///
/// Non-blocking: if biomeOS is not running, registration fails gracefully
/// and no heartbeat thread is spawned.
fn register_with_neural_api_background() {
    std::thread::Builder::new()
        .name("neural-register".into())
        .spawn(|| {
            let our_socket = match petal_tongue_ipc::socket_path::get_petaltongue_socket_path() {
                Ok(p) => p,
                Err(e) => {
                    tracing::debug!(
                        "Cannot determine own socket path for Neural API registration: {e}"
                    );
                    return;
                }
            };

            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::error!("Failed to create Neural API registration runtime: {e}");
                    return;
                }
            };

            match rt.block_on(
                petal_tongue_ui::neural_registration::register_with_neural_api(&our_socket),
            ) {
                Ok(provider) => {
                    tracing::info!("🧠 Neural API registration successful, starting heartbeat");
                    petal_tongue_ui::neural_registration::spawn_heartbeat(provider, our_socket);
                }
                Err(e) => {
                    tracing::info!("Neural API not available: {e} (standalone mode)");
                }
            }
        })
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parse_default() {
        let cli = Cli::parse_from(["petal-tongue"]);
        assert!(cli.command.is_none());
        assert!(cli.scenario.is_none());
    }

    #[test]
    fn cli_parse_ui_subcommand() {
        let cli = Cli::parse_from(["petal-tongue", "ui"]);
        assert!(matches!(cli.command, Some(Commands::Ui { .. })));
        if let Some(Commands::Ui { scenario }) = &cli.command {
            assert!(scenario.is_none());
        }
    }

    #[test]
    fn cli_parse_ui_with_scenario() {
        let cli = Cli::parse_from(["petal-tongue", "ui", "--scenario", "/tmp/test.json"]);
        assert!(matches!(cli.command, Some(Commands::Ui { .. })));
        if let Some(Commands::Ui { scenario }) = &cli.command {
            assert_eq!(
                scenario.as_ref().map(|p| p.to_string_lossy().into_owned()),
                Some("/tmp/test.json".to_string())
            );
        }
    }

    #[test]
    fn cli_parse_global_scenario() {
        let cli = Cli::parse_from(["petal-tongue", "--scenario", "/path/to/scenario.json"]);
        assert!(cli.scenario.is_some());
        assert_eq!(
            cli.scenario
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            Some("/path/to/scenario.json".to_string())
        );
    }

    #[test]
    fn scenario_path_from_ui_overrides_global() {
        let cli = Cli::parse_from([
            "petal-tongue",
            "--scenario",
            "/global.json",
            "ui",
            "--scenario",
            "/ui.json",
        ]);
        let scenario_path = match &cli.command {
            Some(Commands::Ui { scenario }) => scenario.clone().or_else(|| cli.scenario.clone()),
            None => cli.scenario.clone(),
        };
        assert_eq!(
            scenario_path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            Some("/ui.json".to_string())
        );
    }

    #[test]
    fn scenario_path_from_ui_falls_back_to_global() {
        let cli = Cli::parse_from(["petal-tongue", "--scenario", "/global.json", "ui"]);
        let scenario_path = match &cli.command {
            Some(Commands::Ui { scenario }) => scenario.clone().or_else(|| cli.scenario.clone()),
            None => cli.scenario.clone(),
        };
        assert_eq!(
            scenario_path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            Some("/global.json".to_string())
        );
    }

    #[test]
    fn scenario_path_none_when_no_command_no_global() {
        let cli = Cli::parse_from(["petal-tongue"]);
        let scenario_path = match &cli.command {
            Some(Commands::Ui { scenario }) => scenario.clone().or_else(|| cli.scenario.clone()),
            None => cli.scenario.clone(),
        };
        assert!(scenario_path.is_none());
    }

    #[test]
    fn ipc_handles_struct_fields() {
        let _handles = IpcHandles {
            sensor_stream: Arc::new(RwLock::new(petal_tongue_ipc::SensorStreamRegistry::new())),
            interaction_subscribers: Arc::new(RwLock::new(
                petal_tongue_ipc::InteractionSubscriberRegistry::new(),
            )),
            callback_tx: None,
        };
    }

    #[test]
    fn has_display_env_detection() {
        let has_display = std::env::var("DISPLAY").is_ok()
            || std::env::var("WAYLAND_DISPLAY").is_ok()
            || cfg!(target_os = "windows")
            || cfg!(target_os = "macos");
        let _ = has_display;
    }

    #[test]
    fn expected_rgba8_buffer_size_from_discovered_display() {
        use petal_tongue_ui::display::backends::discovered_display::expected_rgba8_buffer_size;
        assert_eq!(expected_rgba8_buffer_size(1920, 1080), 1920 * 1080 * 4);
        assert_eq!(expected_rgba8_buffer_size(1, 1), 4);
    }
}
