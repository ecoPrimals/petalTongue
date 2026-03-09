// SPDX-License-Identifier: AGPL-3.0-only
//! Individual panel builders - render functions for each app panel.

use super::layout;
use crate::accessibility::ColorPalette;
use crate::accessibility_panel::AccessibilityPanel;
use crate::tool_integration::ToolManager;
use petal_tongue_core::{GraphEngine, LayoutAlgorithm, Modality, constants::PRIMAL_NAME};
use petal_tongue_graph::Visual2DRenderer;
use petal_tongue_graph::{AudioFileGenerator, AudioSonificationRenderer};
use std::sync::{Arc, RwLock};

/// Render the top menu bar
/// Returns true if refresh was clicked
pub fn render_top_menu_bar(
    ui: &mut egui::Ui,
    palette: &ColorPalette,
    accessibility_panel: &mut AccessibilityPanel,
    visual_renderer: &mut Visual2DRenderer,
    tools: &mut ToolManager,
    current_layout: &mut LayoutAlgorithm,
    graph: &Arc<RwLock<GraphEngine>>,
    show_dashboard: &mut bool,
    show_controls: &mut bool,
    show_audio_panel: &mut bool,
    show_capability_panel: &mut bool,
    show_neural_proprioception: &mut bool,
    show_neural_metrics: &mut bool,
    show_graph_builder: &mut bool,
) -> bool {
    ui.heading(
        egui::RichText::new("🌸 petalTongue")
            .size(accessibility_panel.scale_font(20.0))
            .color(palette.accent),
    );
    ui.label(
        egui::RichText::new("Universal Representation System")
            .size(accessibility_panel.scale_font(14.0))
            .color(palette.text_dim),
    );

    ui.separator();

    if ui.button("Reset Camera").clicked() {
        visual_renderer.reset_camera();
    }

    ui.separator();

    // Tools menu (capability-based, not hardcoded)
    ui.menu_button("🔧 Tools", |ui| {
        tools.render_tools_menu(ui);
    });

    ui.separator();

    // View menu (panel visibility toggles)
    ui.menu_button("👁️ View", |ui| {
        ui.checkbox(show_dashboard, "System Dashboard");
        ui.checkbox(show_controls, "Controls Panel");
        ui.checkbox(show_audio_panel, "Audio Panel");
        ui.checkbox(show_capability_panel, "Capabilities");
        ui.separator();
        ui.label(egui::RichText::new("Neural API Panels").strong());
        if ui
            .checkbox(show_neural_proprioception, "🧠 Proprioception (P)")
            .changed()
        {
            tracing::info!(
                "Neural proprioception panel toggled: {}",
                show_neural_proprioception
            );
        }
        if ui
            .checkbox(show_neural_metrics, "📊 Metrics Dashboard (M)")
            .changed()
        {
            tracing::info!("Neural metrics dashboard toggled: {}", show_neural_metrics);
        }
        if ui
            .checkbox(show_graph_builder, "🎨 Graph Builder (G)")
            .changed()
        {
            tracing::info!("Graph Builder toggled: {}", show_graph_builder);
        }
    });

    ui.separator();

    layout::render_layout_selector(ui, current_layout, graph);

    ui.separator();

    // Refresh button
    let refresh_clicked = ui.button("🔄 Refresh").clicked();

    ui.separator();

    // Accessibility panel toggle
    if ui.button("♿ Accessibility").clicked() {
        accessibility_panel.show = !accessibility_panel.show;
    }

    ui.separator();

    // Dashboard toggle
    ui.checkbox(show_dashboard, "📊 Dashboard");

    ui.separator();

    ui.checkbox(show_controls, "Controls");
    ui.checkbox(show_audio_panel, "Audio Info");
    ui.checkbox(show_capability_panel, "🔍 Capabilities");

    refresh_clicked
}

