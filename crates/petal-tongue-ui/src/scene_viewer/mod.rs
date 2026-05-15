// SPDX-License-Identifier: AGPL-3.0-or-later
//! Live scientific visualization of IPC-pushed DataBindings.
//!
//! Reads compiled `grammar_scenes` from the shared `VisualizationState`,
//! groups them by session, and renders each as a `SceneWidget` tile in a
//! responsive grid. This is petalTongue's first-class scientific renderer.
//!
//! Entrance animations use petalTongue's `Easing` functions for manim-style
//! fade-in and stroke-draw effects when new sessions arrive.

mod interaction;
mod parameters;

use crate::live_sessions::{domain_color_rgb, format_session_age};
use crate::scene_bridge::{FrameHitMap, SceneWidget};
use crate::scene_interaction::{SceneInteractionState, ViewCamera};
use petal_tongue_ipc::{InteractionSubscriberRegistry, VisualizationState};
use petal_tongue_scene::animation::Easing;
use petal_tongue_scene::render_plan::RenderPlan;
use std::sync::{Arc, RwLock};

const ENTRANCE_DURATION_SECS: f64 = 0.6;
const STAGGER_DELAY_SECS: f64 = 0.15;

/// Render the scene viewer into the given `Ui`.
///
/// When `session_filter` is `Some(key)`, a single binding is shown full-size.
/// When `None`, all sessions are rendered as a tiled grid.
///
/// The optional `interaction_subs` connects scene selections to the IPC
/// interaction event system so springs can react to user picks.
pub fn render(
    ui: &mut egui::Ui,
    viz_state: &Option<Arc<RwLock<VisualizationState>>>,
    session_filter: Option<&str>,
) {
    render_with_interaction(ui, viz_state, session_filter, None);
}

/// Extended render that bridges scene clicks to IPC interaction events.
pub fn render_with_interaction(
    ui: &mut egui::Ui,
    viz_state: &Option<Arc<RwLock<VisualizationState>>>,
    session_filter: Option<&str>,
    interaction_subs: Option<&Arc<RwLock<InteractionSubscriberRegistry>>>,
) {
    let Some(state_arc) = viz_state else {
        render_empty(ui, "No IPC server connected");
        return;
    };
    let Ok(state) = state_arc.read() else {
        render_empty(ui, "Visualization state locked");
        return;
    };

    if state.grammar_scenes.is_empty() {
        render_empty(ui, "No visualization sessions \u{2014} push data with litho visualize --format dashboard");
        return;
    }

    render_summary_bar(ui, &state);
    ui.separator();

    if let Some(key) = session_filter {
        render_expanded(ui, &state, key, interaction_subs);
    } else {
        render_tiled(ui, &state, interaction_subs);
    }
}

fn render_empty(ui: &mut egui::Ui, message: &str) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.label(
                egui::RichText::new("Scene Viewer")
                    .size(24.0)
                    .color(egui::Color32::from_gray(120)),
            );
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(message)
                    .size(14.0)
                    .color(egui::Color32::from_gray(100)),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Press V to return to graph view")
                    .size(11.0)
                    .color(egui::Color32::from_gray(80)),
            );
        });
    });
}

fn render_summary_bar(ui: &mut egui::Ui, state: &VisualizationState) {
    let session_count = state.sessions.len();
    let scene_count = state.grammar_scenes.len();
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Scene Viewer")
                .size(14.0)
                .strong(),
        );
        ui.separator();
        ui.label(
            egui::RichText::new(format!(
                "{session_count} session(s), {scene_count} binding(s)"
            ))
            .size(12.0)
            .color(egui::Color32::from_gray(160)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new("V: toggle view")
                    .size(11.0)
                    .color(egui::Color32::from_gray(100)),
            );
        });
    });
}

/// Render a single binding expanded to full central panel size.
fn render_expanded(
    ui: &mut egui::Ui,
    state: &VisualizationState,
    key: &str,
    interaction_subs: Option<&Arc<RwLock<InteractionSubscriberRegistry>>>,
) {
    if let Some(compiled) = state.grammar_scenes.get(key) {
        let camera_id = egui::Id::new("scene_camera").with(key);
        let interaction_id = egui::Id::new("scene_ix").with(key);

        ui.horizontal(|ui| {
            let chart_title = compiled.grammar.title.as_deref().unwrap_or(key);
            ui.label(egui::RichText::new(chart_title).size(12.0).strong());
            ui.separator();
            let cam: ViewCamera = ui.data(|d| d.get_temp(camera_id).unwrap_or_default());
            ui.label(
                egui::RichText::new(format!("{:.0}%", cam.zoom * 100.0))
                    .size(10.0)
                    .color(egui::Color32::from_gray(140)),
            );
            if ui.small_button("Fit").clicked() {
                ui.data_mut(|d| d.insert_temp(camera_id, ViewCamera::default()));
            }
            if ui.small_button("Clear selection").clicked() {
                ui.data_mut(|d| d.insert_temp(interaction_id, SceneInteractionState::default()));
            }
        });

        let override_plan = parameters::render_parameter_strip(ui, compiled, key);

        let default_plan = RenderPlan::new(compiled.scene.clone(), compiled.grammar.clone());
        let plan = override_plan.as_ref().unwrap_or(&default_plan);
        let size = ui.available_size() - egui::vec2(0.0, 32.0);

        let mut hit_map = FrameHitMap::default();
        let response = SceneWidget::new(plan)
            .desired_size(size)
            .with_hit_map(&mut hit_map)
            .show(ui);

        interaction::handle_camera_input(ui, &response, camera_id);
        interaction::handle_scene_interaction_with_camera(
            ui, &response, &hit_map, interaction_id, camera_id,
            key, interaction_subs,
        );
        interaction::draw_selection_overlays(ui, &response, &hit_map, interaction_id);
        if interaction::handle_transition_animation(ui, &response, compiled, key) {
            ui.ctx().request_repaint();
        }
        interaction::render_detail_strip(ui, interaction_id);
    } else {
        ui.label(
            egui::RichText::new(format!("Binding '{key}' not found"))
                .color(egui::Color32::from_rgb(200, 100, 100)),
        );
    }
}

