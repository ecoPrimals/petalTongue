// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario file loading and post-start scenario application (graph, modes, motor commands).

use super::PetalTongueApp;
use petal_tongue_core::MotorCommand;

/// Load a scenario from disk if a path was given; logs errors and returns `(scenario, path_for_providers)`.
pub(super) fn load_scenario_from_cli(
    scenario_path: Option<std::path::PathBuf>,
) -> (
    Option<crate::scenario::Scenario>,
    Option<std::path::PathBuf>,
) {
    scenario_path.map_or((None, None), |path| {
        match crate::scenario::Scenario::load(&path) {
            Ok(s) => {
                tracing::info!("✅ Scenario loaded successfully: {}", s.name);
                (Some(s), Some(path))
            }
            Err(e) => {
                tracing::error!("❌ Failed to load scenario: {}", e);
                (None, None)
            }
        }
    })
}

/// Apply awakening, audio, status, and scenario or tutorial data after core `PetalTongueApp` construction.
pub(super) fn finalize_app_startup(
    app: &mut PetalTongueApp,
    scenario: Option<&crate::scenario::Scenario>,
    tutorial_mode: &crate::tutorial_mode::TutorialMode,
    needs_fallback: bool,
) {
    // Awakening: respect env var, then scenario config
    let env_awakening = std::env::var("AWAKENING_ENABLED")
        .ok()
        .and_then(|v| v.parse::<bool>().ok());
    let scenario_awakening = scenario.map(|s| s.ui_config.awakening_enabled);
    let awakening_enabled = env_awakening.unwrap_or_else(|| scenario_awakening.unwrap_or(true));

    if awakening_enabled {
        tracing::info!("🌸 Awakening experience enabled");
        app.awakening_overlay.start();
    }

    tracing::info!("🎵 Initializing startup audio...");
    let startup_audio = crate::startup_audio::StartupAudio::new();
    if startup_audio.has_startup_music() {
        tracing::info!(
            "🎵 Startup music found: {:?}",
            startup_audio.startup_music_path()
        );
    }
    startup_audio.play(&app.audio_system);

    tracing::info!("📊 Writing initial status file for AI observability...");
    app.status_reporter.update_health("healthy");
    app.status_reporter.force_write();

    if let Some(loaded_scenario) = scenario {
        tracing::info!(
            "📋 Loading {} primals and {} edges from scenario",
            loaded_scenario.primal_count(),
            loaded_scenario.edge_count()
        );
        let primals = loaded_scenario.to_primal_infos();
        let Ok(mut graph) = app.graph.write() else {
            tracing::error!("graph lock poisoned");
            return;
        };
        graph.clear();
        for primal in &primals {
            graph.add_node(primal.clone());
        }
        for edge in &loaded_scenario.edges {
            graph.add_edge(edge.clone());
        }
        tracing::info!(
            "✅ Scenario loaded: {} nodes, {} edges",
            primals.len(),
            loaded_scenario.edge_count()
        );
        drop(graph);

        let panels = &loaded_scenario.ui_config.show_panels;
        app.show_top_menu = panels.top_menu;
        app.show_neural_proprioception = panels.proprioception && app.show_neural_proprioception;

        let mode = &loaded_scenario.mode;
        if !mode.is_empty() && mode != "live-ecosystem" && mode != "doom-showcase" {
            let cmds = crate::mode_presets::commands_for_mode(mode);
            if !cmds.is_empty() {
                tracing::info!(
                    "🎛️  Applying scenario mode '{mode}' ({} commands)",
                    cmds.len()
                );
                for cmd in cmds {
                    super::events::apply_motor_command(app, cmd);
                }
            }
        }

        let zoom_str = &loaded_scenario.ui_config.initial_zoom;
        if zoom_str == "fit" {
            super::events::apply_motor_command(app, MotorCommand::FitToView);
        } else if let Ok(level) = zoom_str.parse::<f32>() {
            super::events::apply_motor_command(app, MotorCommand::SetZoom { level });
        }
    } else if tutorial_mode.is_enabled() {
        tutorial_mode.load_into_graph(std::sync::Arc::clone(&app.graph), app.current_layout);
    } else if needs_fallback {
        crate::tutorial_mode::TutorialMode::create_fallback_scenario(
            std::sync::Arc::clone(&app.graph),
            app.current_layout,
        );
    } else {
        app.refresh_graph_data();
    }
}
