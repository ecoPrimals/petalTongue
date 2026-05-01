// SPDX-License-Identifier: AGPL-3.0-or-later
//! Panel routing and layout.
//!
//! Renders all UI panels based on visibility state.
//! Extracted from app/mod.rs for domain separation.

use super::PetalTongueApp;
use crate::panel_registry::PanelInstance;
use crate::tool_integration::ToolPanel;

/// Render all panels based on current visibility state.
pub fn render_all_panels(ctx: &egui::Context, app: &mut PetalTongueApp) {
    let palette = app.accessibility_panel.get_palette();

    if app.show_top_menu {
        egui::TopBottomPanel::top("top_panel")
            .frame(
                egui::Frame::none()
                    .fill(palette.background)
                    .inner_margin(8.0),
            )
            .show(ctx, |ui| {
                let refresh_clicked = egui::menu::bar(ui, |ui| {
                    crate::app_panels::render_top_menu_bar(
                        ui,
                        &palette,
                        &mut app.accessibility_panel,
                        &mut app.visual_renderer,
                        &mut app.tools,
                        &mut app.current_layout,
                        &app.graph,
                        &mut app.show_dashboard,
                        &mut app.show_controls,
                        &mut app.show_audio_panel,
                        &mut app.show_capability_panel,
                        &mut app.show_neural_proprioception,
                        &mut app.show_neural_metrics,
                        &mut app.show_graph_builder,
                    )
                })
                .inner;
                if refresh_clicked {
                    app.refresh_graph_data();
                }
            });
    }

    if app.show_controls {
        egui::SidePanel::left("controls_panel")
            .default_width(280.0)
            .frame(
                egui::Frame::none()
                    .fill(palette.background_alt)
                    .inner_margin(12.0),
            )
            .show(ctx, |ui| {
                let elapsed = app.last_refresh.elapsed().as_secs_f32();
                let refresh_clicked = crate::app_panels::render_controls_panel(
                    ui,
                    &palette,
                    &app.accessibility_panel,
                    &mut app.auto_refresh,
                    &mut app.refresh_interval,
                    elapsed,
                    &mut app.show_animation,
                    &mut app.visual_renderer,
                );
                if refresh_clicked {
                    app.refresh_graph_data();
                }
            });
    }

    if app.show_audio_panel {
        egui::SidePanel::right("audio_panel")
            .default_width(380.0)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(30, 30, 35))
                    .inner_margin(12.0),
            )
            .show(ctx, |ui| {
                crate::app_panels::render_audio_panel(
                    ui,
                    &palette,
                    &app.accessibility_panel,
                    &mut app.audio_renderer,
                    &app.audio_generator,
                    &app.visual_renderer,
                    &app.capabilities,
                );
            });
    }

    if app.show_capability_panel {
        crate::app_panels::render_capability_panel(ctx, &palette, &app.capabilities);
    }

    if app.show_dashboard {
        egui::SidePanel::right("dashboard_panel")
            .default_width(220.0)
            .resizable(true)
            .frame(
                egui::Frame::none()
                    .fill(palette.background_alt)
                    .inner_margin(12.0),
            )
            .show(ctx, |ui| {
                let font_scale = app.accessibility_panel.settings.font_size.multiplier();
                app.system_dashboard.render_compact(
                    ui,
                    &palette,
                    font_scale,
                    Some(&app.audio_system),
                );
                ui.add_space(8.0);
                crate::system_dashboard::SystemDashboard::render_sensory_status(
                    ui,
                    &palette,
                    font_scale,
                    &app.rendering_awareness,
                    &app.sensor_registry,
                );
                ui.add_space(8.0);
                crate::system_dashboard::SystemDashboard::render_proprioception_status(
                    ui,
                    &palette,
                    font_scale,
                    &mut app.proprioception,
                );
            });
    }

    if app.show_trust_dashboard {
        egui::SidePanel::right("trust_dashboard_panel")
            .default_width(280.0)
            .resizable(true)
            .frame(
                egui::Frame::none()
                    .fill(palette.background_alt)
                    .inner_margin(12.0),
            )
            .show(ctx, |ui| {
                let font_scale = app.accessibility_panel.settings.font_size.multiplier();
                let intents =
                    app.trust_dashboard
                        .render(ui, &palette, font_scale, Some(&app.audio_system));
                for intent in intents {
                    match intent {
                        crate::trust_dashboard::TrustIntent::PlayAudio { sound } => {
                            app.audio_system.play(&sound);
                        }
                    }
                }
            });
    }

    if app.show_neural_proprioception {
        if let Ok(reg) = app.channel_registry.read() {
            let snapshots = reg.snapshots();
            let afferent: Vec<_> = snapshots
                .iter()
                .filter(|s| s.direction == petal_tongue_core::ChannelDirection::Afferent)
                .cloned()
                .collect();
            let efferent: Vec<_> = snapshots
                .iter()
                .filter(|s| s.direction == petal_tongue_core::ChannelDirection::Efferent)
                .cloned()
                .collect();
            app.neural_proprioception_panel
                .merge_local_channels(afferent, efferent);
        }
        egui::Window::new("Neural API Proprioception")
            .default_width(500.0)
            .default_height(600.0)
            .default_pos([100.0, 100.0])
            .show(ctx, |ui| {
                if let Some(provider) = &app.neural_api_provider {
                    app.tokio_runtime.block_on(async {
                        app.neural_proprioception_panel
                            .update(provider.as_ref())
                            .await;
                    });
                }
                app.neural_proprioception_panel.render(ui);
            });
    }

    if app.show_neural_metrics {
        egui::Window::new("Neural API Metrics")
            .default_width(600.0)
            .default_height(500.0)
            .default_pos([150.0, 150.0])
            .show(ctx, |ui| {
                if let Some(provider) = &app.neural_api_provider {
                    app.tokio_runtime.block_on(async {
                        app.neural_metrics_dashboard.update(provider.as_ref()).await;
                    });
                    app.neural_metrics_dashboard.render(ui);
                } else {
                    ui.label("Neural API not available");
                    ui.label("Start biomeOS nucleus to enable metrics data.");
                }
            });
    }

    if app.show_graph_builder {
        egui::Window::new("Graph Builder")
            .default_width(1200.0)
            .default_height(800.0)
            .default_pos([50.0, 50.0])
            .resizable(true)
            .show(ctx, |ui| {
                if app.neural_api_provider.is_some() {
                    ui.heading("Neural Graph Builder");
                    ui.separator();
                    ui.label("Interactive visual graph construction for Neural API.");
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("Canvas");
                            ui.separator();
                            app.graph_canvas.render(ui, &palette);
                        });
                    });
                    ui.separator();
                    ui.label("Coming soon: Node palette, property editor, and graph management.");
                } else {
                    ui.label("Neural API not available");
                    ui.label("Start biomeOS nucleus to enable Graph Builder.");
                    ui.separator();
                    ui.label("The Graph Builder requires Neural API for graph persistence and execution.");
                }
            });
    }

    for (idx, panel) in app.custom_panels.iter_mut().enumerate() {
        egui::Window::new(panel.title())
            .id(egui::Id::new(format!("custom_panel_{idx}")))
            .default_width(640.0)
            .default_height(480.0)
            .resizable(true)
            .show(ctx, |ui| {
                panel.update();
                panel.render(ui);
            });
    }

    let selected_id_clone = app
        .visual_renderer
        .selected_node()
        .map(std::string::ToString::to_string);
    if let Some(selected_id) = selected_id_clone {
        egui::SidePanel::right("primal_details_panel")
            .default_width(350.0)
            .resizable(true)
            .show(ctx, |ui| {
                crate::app_panels::render_primal_details_panel(
                    ui,
                    &selected_id,
                    &palette,
                    &app.graph,
                    &app.adapter_registry,
                    &mut app.visual_renderer,
                );
            });
    }

    // Live IPC visualization sessions from springs
    if let Some(ref viz_state) = app.visualization_state {
        let has_sessions = {
            let summaries = crate::live_sessions::active_session_summaries(viz_state);
            !summaries.is_empty()
        };
        if has_sessions {
            egui::TopBottomPanel::bottom("live_sessions_panel")
                .default_height(120.0)
                .resizable(true)
                .show(ctx, |ui| {
                    crate::live_sessions::render_sessions_panel(ui, viz_state);
                });
        }
    }

    // Motor-driven panel content (from compositions via motor.panel.update)
    if !app.panel_content_store.is_empty() {
        egui::Window::new("Composition Panels")
            .default_width(350.0)
            .default_pos([200.0, 200.0])
            .resizable(true)
            .show(ctx, |ui| {
                render_motor_panel_content(ui, &app.panel_content_store);
            });
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(tool) = app.tools.visible_tool() {
            tool.render_panel(ui);
        } else {
            app.visual_renderer.render(ui);
        }
    });

    // Motor-driven notifications (from compositions via motor.notification)
    app.notification_queue.drain_expired();
    if !app.notification_queue.is_empty() {
        render_notification_toasts(ctx, &app.notification_queue);
    }
}

