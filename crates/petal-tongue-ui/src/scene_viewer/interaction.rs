// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scene interaction: camera control, click/hover handling, selection overlays.

use crate::scene_bridge::FrameHitMap;
use crate::scene_interaction::{SceneInteractionState, ViewCamera};
use petal_tongue_ipc::{InteractionApplyRequest, InteractionSubscriberRegistry};
use petal_tongue_scene::animation::Easing;
use std::sync::{Arc, RwLock};

pub(crate) const HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::from_rgb(100, 180, 255);
pub(crate) const HOVER_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(100, 180, 255, 80);
pub(crate) const TRANSITION_DURATION_SECS: f64 = 0.35;

/// Handle scroll-zoom and drag-pan on the camera (expanded view only).
pub(crate) fn handle_camera_input(
    ui: &mut egui::Ui,
    response: &egui::Response,
    camera_id: egui::Id,
) {
    let mut cam: ViewCamera =
        ui.data_mut(|d| d.get_temp::<ViewCamera>(camera_id).unwrap_or_default());

    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > 0.1 {
            let factor = if scroll > 0.0 { 1.1 } else { 1.0 / 1.1 };
            let focal = ui
                .input(|i| i.pointer.hover_pos())
                .unwrap_or(response.rect.center());
            cam.zoom_at(focal - response.rect.min.to_vec2(), factor);
            ui.ctx().request_repaint();
        }
    }

    if response.dragged_by(egui::PointerButton::Middle)
        || (response.dragged_by(egui::PointerButton::Primary)
            && ui.input(|i| i.modifiers.ctrl))
    {
        cam.offset += response.drag_delta();
        ui.ctx().request_repaint();
    }

    ui.data_mut(|d| d.insert_temp(camera_id, cam));
}

/// Like `handle_scene_interaction` but transforms hit-map queries through the camera.
pub(crate) fn handle_scene_interaction_with_camera(
    ui: &mut egui::Ui,
    response: &egui::Response,
    hit_map: &FrameHitMap,
    state_id: egui::Id,
    camera_id: egui::Id,
    binding_key: &str,
    interaction_subs: Option<&Arc<RwLock<InteractionSubscriberRegistry>>>,
) {
    let cam: ViewCamera =
        ui.data_mut(|d| d.get_temp::<ViewCamera>(camera_id).unwrap_or_default());

    let mut ix_state: SceneInteractionState = ui.data_mut(|d| {
        d.get_temp::<SceneInteractionState>(state_id)
            .unwrap_or_default()
    });

    if response.hovered() {
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            let scene_pt = cam.screen_to_scene(pos, response.rect.min);
            if let Some(prov) = hit_map.query(scene_pt.x, scene_pt.y) {
                ix_state.hovered_id = prov.data_id.clone();
                ix_state.hovered_prov = Some(prov.clone());

                egui::show_tooltip_at_pointer(
                    ui.ctx(),
                    ui.layer_id(),
                    egui::Id::new("scene_tooltip"),
                    |ui| {
                        if let Some(ref data_id) = prov.data_id {
                            ui.label(egui::RichText::new(data_id).size(11.0).strong());
                        } else {
                            ui.label(egui::RichText::new(&prov.node_id).size(11.0).strong());
                        }
                        ui.label(
                            egui::RichText::new(format!(
                                "({:.2}, {:.2})",
                                prov.world_x, prov.world_y
                            ))
                            .size(10.0)
                            .color(egui::Color32::from_gray(140)),
                        );
                    },
                );
            } else {
                ix_state.hovered_id = None;
                ix_state.hovered_prov = None;
            }
        }
    } else {
        ix_state.hovered_id = None;
        ix_state.hovered_prov = None;
    }

    if response.clicked()
        && !ui.input(|i| i.modifiers.ctrl)
    {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let scene_pt = cam.screen_to_scene(pos, response.rect.min);
            if let Some(prov) = hit_map.query(scene_pt.x, scene_pt.y) {
                if let Some(ref data_id) = prov.data_id {
                    let selected = ix_state.toggle_selection(data_id);
                    fire_interaction_event(
                        interaction_subs,
                        if selected { "select" } else { "deselect" },
                        data_id,
                        binding_key,
                    );
                }
            } else {
                if !ix_state.selected_ids.is_empty() {
                    fire_interaction_event(interaction_subs, "deselect", "*", binding_key);
                }
                ix_state.clear_selection();
            }
        }
    }

    ui.data_mut(|d| d.insert_temp(state_id, ix_state));
}

