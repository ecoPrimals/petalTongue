// SPDX-License-Identifier: AGPL-3.0-only
//! Individual panel builders - render functions for each app panel.

use super::layout;
use crate::accessibility::ColorPalette;
use crate::accessibility_panel::AccessibilityPanel;
use crate::tool_integration::ToolManager;
use petal_tongue_core::ModalityStatus;
use petal_tongue_core::{GraphEngine, LayoutAlgorithm, Modality, constants::PRIMAL_NAME};
use petal_tongue_graph::Visual2DRenderer;
use petal_tongue_graph::{AudioFileGenerator, AudioSonificationRenderer};
use std::sync::{Arc, RwLock};

#[must_use]
const fn modality_status_icon_and_color(status: ModalityStatus) -> (&'static str, egui::Color32) {
    match status {
        ModalityStatus::Available => ("✅", egui::Color32::from_rgb(100, 255, 100)),
        ModalityStatus::NotInitialized => ("⚠️", egui::Color32::from_rgb(255, 200, 100)),
        ModalityStatus::Unavailable => ("❌", egui::Color32::from_rgb(255, 100, 100)),
        ModalityStatus::Disabled => ("🔇", egui::Color32::from_rgb(150, 150, 150)),
    }
}

#[must_use]
const fn modality_tested_text(tested: bool) -> &'static str {
    if tested { "tested" } else { "not tested" }
}

#[must_use]
pub const fn audio_tier_label(has_toadstool: bool, has_user_sounds: bool) -> &'static str {
    if has_toadstool {
        "Active tier: Toadstool Synthesis"
    } else if has_user_sounds {
        "Active tier: User Sound Files"
    } else {
        "Active tier: Pure Rust Tones"
    }
}

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

                let has_user_sounds = std::env::var("PETALTONGUE_SOUNDS_DIR").is_ok();
                let has_toadstool = std::env::var("TOADSTOOL_URL").is_ok();
                let tier_label = audio_tier_label(has_toadstool, has_user_sounds);

                ui.label(
                    egui::RichText::new(tier_label)
                        .size(13.0)
                        .strong()
                        .color(egui::Color32::from_rgb(100, 255, 150)),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new("Tiers: Pure Rust (built-in) → User sounds → Toadstool")
                        .size(10.0)
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

    // Instrument mapping — adapts dynamically per node via describe_node_audio()
    ui.heading(egui::RichText::new("🎹 Instrument Mapping").size(16.0));
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Audio adapts to each primal's type and capabilities.")
            .size(12.0)
            .color(egui::Color32::from_rgb(200, 200, 200)),
    );
    ui.label(
        egui::RichText::new("Select a node to hear its unique representation.")
            .size(11.0)
            .italics()
            .color(egui::Color32::GRAY),
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
        let soundscape = audio_renderer.generate_audio_attributes();

        let export_dir = std::path::PathBuf::from("audio_export");
        if !export_dir.exists() {
            let _ = std::fs::create_dir_all(&export_dir);
        }
        let filepath = export_dir.join("graph_soundscape.wav");
        if let Err(e) = audio_generator.export_soundscape(&filepath, &soundscape, 3.0) {
            tracing::error!("Failed to export soundscape: {}", e);
        } else {
            tracing::info!("Exported soundscape to: {}", filepath.display());
        }
    }

    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("(Saves to ./audio_export/graph_soundscape.wav)")
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
                let (icon, color) = modality_status_icon_and_color(cap.status);
                let tested_text = modality_tested_text(cap.tested);

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
            ui.label("Honest self-assessment ensures reliability.\nThis system reports only capabilities it has verified.");
        });
}

pub use super::primal_details::render_primal_details_panel;

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::ModalityStatus;

    #[test]
    fn modality_status_icon_available() {
        let (icon, _) = modality_status_icon_and_color(ModalityStatus::Available);
        assert_eq!(icon, "✅");
    }

    #[test]
    fn modality_status_icon_unavailable() {
        let (icon, _) = modality_status_icon_and_color(ModalityStatus::Unavailable);
        assert_eq!(icon, "❌");
    }

    #[test]
    fn modality_status_icon_disabled() {
        let (icon, _) = modality_status_icon_and_color(ModalityStatus::Disabled);
        assert_eq!(icon, "🔇");
    }

    #[test]
    fn modality_tested_text_true() {
        assert_eq!(modality_tested_text(true), "tested");
    }

    #[test]
    fn modality_tested_text_false() {
        assert_eq!(modality_tested_text(false), "not tested");
    }

    #[test]
    fn modality_status_icon_not_initialized() {
        let (icon, _) = modality_status_icon_and_color(ModalityStatus::NotInitialized);
        assert_eq!(icon, "⚠️");
    }

    #[test]
    fn modality_status_colors_distinct() {
        let (_, avail) = modality_status_icon_and_color(ModalityStatus::Available);
        let (_, unavail) = modality_status_icon_and_color(ModalityStatus::Unavailable);
        let (_, disabled) = modality_status_icon_and_color(ModalityStatus::Disabled);
        assert_ne!(avail, unavail);
        assert_ne!(avail, disabled);
        assert_ne!(unavail, disabled);
    }

    #[test]
    fn primal_name_constant_matches_capability_panel() {
        assert_eq!(PRIMAL_NAME, "petalTongue");
    }

    #[test]
    fn modality_status_icon_and_color_all_variants() {
        let (icon, _) = modality_status_icon_and_color(ModalityStatus::Available);
        assert_eq!(icon, "✅");
        let (icon, _) = modality_status_icon_and_color(ModalityStatus::NotInitialized);
        assert_eq!(icon, "⚠️");
        let (icon, _) = modality_status_icon_and_color(ModalityStatus::Unavailable);
        assert_eq!(icon, "❌");
        let (icon, _) = modality_status_icon_and_color(ModalityStatus::Disabled);
        assert_eq!(icon, "🔇");
    }

    #[test]
    fn audio_tier_label_toadstool() {
        assert_eq!(
            audio_tier_label(true, false),
            "Active tier: Toadstool Synthesis"
        );
        assert_eq!(
            audio_tier_label(true, true),
            "Active tier: Toadstool Synthesis"
        );
    }

    #[test]
    fn audio_tier_label_user_sounds() {
        assert_eq!(
            audio_tier_label(false, true),
            "Active tier: User Sound Files"
        );
    }

    #[test]
    fn audio_tier_label_pure_rust() {
        assert_eq!(
            audio_tier_label(false, false),
            "Active tier: Pure Rust Tones"
        );
    }
}