/// Render composition-driven panel content pushed via `motor.panel.update`.
fn render_motor_panel_content(ui: &mut egui::Ui, store: &super::motor_state::PanelContentStore) {
    for (panel_key, content) in store.iter() {
        if let Some(title) = &content.title {
            ui.heading(title);
        } else {
            ui.heading(panel_key);
        }
        ui.separator();
        render_json_value(ui, &content.content, 0);
        ui.add_space(8.0);
    }
}

/// Render a JSON value as egui labels (recursive for objects/arrays).
fn render_json_value(ui: &mut egui::Ui, value: &serde_json::Value, depth: usize) {
    const MAX_DEPTH: usize = 4;
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                if depth < MAX_DEPTH && (v.is_object() || v.is_array()) {
                    ui.collapsing(k, |ui| {
                        render_json_value(ui, v, depth + 1);
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.strong(format!("{k}:"));
                        ui.label(format_json_leaf(v));
                    });
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                if depth < MAX_DEPTH && (v.is_object() || v.is_array()) {
                    ui.collapsing(format!("[{i}]"), |ui| {
                        render_json_value(ui, v, depth + 1);
                    });
                } else {
                    ui.label(format!("[{i}] {}", format_json_leaf(v)));
                }
            }
        }
        other => {
            ui.label(format_json_leaf(other));
        }
    }
}

