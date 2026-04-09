// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::accessibility::{ColorPalette, ColorScheme};
use crate::tool_integration::ToolManager;
use petal_tongue_core::{CapabilityDetector, GraphEngine, ModalityStatus};
use petal_tongue_graph::Visual2DRenderer;
use std::sync::{Arc, RwLock};

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
fn audio_tier_label_discovered_synthesis() {
    assert_eq!(
        audio_tier_label(true, false),
        "Active tier: Discovered Synthesis"
    );
    assert_eq!(
        audio_tier_label(true, true),
        "Active tier: Discovered Synthesis"
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

#[test]
fn render_capability_panel_headless() {
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let capabilities = CapabilityDetector::default();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        render_capability_panel(ctx, &palette, &capabilities);
    });
}

#[test]
fn render_top_menu_bar_headless() {
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let mut accessibility_panel = crate::accessibility_panel::AccessibilityPanel::default();
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let mut visual_renderer = Visual2DRenderer::new(graph.clone());
    let mut tools = ToolManager::new();
    let mut current_layout = petal_tongue_core::LayoutAlgorithm::ForceDirected;
    let mut show_dashboard = false;
    let mut show_controls = false;
    let mut show_audio_panel = false;
    let mut show_capability_panel = false;
    let mut show_neural_proprioception = false;
    let mut show_neural_metrics = false;
    let mut show_graph_builder = false;

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            let _ = render_top_menu_bar(
                ui,
                &palette,
                &mut accessibility_panel,
                &mut visual_renderer,
                &mut tools,
                &mut current_layout,
                &graph,
                &mut show_dashboard,
                &mut show_controls,
                &mut show_audio_panel,
                &mut show_capability_panel,
                &mut show_neural_proprioception,
                &mut show_neural_metrics,
                &mut show_graph_builder,
            );
        });
    });
}

#[test]
fn render_controls_panel_headless() {
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let accessibility_panel = crate::accessibility_panel::AccessibilityPanel::default();
    let mut auto_refresh = false;
    let mut refresh_interval = 5.0;
    let last_refresh_elapsed = 2.5;
    let mut show_animation = true;
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let mut visual_renderer = Visual2DRenderer::new(graph);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::SidePanel::left("controls").show(ctx, |ui| {
            let _ = render_controls_panel(
                ui,
                &palette,
                &accessibility_panel,
                &mut auto_refresh,
                &mut refresh_interval,
                last_refresh_elapsed,
                &mut show_animation,
                &mut visual_renderer,
            );
        });
    });
}

#[test]
fn render_audio_panel_headless() {
    use petal_tongue_graph::AudioFileGenerator;
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let accessibility_panel = crate::accessibility_panel::AccessibilityPanel::default();
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let mut audio_renderer = petal_tongue_graph::AudioSonificationRenderer::new(graph.clone());
    let audio_generator = AudioFileGenerator::new();
    let visual_renderer = Visual2DRenderer::new(graph);
    let capabilities = CapabilityDetector::default();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::SidePanel::right("audio").show(ctx, |ui| {
            render_audio_panel(
                ui,
                &palette,
                &accessibility_panel,
                &mut audio_renderer,
                &audio_generator,
                &visual_renderer,
                &capabilities,
            );
        });
    });
}

#[test]
fn render_audio_panel_with_selected_node() {
    use petal_tongue_core::{PrimalHealthStatus, PrimalInfo};
    use petal_tongue_graph::AudioFileGenerator;
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let accessibility_panel = crate::accessibility_panel::AccessibilityPanel::default();
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    {
        let mut g = graph.write().unwrap();
        g.add_node(PrimalInfo::new(
            "test-node",
            "Test",
            "compute",
            "test://local",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ));
    }
    let mut audio_renderer = petal_tongue_graph::AudioSonificationRenderer::new(graph.clone());
    let audio_generator = AudioFileGenerator::new();
    let mut visual_renderer = Visual2DRenderer::new(graph);
    visual_renderer.set_selected_node(Some("test-node".to_string()));
    let capabilities = CapabilityDetector::default();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::SidePanel::right("audio").show(ctx, |ui| {
            render_audio_panel(
                ui,
                &palette,
                &accessibility_panel,
                &mut audio_renderer,
                &audio_generator,
                &visual_renderer,
                &capabilities,
            );
        });
    });
}

#[test]
fn render_top_menu_bar_refresh_clicked() {
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let mut accessibility_panel = crate::accessibility_panel::AccessibilityPanel::default();
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let mut visual_renderer = Visual2DRenderer::new(graph.clone());
    let mut tools = ToolManager::new();
    let mut current_layout = petal_tongue_core::LayoutAlgorithm::ForceDirected;
    let mut show_dashboard = false;
    let mut show_controls = false;
    let mut show_audio_panel = false;
    let mut show_capability_panel = false;
    let mut show_neural_proprioception = false;
    let mut show_neural_metrics = false;
    let mut show_graph_builder = false;

    let ctx = egui::Context::default();
    let mut refresh_clicked = false;
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            refresh_clicked = render_top_menu_bar(
                ui,
                &palette,
                &mut accessibility_panel,
                &mut visual_renderer,
                &mut tools,
                &mut current_layout,
                &graph,
                &mut show_dashboard,
                &mut show_controls,
                &mut show_audio_panel,
                &mut show_capability_panel,
                &mut show_neural_proprioception,
                &mut show_neural_metrics,
                &mut show_graph_builder,
            );
        });
    });
    assert!(!refresh_clicked);
}

#[test]
fn render_controls_panel_animation_changed() {
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let accessibility_panel = crate::accessibility_panel::AccessibilityPanel::default();
    let mut auto_refresh = false;
    let mut refresh_interval = 5.0;
    let mut show_animation = false;
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let mut visual_renderer = Visual2DRenderer::new(graph);

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::SidePanel::left("controls").show(ctx, |ui| {
            let _ = render_controls_panel(
                ui,
                &palette,
                &accessibility_panel,
                &mut auto_refresh,
                &mut refresh_interval,
                2.5,
                &mut show_animation,
                &mut visual_renderer,
            );
        });
    });
}
