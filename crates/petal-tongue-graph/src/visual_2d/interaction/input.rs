// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pan/zoom, selection, and interactive node/edge gestures.

use egui::Pos2;

use super::super::{EdgeDraft, Visual2DRenderer};
use super::graph_ops::{create_node_at, delete_node, try_complete_edge_on_drag_release};

/// Handle user input (pan, zoom, click, double-click, drag)
pub fn handle_input(
    renderer: &mut Visual2DRenderer,
    response: &egui::Response,
    screen_center: Pos2,
) {
    if response.hovered() {
        let scroll_delta = response.ctx.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 {
            let zoom_factor = scroll_delta.mul_add(0.001, 1.0);
            renderer.zoom = (renderer.zoom * zoom_factor).clamp(0.1, 10.0);
        }
    }

    if renderer.interactive_mode
        && response.double_clicked()
        && let Some(mouse_pos) = response.interact_pointer_pos()
    {
        let world_pos = renderer.screen_to_world(mouse_pos, screen_center);
        create_node_at(renderer, world_pos);
    }

    if renderer.interactive_mode
        && response.drag_started()
        && let Some(mouse_pos) = response.interact_pointer_pos()
    {
        let world_pos = renderer.screen_to_world(mouse_pos, screen_center);

        let Ok(graph) = renderer.graph.read() else {
            tracing::error!("graph lock poisoned");
            return;
        };
        let node_under_cursor = graph.nodes().iter().find(|node| {
            let distance = node.position.distance_to(world_pos);
            distance < 20.0
        });

        if let Some(node) = node_under_cursor {
            renderer.dragging_node = Some(node.info.id.clone());
        }
    }

    if renderer.interactive_mode && response.dragged() {
        if let Some(ref dragging_id) = renderer.dragging_node {
            if response.drag_delta().length() > 10.0 {
                if renderer.drawing_edge.is_none() {
                    renderer.drawing_edge = Some(EdgeDraft {
                        from: dragging_id.clone(),
                        current_pos: response.interact_pointer_pos().unwrap_or_default(),
                    });
                } else if let Some(ref mut edge_draft) = renderer.drawing_edge {
                    edge_draft.current_pos = response.interact_pointer_pos().unwrap_or_default();
                }
            }
        } else {
            renderer.camera_offset += response.drag_delta();
            renderer.is_dragging = true;
        }
    } else {
        renderer.is_dragging = false;
    }

    if renderer.interactive_mode && response.drag_stopped() {
        try_complete_edge_on_drag_release(renderer, response, screen_center);
    }

    if response.clicked()
        && !renderer.is_dragging
        && renderer.drawing_edge.is_none()
        && let Some(mouse_pos) = response.interact_pointer_pos()
    {
        let world_pos = renderer.screen_to_world(mouse_pos, screen_center);

        let Ok(graph) = renderer.graph.read() else {
            tracing::error!("graph lock poisoned");
            return;
        };
        let clicked_node = graph.nodes().iter().find(|node| {
            let distance = node.position.distance_to(world_pos);
            distance < 20.0
        });

        if let Some(node) = clicked_node {
            renderer.selected_node = Some(node.info.id.clone());
        } else {
            renderer.selected_node = None;
        }
    }

    if renderer.interactive_mode {
        response.ctx.input(|i| {
            if (i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace))
                && let Some(selected_id) = renderer.selected_node.clone()
            {
                delete_node(renderer, selected_id.as_str());
                renderer.selected_node = None;
            }
        });
    }
}