fn format_json_leaf(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Render notification toasts as floating overlays at the top of the screen.
fn render_notification_toasts(ctx: &egui::Context, queue: &super::motor_state::NotificationQueue) {
    let screen = ctx.screen_rect();
    let toast_width = 400.0_f32.min(screen.width() - 40.0);
    let start_x = screen.center().x - toast_width / 2.0;
    let mut y_offset = 40.0;

    for (i, entry) in queue.active().iter().enumerate().take(5) {
        let (bg, text_color) = notification_colors(&entry.level);

        egui::Area::new(egui::Id::new(format!("notification_toast_{i}")))
            .fixed_pos([start_x, y_offset])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(bg)
                    .rounding(6.0)
                    .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                    .show(ui, |ui| {
                        ui.set_width(toast_width);
                        ui.horizontal(|ui| {
                            let icon = match entry.level.as_str() {
                                "error" => "!!",
                                "warn" => "!",
                                "success" => "OK",
                                _ => "i",
                            };
                            ui.colored_label(text_color, icon);
                            ui.colored_label(text_color, &entry.message);
                        });
                    });
            });
        y_offset += 48.0;
    }
    ctx.request_repaint();
}

fn notification_colors(level: &str) -> (egui::Color32, egui::Color32) {
    match level {
        "error" => (
            egui::Color32::from_rgb(120, 30, 30),
            egui::Color32::from_rgb(255, 180, 180),
        ),
        "warn" => (
            egui::Color32::from_rgb(120, 100, 20),
            egui::Color32::from_rgb(255, 230, 140),
        ),
        "success" => (
            egui::Color32::from_rgb(30, 100, 40),
            egui::Color32::from_rgb(180, 255, 190),
        ),
        _ => (
            egui::Color32::from_rgb(40, 55, 80),
            egui::Color32::from_rgb(200, 220, 255),
        ),
    }
}
