// SPDX-License-Identifier: AGPL-3.0-or-later
//! Motor command handling (efferent channel sink).
//!
//! Drains pending motor commands and applies them to UI state.
//! Extracted from app/mod.rs for domain separation.

use petal_tongue_core::{MotorCommand, PanelId};

use super::PetalTongueApp;
use super::layout;

/// Drain all pending motor commands and apply them to UI state.
pub fn drain_motor_commands(app: &mut PetalTongueApp) {
    while let Ok(cmd) = app.motor_rx.try_recv() {
        apply_motor_command(app, cmd);
    }
}

/// Apply a single motor command to UI state (efferent signal → effector).
pub fn apply_motor_command(app: &mut PetalTongueApp, cmd: MotorCommand) {
    let cmd_description = format!("{cmd:?}");
    match cmd {
        MotorCommand::RenderFrame { .. }
        | MotorCommand::UpdateDisplay
        | MotorCommand::ClearDisplay => {
            // Rendering commands handled by the existing awareness system
        }
        MotorCommand::SetPanelVisibility { panel, visible } => {
            match panel {
                PanelId::LeftSidebar => app.show_controls = visible,
                PanelId::RightSidebar => {
                    app.show_audio_panel = visible;
                    app.show_dashboard = visible;
                    app.show_trust_dashboard = visible;
                }
                PanelId::TopMenu => app.show_top_menu = visible,
                PanelId::SystemDashboard => app.show_dashboard = visible,
                PanelId::AudioPanel => app.show_audio_panel = visible,
                PanelId::TrustDashboard => app.show_trust_dashboard = visible,
                PanelId::Proprioception => app.show_neural_proprioception = visible,
                PanelId::GraphStats => app.visual_renderer.set_show_stats(visible),
                PanelId::Custom(_) => {}
            }
            tracing::debug!("Motor: SetPanelVisibility({panel:?}, {visible})");
        }
        MotorCommand::SetZoom { level } => {
            app.visual_renderer.set_zoom(level);
            tracing::debug!("Motor: SetZoom({level})");
        }
        MotorCommand::FitToView => {
            app.visual_renderer.fit_to_view(&app.graph);
            tracing::debug!("Motor: FitToView");
        }
        MotorCommand::Navigate { ref target_node } => {
            app.visual_renderer
                .navigate_to_node(target_node, &app.graph);
            tracing::debug!("Motor: Navigate({target_node})");
        }
        MotorCommand::SelectNode { ref node_id } => {
            if let Some(id) = node_id {
                app.visual_renderer.select_node(Some(id));
            } else {
                app.visual_renderer.select_node(None::<&str>);
            }
            tracing::debug!("Motor: SelectNode({node_id:?})");
        }
        MotorCommand::SetLayout { ref algorithm } => {
            let layout = layout::layout_from_str(algorithm);
            app.current_layout = layout;
            if let Ok(mut graph) = app.graph.write() {
                graph.set_layout(layout);
            }
            tracing::debug!("Motor: SetLayout({algorithm})");
        }
        MotorCommand::SetMode { ref mode } => {
            tracing::info!("Motor: SetMode({mode})");
            app.neural_proprioception_panel.set_current_mode(mode);
            let commands = crate::mode_presets::commands_for_mode(mode);
            for sub_cmd in commands {
                apply_motor_command(app, sub_cmd);
            }
        }
        MotorCommand::SetAwakening { enabled } => {
            if !enabled {
                app.awakening_overlay.skip();
            }
            tracing::debug!("Motor: SetAwakening({enabled})");
        }
        MotorCommand::LoadScenario { ref path } => {
            tracing::info!("Motor: LoadScenario({path})");
        }
        MotorCommand::SetContinuousMode { enabled } => {
            app.continuous_mode = enabled;
            tracing::debug!("Motor: SetContinuousMode({enabled})");
        }
        MotorCommand::SetPhysics { enabled } => {
            app.tick_clock.config_mut().physics_enabled = enabled;
            tracing::debug!("Motor: SetPhysics({enabled})");
        }
        MotorCommand::SetSceneAnimation { enabled } => {
            app.tick_clock.config_mut().animation_enabled = enabled;
            tracing::debug!("Motor: SetSceneAnimation({enabled})");
        }
        MotorCommand::PlayAudio { ref sound } => {
            app.audio_system.play(sound);
            tracing::debug!("Motor: PlayAudio({sound})");
        }
    }
    app.neural_proprioception_panel
        .record_motor_command(&cmd_description);
}
