// SPDX-License-Identifier: AGPL-3.0-only
//! Interaction handling for 2D graph visualization (pan, zoom, create, edit).

use crate::capability_validator::{ValidationResult, validate_connection};
use egui::Pos2;
use petal_tongue_core::graph_engine::Position;
use petal_tongue_core::{PrimalHealthStatus, PrimalId, PrimalInfo, Properties, PropertyValue};

use super::{EdgeDraft, Visual2DRenderer};

/// Handle user input (pan, zoom, click, double-click, drag)
pub fn handle_input(
    renderer: &mut Visual2DRenderer,
    response: &egui::Response,
    screen_center: Pos2,
) {
    // Handle zoom (scroll wheel)
    if response.hovered() {
        let scroll_delta = response.ctx.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 {
            let zoom_factor = 1.0 + (scroll_delta * 0.001);
            renderer.zoom = (renderer.zoom * zoom_factor).clamp(0.1, 10.0);
        }
    }

    // Interactive mode: Handle double-click to create node
    if renderer.interactive_mode && response.double_clicked() {
        if let Some(mouse_pos) = response.interact_pointer_pos() {
            let world_pos = renderer.screen_to_world(mouse_pos, screen_center);
            create_node_at(renderer, world_pos);
        }
    }

    // Interactive mode: Handle drag to move node or create edge
    if renderer.interactive_mode && response.drag_started() {
        if let Some(mouse_pos) = response.interact_pointer_pos() {
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
    }

    // Interactive mode: Handle drag for edge creation
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

    // Interactive mode: Handle drag release for edge completion
    if renderer.interactive_mode && response.drag_stopped() {
        try_complete_edge_on_drag_release(renderer, response, screen_center);
    }

    // Handle node selection (click)
    if response.clicked() && !renderer.is_dragging && renderer.drawing_edge.is_none() {
        if let Some(mouse_pos) = response.interact_pointer_pos() {
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
    }

    // Interactive mode: Handle delete key
    if renderer.interactive_mode {
        response.ctx.input(|i| {
            if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                if let Some(selected_id) = renderer.selected_node.clone() {
                    delete_node(renderer, selected_id.as_str());
                    renderer.selected_node = None;
                }
            }
        });
    }
}

/// Try to complete an edge when drag is released (drop on target node).
fn try_complete_edge_on_drag_release(
    renderer: &mut Visual2DRenderer,
    response: &egui::Response,
    screen_center: Pos2,
) {
    if let Some(edge_draft) = renderer.drawing_edge.take() {
        if let Some(mouse_pos) = response.interact_pointer_pos() {
            let world_pos = renderer.screen_to_world(mouse_pos, screen_center);

            let target_id = {
                let Ok(graph) = renderer.graph.read() else {
                    tracing::error!("graph lock poisoned");
                    return;
                };
                graph
                    .nodes()
                    .iter()
                    .find(|node| {
                        let distance = node.position.distance_to(world_pos);
                        distance < 20.0 && node.info.id.as_str() != edge_draft.from.as_str()
                    })
                    .map(|node| node.info.id.clone())
            };

            if let Some(target) = target_id {
                create_edge(renderer, edge_draft.from, target);
            }
        }
    }
    renderer.dragging_node = None;
}

/// Create a new node at the given world position (interactive mode)
fn create_node_at(renderer: &mut Visual2DRenderer, world_pos: Position) {
    let Ok(mut graph) = renderer.graph.write() else {
        tracing::error!("graph lock poisoned");
        return;
    };

    let node_count = graph.nodes().len();
    let new_id = format!("interactive-node-{}", node_count + 1);

    let mut properties = Properties::new();
    properties.insert(
        "created_by".to_string(),
        PropertyValue::String("interactive-paint".to_string()),
    );
    properties.insert(
        "family_id".to_string(),
        PropertyValue::String("interactive".to_string()),
    );

    let new_primal = PrimalInfo {
        id: PrimalId::from(new_id.clone()),
        name: format!("Node {}", node_count + 1),
        primal_type: "custom".to_string(),
        endpoint: format!("interactive://{}", new_id),
        capabilities: vec!["interactive".to_string()],
        health: PrimalHealthStatus::Healthy,
        last_seen: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        endpoints: None,
        metadata: None,
        properties,
        #[expect(deprecated)]
        trust_level: None,
        #[expect(deprecated)]
        family_id: None,
    };

    graph.add_node(new_primal);

    if let Some(node) = graph.get_node_mut(&new_id) {
        node.position = world_pos;
    }

    drop(graph);
    renderer.selected_node = Some(PrimalId::from(new_id));
}

/// Create an edge between two nodes (interactive mode)
fn create_edge(renderer: &mut Visual2DRenderer, from: PrimalId, to: PrimalId) {
    use petal_tongue_core::TopologyEdge;

    let Ok(graph) = renderer.graph.read() else {
        tracing::error!("graph lock poisoned");
        return;
    };

    let edge_exists = graph.edges().iter().any(|e| {
        (e.from.as_str() == from.as_str() && e.to.as_str() == to.as_str())
            || (e.from.as_str() == to.as_str() && e.to.as_str() == from.as_str())
    });

    if edge_exists {
        return;
    }

    let from_node = graph.get_node(from.as_str());
    let to_node = graph.get_node(to.as_str());

    if let (Some(from_primal), Some(to_primal)) = (from_node, to_node) {
        let validation = validate_connection(&from_primal.info, &to_primal.info);

        match validation {
            ValidationResult::Invalid(reason) => {
                tracing::warn!("❌ Connection invalid: {}", reason);
                return;
            }
            ValidationResult::Warning(reason) => {
                tracing::info!("⚠️ Connection warning: {}", reason);
            }
            ValidationResult::Valid => {
                tracing::info!("✅ Connection validated");
            }
        }
    }

    drop(graph);

    let Ok(mut graph) = renderer.graph.write() else {
        tracing::error!("graph lock poisoned");
        return;
    };
    graph.add_edge(TopologyEdge {
        from,
        to,
        edge_type: "interactive".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });
}

/// Delete a node (interactive mode)
fn delete_node(renderer: &mut Visual2DRenderer, node_id: &str) {
    let Ok(mut graph) = renderer.graph.write() else {
        tracing::error!("graph lock poisoned");
        return;
    };
    graph.remove_node(node_id);
}