/// Render all sessions as a tiled grid of SceneWidget tiles.
fn render_tiled(
    ui: &mut egui::Ui,
    state: &VisualizationState,
    interaction_subs: Option<&Arc<RwLock<InteractionSubscriberRegistry>>>,
) {
    let viewer_id = egui::Id::new("scene_viewer_state");
    let available_width = ui.available_width();
    let columns = if available_width > 900.0 { 3 } else if available_width > 500.0 { 2 } else { 1 };
    let tile_width = (available_width - (columns as f32 - 1.0) * 8.0) / columns as f32;
    let tile_height = tile_width * 0.65;
    let tile_size = egui::Vec2::new(tile_width, tile_height);

    let mut session_groups: std::collections::BTreeMap<String, Vec<(String, &petal_tongue_ipc::CompiledBinding)>> =
        std::collections::BTreeMap::new();
    for (key, compiled) in &state.grammar_scenes {
        let session_id = key.split(':').next().unwrap_or(key).to_string();
        session_groups
            .entry(session_id)
            .or_default()
            .push((key.clone(), compiled));
    }

    let mut any_animating = false;
    let mut global_idx = 0usize;

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for (session_id, bindings) in &session_groups {
                let session_meta = state.sessions.get(session_id);
                let title = session_meta.map_or(session_id.as_str(), |s| s.title.as_str());
                let domain = session_meta.and_then(|s| s.domain.clone());
                let (r, g, b) = domain_color_rgb(&domain);
                let age_text = session_meta
                    .map(|s| format_session_age(s.updated_at.elapsed().as_secs_f32()))
                    .unwrap_or_default();

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("\u{25cf}")
                            .color(egui::Color32::from_rgb(r, g, b)),
                    );
                    ui.label(egui::RichText::new(title).size(13.0).strong());
                    ui.label(
                        egui::RichText::new(format!(
                            "({} binding{})",
                            bindings.len(),
                            if bindings.len() == 1 { "" } else { "s" }
                        ))
                        .size(11.0)
                        .color(egui::Color32::from_gray(140)),
                    );
                    if !age_text.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("\u{2022} {age_text}"))
                                .size(11.0)
                                .color(egui::Color32::from_gray(120)),
                        );
                    }
                });
                ui.add_space(4.0);

                ui.horizontal_wrapped(|ui| {
                    for (key, compiled) in bindings {
                        let anim_id = viewer_id.with(key);
                        let current_time = ui.input(|i| i.time);
                        let anim_start: f64 = ui
                            .data_mut(|d| *d.get_temp_mut_or_insert_with(anim_id, || current_time));
                        let elapsed = ui.input(|i| i.time) - anim_start;
                        let delayed = elapsed - global_idx as f64 * STAGGER_DELAY_SECS;
                        let opacity = if delayed <= 0.0 {
                            0.0_f32
                        } else {
                            let t = (delayed / ENTRANCE_DURATION_SECS).min(1.0);
                            Easing::EaseOut.apply(t) as f32
                        };
                        if opacity < 0.999 {
                            any_animating = true;
                        }

                        let plan = RenderPlan::new(
                            compiled.scene.clone(),
                            compiled.grammar.clone(),
                        );

                        let bg = egui::Color32::from_rgba_unmultiplied(25, 25, 25, (opacity * 255.0) as u8);
                        let stroke_alpha = (opacity * 255.0) as u8;
                        let frame = egui::Frame::none()
                            .fill(bg)
                            .rounding(4.0)
                            .inner_margin(6.0)
                            .stroke(egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgba_unmultiplied(50, 50, 50, stroke_alpha),
                            ));

                        frame.show(ui, |ui| {
                            ui.set_opacity(opacity);
                            let chart_title = compiled
                                .grammar
                                .title
                                .as_deref()
                                .unwrap_or(key.as_str());
                            ui.label(
                                egui::RichText::new(chart_title)
                                    .size(11.0)
                                    .color(egui::Color32::from_gray(180)),
                            );
                            let mut hit_map = FrameHitMap::default();
                            let response = SceneWidget::new(&plan)
                                .desired_size(tile_size)
                                .with_hit_map(&mut hit_map)
                                .show(ui);

                            let interaction_id = egui::Id::new("scene_ix").with(key);
                            interaction::handle_scene_interaction(
                                ui, &response, &hit_map, interaction_id,
                                key, interaction_subs,
                            );
                            interaction::draw_selection_overlays(ui, &response, &hit_map, interaction_id);
                            if interaction::handle_transition_animation(ui, &response, compiled, key) {
                                any_animating = true;
                            }
                        });

                        global_idx += 1;
                    }
                });
                ui.add_space(12.0);
            }
        });

    if any_animating {
        ui.ctx().request_repaint();
    }
}