/// Render the controls panel (left side)
/// Returns true if refresh was clicked
pub fn render_controls_panel(
    ui: &mut egui::Ui,
    palette: &ColorPalette,
    accessibility_panel: &AccessibilityPanel,
    auto_refresh: &mut bool,
    refresh_interval: &mut f32,
    last_refresh_elapsed: f32,
    show_animation: &mut bool,
    visual_renderer: &mut Visual2DRenderer,
) -> bool {
    ui.heading(egui::RichText::new("⚙️ Controls").size(accessibility_panel.scale_font(18.0)));
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    ui.label(egui::RichText::new("🖱️ Mouse Controls").strong());
    ui.add_space(4.0);
    ui.label("  • Drag: Pan camera");
    ui.label("  • Scroll: Zoom in/out");
    ui.label("  • Click: Select node");

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);
    ui.heading(egui::RichText::new("🎨 Health Legend").size(accessibility_panel.scale_font(16.0)));

    // Use accessibility colors - respects color-blind modes!
    ui.horizontal(|ui| {
        ui.colored_label(palette.healthy, "⬤");
        ui.label("Healthy");
    });
    ui.horizontal(|ui| {
        ui.colored_label(palette.warning, "⬤");
        ui.label("Warning");
    });
    ui.horizontal(|ui| {
        ui.colored_label(palette.error, "⬤");
        ui.label("Critical");
    });
    ui.horizontal(|ui| {
        ui.colored_label(palette.text_dim, "⬤");
        ui.label("Unknown");
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    // Refresh controls
    ui.heading(egui::RichText::new("🔄 Auto-Refresh").size(16.0));
    ui.add_space(4.0);
    ui.checkbox(auto_refresh, "Enabled");
    ui.add(egui::Slider::new(refresh_interval, 1.0..=60.0).text("Interval (s)"));

    ui.label(format!("Last refresh: {last_refresh_elapsed:.1}s ago"));

    let refresh_clicked = ui.button("Refresh Now").clicked();

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);

    // Animation controls
    ui.heading(egui::RichText::new("✨ Animation").size(16.0));
    ui.add_space(4.0);
    if ui
        .checkbox(show_animation, "Flow Particles & Pulses")
        .changed()
    {
        // Update visual renderer animation state
        visual_renderer.set_animation_enabled(*show_animation);
    }
    ui.label(
        egui::RichText::new("Visualizes data flow between primals")
            .size(11.0)
            .color(egui::Color32::GRAY),
    );

    refresh_clicked
}

/// Render the audio information panel (right side)
pub fn render_audio_panel(
    ui: &mut egui::Ui,
    _palette: &ColorPalette,
    _accessibility_panel: &AccessibilityPanel,
    audio_renderer: &mut AudioSonificationRenderer,
    audio_generator: &AudioFileGenerator,
    visual_renderer: &Visual2DRenderer,
    capabilities: &petal_tongue_core::CapabilityDetector,
) {
    ui.heading(egui::RichText::new("🎵 Audio Representation").size(18.0));
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // Check if audio is actually available
    let audio_available = capabilities.is_available(Modality::Audio);
    if !audio_available {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(40, 50, 45))
            .stroke(egui::Stroke::new(
                1.0,
                egui::Color32::from_rgb(100, 200, 150),
            ))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("🔊 Pure Rust Audio Available")
                        .size(14.0)
                        .strong()
                        .color(egui::Color32::from_rgb(150, 255, 200)),
                );
                ui.add_space(6.0);
                if let Some(audio_cap) = capabilities.get_status(Modality::Audio) {
                    ui.label(
                        egui::RichText::new(&audio_cap.reason)
                            .size(12.0)
                            .color(egui::Color32::from_rgb(200, 220, 210)),
                    );
                }
                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new("✅ Audio System: Multi-Tier")
                        .size(13.0)
                        .strong(),
                );
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("1️⃣");
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Pure Rust Tones")
                                .strong()
                                .color(egui::Color32::from_rgb(100, 255, 150)),
                        );
                        ui.label(
                            egui::RichText::new("Always available, no dependencies")
                                .size(10.0)
                                .color(egui::Color32::GRAY),
                        );
                        ui.label(
                            egui::RichText::new("8 UI sounds (success, error, notification, etc.)")
                                .size(10.0)
                                .color(egui::Color32::GRAY),
                        );
                    });
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("2️⃣");
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("User Sound Files").strong());
                        ui.label(
                            egui::RichText::new("Set PETALTONGUE_SOUNDS_DIR=<path>")
                                .size(10.0)
                                .color(egui::Color32::GRAY),
                        );
                        ui.label(
                            egui::RichText::new("Supports WAV, MP3, OGG files")
                                .size(10.0)
                                .color(egui::Color32::GRAY),
                        );
                    });
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("3️⃣");
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Toadstool Synthesis").strong());
                        ui.label(
                            egui::RichText::new("Set TOADSTOOL_URL=http://localhost:port")
                                .size(10.0)
                                .color(egui::Color32::GRAY),
                        );
                        ui.label(
                            egui::RichText::new("Advanced music, voice, soundscapes")
                                .size(10.0)
                                .color(egui::Color32::GRAY),
                        );
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                ui.label(egui::RichText::new("💡 Quick Start:").size(12.0).strong());
                ui.label(
                    egui::RichText::new("Pure Rust audio works NOW (mathematical waveforms)")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(200, 220, 210)),
                );
                ui.label(
                    egui::RichText::new(
                        "For advanced features, connect Toadstool or add sound files",
                    )
                    .size(10.0)
                    .italics()
                    .color(egui::Color32::GRAY),
                );
            });
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
    }

    // Master volume control
    let mut volume = audio_renderer.master_volume();
    ui.horizontal(|ui| {
        ui.label("Master Volume:");
        if ui.add(egui::Slider::new(&mut volume, 0.0..=1.0)).changed() {
            audio_renderer.set_master_volume(volume);
        }
    });

    // Enable/disable toggle
    let mut enabled = audio_renderer.is_enabled();
    ui.horizontal(|ui| {
        ui.label("Audio Enabled:");
        if ui.checkbox(&mut enabled, "").changed() {
            audio_renderer.set_enabled(enabled);
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // Soundscape description
    ui.heading(egui::RichText::new("🎼 Soundscape").size(16.0));
    ui.add_space(4.0);
    let description = audio_renderer.describe_soundscape();
    ui.label(
        egui::RichText::new(description)
            .size(13.0)
            .color(egui::Color32::from_rgb(200, 200, 200)),
    );

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // Node-level audio info
    if let Some(selected_id) = visual_renderer.selected_node() {
        ui.heading(egui::RichText::new("🎯 Selected Node").size(16.0));
        ui.add_space(4.0);
        if let Some(node_desc) = audio_renderer.describe_node_audio(selected_id) {
            ui.label(
                egui::RichText::new(node_desc)
                    .size(13.0)
                    .color(egui::Color32::from_rgb(255, 230, 150)),
            );
        }
    } else {
        ui.heading(
            egui::RichText::new("🎯 Selected Node")
                .size(16.0)
                .color(egui::Color32::GRAY),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Click a node to hear its audio representation")
                .size(12.0)
                .italics()
                .color(egui::Color32::GRAY),
        );
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // Instrument legend
    ui.heading(egui::RichText::new("🎹 Instrument Mapping").size(16.0));
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("🐻 Security → Deep Bass")
            .color(egui::Color32::from_rgb(100, 150, 255)),
    );
    ui.label(
        egui::RichText::new("🍄 Compute → Rhythmic Drums")
            .color(egui::Color32::from_rgb(255, 200, 100)),
    );
    ui.label(
        egui::RichText::new("🐦 Discovery → Light Chimes")
            .color(egui::Color32::from_rgb(150, 255, 150)),
    );
    ui.label(
        egui::RichText::new("🏠 Storage → Sustained Strings")
            .color(egui::Color32::from_rgb(255, 150, 255)),
    );
    ui.label(
        egui::RichText::new("🐿️ AI → High Synth").color(egui::Color32::from_rgb(255, 100, 100)),
    );

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // Export audio button
    ui.heading(egui::RichText::new("💾 Export Audio").size(16.0));
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Export the current soundscape to a WAV file")
            .size(12.0)
            .color(egui::Color32::from_rgb(180, 180, 180)),
    );
    ui.add_space(6.0);

    if ui.button("💾 Export Soundscape to WAV").clicked() {
        // Export graph soundscape
        let soundscape = audio_renderer.generate_audio_attributes();

        let filepath = std::path::PathBuf::from("graph_soundscape.wav");
        if let Err(e) = audio_generator.export_soundscape(&filepath, &soundscape, 3.0) {
            tracing::error!("Failed to export soundscape: {}", e);
        } else {
            tracing::info!("Exported soundscape to: {}", filepath.display());
        }
    }

    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("(File will be saved to ./audio_export/)")
            .size(10.0)
            .italics()
            .color(egui::Color32::GRAY),
    );
}

