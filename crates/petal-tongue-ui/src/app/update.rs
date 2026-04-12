// SPDX-License-Identifier: AGPL-3.0-or-later
//! Frame update logic for petalTongue UI.
//!
//! Handles sensory processing, motor command draining, IPC broadcasting,
//! keyboard shortcuts, game loop tick, and panel rendering.

use super::PetalTongueApp;
use super::events;
use super::panels;
use super::sensory;
use crate::keyboard_shortcuts::ShortcutAction;
use petal_tongue_scene::game_loop::tick_frame;
use std::time::{Duration, Instant};

/// Run one update cycle (sensory → motor drain → panels).
///
/// Called by both `eframe::App::update` and headless harness.
pub fn run_update(app: &mut PetalTongueApp, ctx: &egui::Context) {
    app.interaction_bridge
        .inverse_pipeline_mut()
        .clear_targets();
    sensory::process_sensory_feedback(app, ctx);
    events::drain_motor_commands(app);

    if let Ok(mut reg) = app.channel_registry.write() {
        if ctx.input(|i| !i.events.is_empty())
            && let Some(ch) = reg.get_mut("keyboard-afferent")
        {
            ch.record_signal_in();
            ch.record_signal_out();
        }
        if ctx.input(|i| i.pointer.any_click() || i.pointer.any_down())
            && let Some(ch) = reg.get_mut("pointer-afferent")
        {
            ch.record_signal_in();
            ch.record_signal_out();
        }
        if let Some(ch) = reg.get_mut("visual-efferent") {
            ch.record_signal_in();
            ch.record_signal_out();
        }
    }

    // Broadcast sensor events to IPC subscribers (engagement/AI consumers)
    if let Some(ref reg) = app.sensor_stream {
        let events = crate::sensor_feed::collect_sensor_events(ctx);
        if !events.is_empty()
            && let Ok(mut r) = reg.write()
        {
            r.broadcast(&events);
        }
    }

    // Broadcast interaction events (selection changes) to IPC subscribers
    if let Some(ref reg) = app.interaction_subscribers {
        let current = app.visual_renderer.selected_node().map(ToString::to_string);
        if current != app.last_broadcast_selection {
            let event = petal_tongue_ipc::InteractionEventNotification {
                event_type: if current.is_some() {
                    "select".to_string()
                } else {
                    "deselect".to_string()
                },
                targets: current.clone().into_iter().collect(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                perspective_id: None,
            };
            if let Ok(mut r) = reg.write() {
                let callbacks = r.broadcast(&event);
                if let Some(ref tx) = app.callback_tx {
                    for cb in callbacks {
                        if let Err(e) = tx.send(cb) {
                            tracing::warn!("callback push failed (channel closed): {e}");
                        }
                    }
                }
            }
            app.last_broadcast_selection = current;
        }
    }

    if app.awakening_overlay.is_active() {
        let delta_time = ctx.input(|i| i.stable_dt);
        if let Err(e) = app.awakening_overlay.update(delta_time) {
            tracing::error!("Awakening overlay update error: {}", e);
        }
        app.awakening_overlay.render(ctx);
        if app.awakening_overlay.should_transition_to_tutorial() {
            tracing::info!("Transitioning to tutorial mode");
            let tutorial = crate::tutorial_mode::TutorialMode::new();
            if tutorial.is_enabled() {
                tutorial.load_into_graph(app.graph.clone(), app.current_layout);
            }
        }
        ctx.request_repaint();
        app.feed_introspection();
        return;
    }

    ctx.input(|i| {
        if i.key_pressed(egui::Key::P) && !i.modifiers.ctrl && !i.modifiers.shift {
            app.show_neural_proprioception = !app.show_neural_proprioception;
            tracing::info!(
                "Neural Proprioception Panel {}",
                if app.show_neural_proprioception {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
        if i.key_pressed(egui::Key::M) && !i.modifiers.ctrl && !i.modifiers.shift {
            app.show_neural_metrics = !app.show_neural_metrics;
            tracing::info!(
                "Neural Metrics Dashboard {}",
                if app.show_neural_metrics {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
        if i.key_pressed(egui::Key::G) && !i.modifiers.ctrl && !i.modifiers.shift {
            app.show_graph_builder = !app.show_graph_builder;
            tracing::info!(
                "Graph Builder {}",
                if app.show_graph_builder {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
    });

    // Apply keyboard shortcuts (Ctrl+D, Ctrl+A, Ctrl+H, Escape, etc.)
    match app.keyboard_shortcuts.handle_input(ctx) {
        ShortcutAction::ToggleHelp => {}
        ShortcutAction::CloseOverlays => {
            app.accessibility_panel.show = false;
            app.keyboard_shortcuts.show_help = false;
        }
        ShortcutAction::ToggleAccessibility => {
            app.accessibility_panel.show = !app.accessibility_panel.show;
        }
        ShortcutAction::ToggleDashboard => {
            app.show_dashboard = !app.show_dashboard;
        }
        ShortcutAction::FocusTools => {
            // Focus tools menu - no-op for headless; tools render in central panel
        }
        ShortcutAction::Refresh => {
            app.refresh_graph_data();
        }
        ShortcutAction::SelectColorScheme(idx) => {
            if let Some(&scheme) = crate::accessibility::ColorScheme::all().get(idx) {
                app.accessibility_panel.settings.color_scheme = scheme;
            }
        }
        ShortcutAction::IncreaseFontSize => {
            app.accessibility_panel.settings.font_size =
                app.accessibility_panel.settings.font_size.increase();
        }
        ShortcutAction::DecreaseFontSize => {
            app.accessibility_panel.settings.font_size =
                app.accessibility_panel.settings.font_size.decrease();
        }
        ShortcutAction::None => {}
    }

    app.system_dashboard
        .set_audio_enabled(app.accessibility_panel.settings.audio_enabled);
    app.system_dashboard
        .set_audio_volume(app.accessibility_panel.settings.audio_volume);

    if app.show_animation
        && let Ok(mut engine) = app.animation_engine.write()
    {
        engine.update();
    }

    // Game loop: fixed-timestep tick for scene animation and physics.
    if app.continuous_mode {
        let dt = ctx.input(|i| i.stable_dt);
        app.tick_clock.begin_frame_with_dt(dt);
        let tick_result = tick_frame(
            &mut app.tick_clock,
            Some(&mut app.physics_world),
            Some(&mut app.animation_player),
            Some(&mut app.active_scene),
        );
        if tick_result.scene_dirty {
            ctx.request_repaint();
        }
    }

    // IPC-to-UI bridge: check for updated visualization sessions.
    if let Some(ref viz_state) = app.visualization_state
        && crate::live_sessions::has_updates_since(viz_state, app.last_session_poll)
    {
        app.last_session_poll = Instant::now();
        ctx.request_repaint();
    }

    // AI interaction adapter: poll for AI-driven interaction commands.
    if let Some(ref interaction_subs) = app.interaction_subscribers
        && let Ok(mut reg) = interaction_subs.try_write()
    {
        app.ai_adapter.poll(&mut reg);
        let commands = app.ai_adapter.drain_commands();
        for cmd in &commands {
            crate::ai_adapter::AiAdapter::apply_command(&mut app.interaction_bridge, cmd);
        }
        if !commands.is_empty() {
            ctx.request_repaint();
        }
    }

    let mut style = (*ctx.style()).clone();
    let palette = app.accessibility_panel.get_palette();
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(palette.text);
    style.visuals.window_fill = palette.background;
    style.visuals.panel_fill = palette.background_alt;
    ctx.set_style(style);

    panels::render_all_panels(ctx, app);

    app.accessibility_panel.show(ctx);
    app.keyboard_shortcuts.render_help(ctx, &palette);

    if app.auto_refresh {
        let elapsed = app.last_refresh.elapsed();
        if elapsed >= Duration::from_secs_f32(app.refresh_interval) {
            app.refresh_graph_data();
        }
        ctx.request_repaint();
    }

    app.feed_introspection();
}
