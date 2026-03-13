// SPDX-License-Identifier: AGPL-3.0-only
//! Panel routing and layout.
//!
//! Renders all UI panels based on visibility state.
//! Extracted from app/mod.rs for domain separation.

use super::PetalTongueApp;

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

    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(tool) = app.tools.visible_tool() {
            tool.render_panel(ui);
        } else {
            app.visual_renderer.render(ui);
        }
    });
}