/// Render the capability panel (modality status window)
pub fn render_capability_panel(
    ctx: &egui::Context,
    _palette: &ColorPalette,
    capabilities: &petal_tongue_core::CapabilityDetector,
) {
    egui::Window::new("🔍 Modality Capabilities")
        .default_width(500.0)
        .default_pos([400.0, 100.0])
        .show(ctx, |ui| {
            ui.heading(
                egui::RichText::new(format!("{PRIMAL_NAME} Self-Awareness")).size(16.0),
            );
            ui.add_space(8.0);
            ui.label("This system knows what it can actually do:");
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);

            for cap in capabilities.get_all() {
                let (icon, color) = match cap.status {
                    petal_tongue_core::ModalityStatus::Available => {
                        ("✅", egui::Color32::from_rgb(100, 255, 100))
                    }
                    petal_tongue_core::ModalityStatus::NotInitialized => {
                        ("⚠️", egui::Color32::from_rgb(255, 200, 100))
                    }
                    petal_tongue_core::ModalityStatus::Unavailable => {
                        ("❌", egui::Color32::from_rgb(255, 100, 100))
                    }
                    petal_tongue_core::ModalityStatus::Disabled => {
                        ("🔇", egui::Color32::from_rgb(150, 150, 150))
                    }
                };

                let tested_text = if cap.tested { "tested" } else { "not tested" };

                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(40, 40, 45))
                    .stroke(egui::Stroke::new(1.0, color))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(icon).size(24.0));
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{:?}", cap.modality))
                                        .size(14.0)
                                        .strong()
                                        .color(color),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{:?} ({})",
                                        cap.status, tested_text
                                    ))
                                    .size(11.0)
                                    .color(egui::Color32::GRAY),
                                );
                            });
                        });
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(&cap.reason)
                                .size(12.0)
                                .color(egui::Color32::from_rgb(200, 200, 200)),
                        );
                    });
                ui.add_space(8.0);
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(egui::RichText::new("💡 Why This Matters").size(14.0).strong());
            ui.add_space(4.0);
            ui.label("In critical situations (wartime AR, disaster response, accessibility),\nfalse capability claims are dangerous. This system is honest about what it can do.");
        });
}

pub use super::primal_details::render_primal_details_panel;