/// Process click and hover events against the hit map (tiled view).
pub(crate) fn handle_scene_interaction(
    ui: &mut egui::Ui,
    response: &egui::Response,
    hit_map: &FrameHitMap,
    state_id: egui::Id,
    binding_key: &str,
    interaction_subs: Option<&Arc<RwLock<InteractionSubscriberRegistry>>>,
) {
    let mut ix_state: SceneInteractionState = ui.data_mut(|d| {
        d.get_temp::<SceneInteractionState>(state_id)
            .unwrap_or_default()
    });

    if response.hovered() {
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            let local = pos - response.rect.min.to_vec2();
            if let Some(prov) = hit_map.query(local.x, local.y) {
                ix_state.hovered_id = prov.data_id.clone();
                ix_state.hovered_prov = Some(prov.clone());

                egui::show_tooltip_at_pointer(
                    ui.ctx(),
                    ui.layer_id(),
                    egui::Id::new("scene_tooltip"),
                    |ui| {
                        if let Some(ref data_id) = prov.data_id {
                            ui.label(egui::RichText::new(data_id).size(11.0).strong());
                        } else {
                            ui.label(egui::RichText::new(&prov.node_id).size(11.0).strong());
                        }
                        ui.label(
                            egui::RichText::new(format!(
                                "({:.2}, {:.2})",
                                prov.world_x, prov.world_y
                            ))
                            .size(10.0)
                            .color(egui::Color32::from_gray(140)),
                        );
                        if ix_state.is_selected(
                            prov.data_id.as_deref().unwrap_or(""),
                        ) {
                            ui.label(
                                egui::RichText::new("selected")
                                    .size(9.0)
                                    .color(HIGHLIGHT_COLOR),
                            );
                        }
                    },
                );
            } else {
                ix_state.hovered_id = None;
                ix_state.hovered_prov = None;
            }
        }
    } else {
        ix_state.hovered_id = None;
        ix_state.hovered_prov = None;
    }

    if response.clicked() {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let local = pos - response.rect.min.to_vec2();
            if let Some(prov) = hit_map.query(local.x, local.y) {
                if let Some(ref data_id) = prov.data_id {
                    let selected = ix_state.toggle_selection(data_id);
                    fire_interaction_event(
                        interaction_subs,
                        if selected { "select" } else { "deselect" },
                        data_id,
                        binding_key,
                    );
                }
            } else {
                if !ix_state.selected_ids.is_empty() {
                    fire_interaction_event(interaction_subs, "deselect", "*", binding_key);
                }
                ix_state.clear_selection();
            }
        }
    }

    ui.data_mut(|d| d.insert_temp(state_id, ix_state));
}

/// Draw highlight overlays for selected and hovered primitives.
pub(crate) fn draw_selection_overlays(
    ui: &egui::Ui,
    response: &egui::Response,
    hit_map: &FrameHitMap,
    state_id: egui::Id,
) {
    let ix_state: SceneInteractionState = ui.data(|d| {
        d.get_temp::<SceneInteractionState>(state_id)
            .unwrap_or_default()
    });

    if ix_state.selected_ids.is_empty() && ix_state.hovered_id.is_none() {
        return;
    }

    let painter = ui.painter();
    let origin = response.rect.min.to_vec2();

    for (rect, prov) in hit_map.iter() {
        let data_id = prov.data_id.as_deref().unwrap_or("");
        if data_id.is_empty() {
            continue;
        }

        let offset_rect = egui::Rect::from_min_max(
            rect.min - origin + response.rect.min.to_vec2(),
            rect.max - origin + response.rect.min.to_vec2(),
        );

        if ix_state.is_selected(data_id) {
            painter.rect_stroke(
                offset_rect.expand(1.0),
                2.0,
                egui::Stroke::new(2.0, HIGHLIGHT_COLOR),
            );
        } else if ix_state.hovered_id.as_deref() == Some(data_id) {
            painter.rect_filled(offset_rect.expand(0.5), 1.0, HOVER_COLOR);
        }
    }
}

/// Render a compact detail strip showing the currently selected element(s).
pub(crate) fn render_detail_strip(ui: &mut egui::Ui, state_id: egui::Id) {
    let ix_state: SceneInteractionState = ui.data(|d| {
        d.get_temp::<SceneInteractionState>(state_id)
            .unwrap_or_default()
    });

    if ix_state.selected_ids.is_empty() {
        return;
    }

    ui.add_space(4.0);
    egui::Frame::none()
        .fill(egui::Color32::from_gray(30))
        .rounding(3.0)
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Selected:")
                        .size(10.0)
                        .color(HIGHLIGHT_COLOR),
                );
                for sel_id in &ix_state.selected_ids {
                    ui.label(
                        egui::RichText::new(sel_id)
                            .size(10.0)
                            .strong()
                            .color(egui::Color32::from_gray(200)),
                    );
                }
                if let Some(ref prov) = ix_state.hovered_prov {
                    if ix_state.is_selected(prov.data_id.as_deref().unwrap_or("")) {
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!(
                                "({:.4}, {:.4})",
                                prov.world_x, prov.world_y
                            ))
                            .size(10.0)
                            .color(egui::Color32::from_gray(160)),
                        );
                    }
                }
            });
        });
}

/// Check if a compiled binding is in a data-driven transition and render a
/// fade overlay. Returns true if animation is still active and requires repaint.
pub(crate) fn handle_transition_animation(
    ui: &mut egui::Ui,
    response: &egui::Response,
    compiled: &petal_tongue_ipc::CompiledBinding,
    key: &str,
) -> bool {
    let Some(ref _prev) = compiled.prev_scene else {
        return false;
    };

    let transition_id = egui::Id::new("scene_transition").with(key);
    let current_time = ui.input(|i| i.time);
    let start: f64 = ui.data_mut(|d| *d.get_temp_mut_or_insert_with(transition_id, || current_time));
    let elapsed = current_time - start;
    let t = (elapsed / TRANSITION_DURATION_SECS).min(1.0);

    if t >= 1.0 {
        return false;
    }

    let progress = Easing::EaseOut.apply(t) as f32;
    let fade_alpha = ((1.0 - progress) * 80.0) as u8;

    if fade_alpha > 0 {
        let painter = ui.painter();
        painter.rect_filled(
            response.rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, fade_alpha),
        );
    }

    true
}

/// Fire an `InteractionApplyRequest` through the IPC registry.
pub(crate) fn fire_interaction_event(
    subs: Option<&Arc<RwLock<InteractionSubscriberRegistry>>>,
    intent: &str,
    target: &str,
    grammar_id: &str,
) {
    let Some(reg_arc) = subs else { return };
    let Ok(mut reg) = reg_arc.try_write() else { return };
    let req = InteractionApplyRequest {
        intent: intent.to_string(),
        targets: vec![target.to_string()],
        grammar_id: Some(grammar_id.to_string()),
    };
    let _response = reg.apply_interaction(&req);
}
